#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//!
//! [package]
//! edition = "2024"
//! ```

#[path = "../../../support/common.rs"]
mod common;

use serde_json::Value;
use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
    process::ExitCode,
};

#[derive(Debug, PartialEq, Eq)]
struct Options {
    package_json_path: PathBuf,
    version_package_json_path: PathBuf,
    print: bool,
}

fn main() -> ExitCode {
    common::main_result(run())
}

fn run() -> Result<(), String> {
    let options = parse_args(env::args_os().skip(1).collect())?;
    let version_json = common::read_json(&options.version_package_json_path)?;
    let version = package_version(&version_json, &options.version_package_json_path)?;
    let mut package_json = common::read_json(&options.package_json_path)?;

    inject_native_optional_dependency_versions(&mut package_json, &version)
        .map_err(|error| format!("{}: {error}", options.package_json_path.display()))?;
    common::write_json_pretty(&options.package_json_path, &package_json)?;
    println!("Updated optionalDependencies to native version {version}");
    if options.print {
        let updated =
            serde_json::to_string_pretty(&package_json).map_err(|error| error.to_string())?;
        println!("{updated}");
    }
    Ok(())
}

fn parse_args(args: Vec<OsString>) -> Result<Options, String> {
    if args.is_empty() {
        return Err("Usage: rust-script tools/commands/release/npm/inject-native-optional-deps.rs <package-json> [version-package-json] [--print]".to_string());
    }

    let mut positional = Vec::new();
    let mut print = false;
    for arg in args {
        if arg == "--print" {
            print = true;
        } else {
            positional.push(PathBuf::from(arg));
        }
    }

    let Some(package_json_path) = positional.first().cloned() else {
        return Err("missing package-json path".to_string());
    };
    if let Some(extra) = positional.get(2) {
        return Err(format!(
            "too many positional arguments: {}",
            extra.display()
        ));
    }
    let version_package_json_path = positional
        .get(1)
        .cloned()
        .unwrap_or_else(|| package_json_path.clone());

    Ok(Options {
        package_json_path,
        version_package_json_path,
        print,
    })
}

fn package_version(package_json: &Value, path: &Path) -> Result<String, String> {
    package_json
        .get("version")
        .and_then(Value::as_str)
        .filter(|version| !version.is_empty())
        .map(str::to_string)
        .ok_or_else(|| format!("Failed to read version from {}", path.display()))
}

fn inject_native_optional_dependency_versions(
    package_json: &mut Value,
    version: &str,
) -> Result<(), String> {
    let Some(package_object) = package_json.as_object_mut() else {
        return Err("package.json must be an object".to_string());
    };
    let Some(optional_dependencies) = package_object
        .get_mut("optionalDependencies")
        .and_then(Value::as_object_mut)
    else {
        return Ok(());
    };

    for dependency_name in optional_dependencies.keys().cloned().collect::<Vec<_>>() {
        if dependency_name.starts_with("@vizejs/native-") {
            optional_dependencies.insert(dependency_name, Value::String(version.to_string()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn updates_only_native_optional_dependency_versions() {
        let mut package_json = json!({
            "optionalDependencies": {
                "@vizejs/native-linux-x64-gnu": "0.0.1",
                "@vizejs/native-darwin-arm64": "0.0.1",
                "@other/native-linux-x64": "9.9.9",
                "fsevents": "^2.3.3"
            }
        });

        inject_native_optional_dependency_versions(&mut package_json, "1.2.3-beta.1").unwrap();

        assert_eq!(
            package_json["optionalDependencies"],
            json!({
                "@other/native-linux-x64": "9.9.9",
                "@vizejs/native-darwin-arm64": "1.2.3-beta.1",
                "@vizejs/native-linux-x64-gnu": "1.2.3-beta.1",
                "fsevents": "^2.3.3"
            })
        );
    }

    #[test]
    fn accepts_manifests_without_optional_dependencies() {
        let mut package_json = json!({ "name": "vize" });

        inject_native_optional_dependency_versions(&mut package_json, "1.2.3").unwrap();

        assert_eq!(package_json, json!({ "name": "vize" }));
    }

    #[test]
    fn defaults_version_manifest_to_target_manifest() {
        assert_eq!(
            parse_args(vec![OsString::from("npm/cli/package.json")]).unwrap(),
            Options {
                package_json_path: PathBuf::from("npm/cli/package.json"),
                version_package_json_path: PathBuf::from("npm/cli/package.json"),
                print: false,
            }
        );
    }

    #[test]
    fn rejects_extra_positional_paths() {
        assert!(parse_args(vec!["one".into(), "two".into(), "three".into()]).is_err());
    }
}
