#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! serde_json = "1"
//!
//! [package]
//! edition = "2024"
//! ```

use serde_json::Value;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<u8, String> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    if args.len() < 2 {
        return Err(
            "Usage: rust-script tools/commands/editors/vscode/run-package-bin.rs <package-name> <bin-name> [args...]"
                .to_string(),
        );
    }
    let package_name = args[0].to_string_lossy();
    let bin_name = args[1].to_string_lossy();
    let cwd = env::current_dir().map_err(|error| format!("cannot read current dir: {error}"))?;
    let package_root = resolve_package_root(&cwd, &package_name)?;
    let package_json = read_json(&package_root.join("package.json"))?;
    let relative_bin = resolve_bin_path(&package_json, &bin_name)?;
    let bin_path = package_root.join(relative_bin);
    if !bin_path.exists() {
        return Err(format!(
            "Package bin \"{bin_name}\" for {package_name} does not exist at {}",
            bin_path.display()
        ));
    }

    let run_with_node = cfg!(windows)
        && !bin_path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| matches!(ext.to_ascii_lowercase().as_str(), "bat" | "cmd" | "exe"));
    let mut command = if run_with_node {
        let mut command = Command::new("node");
        command.arg(&bin_path);
        command
    } else {
        Command::new(&bin_path)
    };
    let status = command
        .args(&args[2..])
        .status()
        .map_err(|error| error.to_string())?;
    Ok(status.code().unwrap_or(1).clamp(0, 255) as u8)
}

fn resolve_package_root(cwd: &Path, name: &str) -> Result<PathBuf, String> {
    let node_modules = cwd.join("node_modules");
    let direct = name
        .split('/')
        .fold(node_modules.clone(), |path, part| path.join(part));
    if direct.join("package.json").exists() {
        return Ok(direct);
    }

    let package_map_path = node_modules.join(".package-map.json");
    if !package_map_path.exists() {
        return Err(format!(
            "Cannot resolve {name}: {} does not exist",
            package_map_path.display()
        ));
    }
    let package_map = read_json(&package_map_path)?;
    let packages = package_map
        .get("packages")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            format!(
                "Cannot resolve {name}: {} has no packages object",
                package_map_path.display()
            )
        })?;
    let expected_version = exact_manifest_version(cwd, name)?;
    let mut candidates = packages
        .iter()
        .filter_map(|(id, entry)| {
            let url = entry.get("url")?.as_str()?;
            if !normalize_separators(url).ends_with(&format!("/node_modules/{name}")) {
                return None;
            }
            let path = node_modules.join(url);
            path.join("package.json").exists().then_some((
                score_package_id(id, name, expected_version.as_deref()),
                path,
            ))
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| right.0.cmp(&left.0));
    candidates
        .into_iter()
        .next()
        .map(|(_, path)| path)
        .ok_or_else(|| {
            format!(
                "Cannot resolve {name}: no matching package entry in {}",
                package_map_path.display()
            )
        })
}

fn exact_manifest_version(cwd: &Path, name: &str) -> Result<Option<String>, String> {
    let manifest_path = cwd.join("package.json");
    if !manifest_path.exists() {
        return Ok(None);
    }
    let manifest = read_json(&manifest_path)?;
    for key in ["dependencies", "devDependencies", "optionalDependencies"] {
        if let Some(value) = manifest
            .get(key)
            .and_then(Value::as_object)
            .and_then(|deps| deps.get(name))
            .and_then(Value::as_str)
        {
            if is_exact_version(value) {
                return Ok(Some(value.to_string()));
            }
        }
    }
    Ok(None)
}

fn is_exact_version(value: &str) -> bool {
    let core = value.split(['-', '+']).next().unwrap_or(value);
    let parts = core.split('.').collect::<Vec<_>>();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
}

fn score_package_id(id: &str, name: &str, expected_version: Option<&str>) -> u8 {
    let mut score = 0;
    if expected_version.is_some_and(|version| id.starts_with(&format!("{name}@{version}"))) {
        score += 2;
    }
    if !id.contains('(') {
        score += 1;
    }
    score
}

fn resolve_bin_path(package_json: &Value, name: &str) -> Result<String, String> {
    let bin = package_json.get("bin");
    if let Some(bin) = bin.and_then(Value::as_str) {
        return Ok(bin.to_string());
    }
    if let Some(bin) = bin
        .and_then(Value::as_object)
        .and_then(|bin| bin.get(name))
        .and_then(Value::as_str)
    {
        return Ok(bin.to_string());
    }
    Err(format!(
        "Package {} does not expose bin \"{name}\"",
        package_json
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("(unknown)")
    ))
}

fn read_json(path: &Path) -> Result<Value, String> {
    serde_json::from_str(
        &fs::read_to_string(path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?,
    )
    .map_err(|error| format!("failed to parse {}: {error}", path.display()))
}

fn normalize_separators(value: &str) -> String {
    value.replace(std::path::MAIN_SEPARATOR, "/")
}
