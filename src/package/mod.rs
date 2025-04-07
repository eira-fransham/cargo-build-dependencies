use toml::Value as Toml;

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::prelude::*;

/// Map from package to version.
type Packages = HashMap<String, String>;

pub fn get_toml(file_path: &str) -> std::io::Result<Toml> {
    let mut toml_file = File::open(file_path)?;
    let mut toml_string = String::new();
    toml_file.read_to_string(&mut toml_string)?;

    Ok(toml_string.parse().expect("Failed to parse toml"))
}

pub fn get_dependencies<'a>(cargo_toml: &'a Toml, cargo_lock: &'a Toml) -> Packages {
    let dependencies = get_toml_dependencies(cargo_toml);
    get_lock_dependencies(cargo_lock, &dependencies)
}

fn get_lock_dependencies<'a>(cargo_lock: &'a Toml, dependencies: &HashSet<String>) -> Packages {
    match cargo_lock.get("package") {
        Some(&Toml::Array(ref packages)) => get_packages(&packages.clone(), dependencies),
        Some(_) => Default::default(),
        None => Default::default(),
    }
}

fn get_toml_dependencies<'a>(cargo_toml: &'a Toml) -> HashSet<String> {
    match cargo_toml.get("dependencies").or_else(|| {
        cargo_toml
            .get("workspace")
            .and_then(|workspace| workspace.get("dependencies"))
    }) {
        Some(&Toml::Table(ref packages)) => packages
            .into_iter()
            .map(|(name, _value)| name.to_string())
            .collect(),
        Some(_) => Default::default(),
        None => Default::default(),
    }
}

fn get_packages(packages: &Vec<Toml>, dependencies: &HashSet<String>) -> Packages {
    packages
        .into_iter()
        .filter_map(|package| match package {
            Toml::Table(map) => {
                let name = get_string_field(map.get("name"));
                let version = get_string_field(map.get("version"));
                if dependencies.contains(&name.to_string()) {
                    Some((name.to_string(), version.to_string()))
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect()
}

fn get_string_field<'a>(field: Option<&'a Toml>) -> &str {
    field.map(|n| n.as_str()).flatten().unwrap_or_default()
}

#[cfg(test)]
mod test;
