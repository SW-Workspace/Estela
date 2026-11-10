#![forbid(unsafe_code)]
use std::process::Command;

use serde_json::Value;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let metadata = metadata()?;
    match std::env::args().nth(1).as_deref() {
        Some("check-deps") => check_dependencies(&metadata),
        Some("check-sqlx") => check_sqlx(&metadata),
        _ => Err("Unsupported task; available: check-deps, check-sqlx".into()),
    }
}

fn metadata() -> Result<Value> {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps", "--locked"])
        .output()?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned().into());
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn check_dependencies(metadata: &Value) -> Result<()> {
    let rules: Value = serde_json::from_str(include_str!("../dependencies.json"))?;
    for package in metadata["packages"].as_array().ok_or("Missing packages")? {
        let name = package["name"].as_str().ok_or("Missing name")?;
        for dependency in package["dependencies"]
            .as_array()
            .ok_or("Missing dependencies")?
        {
            check_dependency(name, dependency, &rules)?;
        }
    }
    println!("Dependency rules passed");
    Ok(())
}

fn check_dependency(name: &str, dependency: &Value, rules: &Value) -> Result<()> {
    let dependency_name = dependency["name"].as_str().ok_or("Missing name")?;
    if !dependency_name.starts_with("estela-") {
        return Ok(());
    }
    let allowed = rules[name].as_array().is_some_and(|list| {
        list.iter()
            .any(|value| value.as_str() == Some(dependency_name))
    });
    if !allowed {
        return Err(format!("Forbidden dependency: {name} -> {dependency_name}").into());
    }
    if dependency_name.starts_with("estela-platform-") && name == "estela-engine" {
        let expected = if dependency_name.ends_with("-win") {
            "cfg(windows)"
        } else {
            "cfg(target_os = \"linux\")"
        };
        if dependency["target"].as_str() != Some(expected) {
            return Err(format!("Incorrect target: {dependency_name}").into());
        }
    }
    Ok(())
}

fn check_sqlx(metadata: &Value) -> Result<()> {
    for package in metadata["packages"].as_array().ok_or("Missing packages")? {
        let dependencies = package["dependencies"]
            .as_array()
            .ok_or("Missing dependencies")?;
        if dependencies
            .iter()
            .any(|dependency| dependency["name"] == "sqlx")
        {
            let status = Command::new("cargo")
                .args(["sqlx", "prepare", "--workspace", "--check"])
                .status()?;
            if !status.success() {
                return Err("SQLx offline metadata check failed".into());
            }
            return Ok(());
        }
    }
    println!("SQLx check not applicable: no workspace crate depends on SQLx yet");
    Ok(())
}
