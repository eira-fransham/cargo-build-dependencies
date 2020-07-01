use toml::Value as Toml;

use std::fs::File;
use std::io::prelude::*;

pub fn get_toml(file_path: &str) -> std::io::Result<Toml> {
    let mut toml_file = File::open(file_path)?;
    let mut toml_string = String::new();
    toml_file.read_to_string(&mut toml_string)?;

    Ok(toml_string.parse().expect("Failed to parse toml"))
}

pub fn get_dependencies<'a>(cargo_toml: &'a Toml, cargo_lock: &'a Toml) -> Vec<String> {
    let dependencies = match cargo_toml.get("dependencies") {
        Some(&Toml::Table(ref packages)) => packages
            .into_iter()
            .map(|(name, _value)| name.to_string())
            .collect(),
        Some(_) => vec![],
        None => vec![],
    };

    match cargo_lock.get("package") {
        Some(&Toml::Array(ref packages)) => get_packages(&packages.clone(), &dependencies),
        Some(_) => vec![],
        None => vec![],
    }
}

fn get_packages(packages: &Vec<Toml>, dependencies: &Vec<String>) -> Vec<String> {
    packages
        .into_iter()
        .filter_map(|package| match package {
            Toml::Table(map) => {
                let name = get_string_field(map.get("name"));
                let version = get_string_field(map.get("version"));
                if dependencies.contains(&name) {
                    Some(format!("{}:{}", name, version))
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect()
}

fn get_string_field<'a>(field: Option<&'a Toml>) -> String {
    field
        .map(|n| n.to_string().replace("\"", ""))
        .unwrap_or_default()
}
