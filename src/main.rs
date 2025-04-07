#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

use clap::{App, Arg};

use std::env;
use std::process::{Command, ExitStatus, ExitCode};

mod package;

fn main() -> ExitCode {
    let matched_args = App::new("cargo build-dependencies")
        .arg(Arg::with_name("build-dependencies"))
        .arg(Arg::with_name("release").long("release"))
        .arg(Arg::with_name("profile").long("profile").takes_value(true))
        .arg(Arg::with_name("target").long("target").takes_value(true))
        .arg(Arg::with_name("ignore-errors").long("ignore-errors"))
        .arg(
            Arg::with_name("exclude")
                .long("exclude")
                .short("x")
                .takes_value(true)
                .multiple(true),
        )
        .get_matches();

    let is_release = matched_args.is_present("release");
    let profile = match matched_args.value_of("profile") {
        Some(value) => {
            if is_release {
                panic!("`--release` is short for `--profile release`, you should only specify one of these flags");
            } else {
                value
            }
        }
        None => {
            if is_release {
                "release"
            } else {
                "dev"
            }
        }
    };
    let target = match matched_args.value_of("target") {
        Some(value) => value,
        None => "",
    };

    let cargo_toml = package::get_toml("Cargo.toml").expect("Can't get Cargo.toml");
    let cargo_lock = package::get_toml("Cargo.lock").expect("Can't get Cargo.lock");
    let mut dependencies = package::get_dependencies(&cargo_toml, &cargo_lock);

    if let Some(excluded_packages) = matched_args.values_of("exclude") {
        for excluded in excluded_packages {
            let existing = dependencies.remove(excluded);
            if existing.is_none() {
                panic!("{} was marked as excluded, but it is not a dependency of the top-level `Cargo.toml`", excluded);
            }
        }
    }

    if dependencies.is_empty() {
        panic!("Can't find dependencies");
    }

    println!("Start building packages");

    let mut exit_status = ExitStatus::default();

    for (name, version) in dependencies {
        let package_status = build_package(
            &format!("{}:{}", name, version),
            profile,
            &target,
            matched_args.is_present("ignore-errors"),
        );

        if !package_status.success() {
            exit_status = package_status;
        }
    }

    println!("Finished");

    if exit_status.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn build_package(pkg_name: &str, profile: &str, target: &str, ignore_errors: bool) -> ExitStatus {
    println!("Building package: {:?}", pkg_name);

    let mut command = Command::new("cargo");
    let command_with_args = command.args(["build", "-p", pkg_name, "--profile", profile]);

    let command_with_args = if !target.is_empty() {
        command_with_args.arg("--target=".to_string() + target)
    } else {
        command_with_args
    };

    execute_command(command_with_args, |status| {
        if ignore_errors {
            eprintln!("{}", status)
        } else {
            panic!("{}", status)
        }
    })
}

fn execute_command<F: FnOnce(String)>(command: &mut Command, report: F) -> ExitStatus {
    let mut child = command
        .envs(env::vars())
        .spawn()
        .expect("Failed to execute process");

    let exit_status = child.wait().expect("Failed to run command");

    if !exit_status.success() {
        match exit_status.code() {
            Some(code) => report(format!("Exited with status code: {}", code)),
            None => report(format!("Process terminated by signal")),
        }
    }

    exit_status
}
