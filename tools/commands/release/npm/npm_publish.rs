#![allow(dead_code)]

use serde_json::{Map, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

const DEPENDENCY_SECTIONS: &[&str] = &[
    "dependencies",
    "optionalDependencies",
    "peerDependencies",
    "devDependencies",
];

pub fn prepare_publish_manifest(package_dir: &Path) -> Result<(), String> {
    let package_json_path = package_dir.join("package.json");
    let mut package_json = read_json(&package_json_path)?;
    let repo_root = find_repo_root(package_dir);

    let Some(repo_root) = repo_root else {
        let unresolved = unresolved_protocols_without_repo_root(&package_json);
        if unresolved.is_empty() {
            return Ok(());
        }
        return Err(format!(
            "Cannot prepare {}\n{}",
            package_json_path.display(),
            unresolved.join("\n")
        ));
    };

    let workspace_versions = collect_workspace_package_versions(&repo_root.join("npm"))?;
    let catalog_versions = parse_catalog_versions(
        &fs::read_to_string(repo_root.join("pnpm-workspace.yaml"))
            .map_err(|error| format!("cannot read pnpm-workspace.yaml: {error}"))?,
    );
    let native_binary_version = workspace_versions
        .get("@vizejs/native")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            package_json
                .get("version")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_default();

    let unresolved = normalize_dependency_sections(
        &mut package_json,
        &workspace_versions,
        &catalog_versions,
        &native_binary_version,
    );
    if !unresolved.is_empty() {
        return Err(format!(
            "Cannot prepare {}\n{}",
            package_json_path.display(),
            unresolved.join("\n")
        ));
    }

    write_json_pretty(&package_json_path, &package_json)?;
    println!(
        "Prepared npm publish manifest at {}",
        package_json_path.display()
    );
    Ok(())
}

pub fn dependency_sections() -> &'static [&'static str] {
    DEPENDENCY_SECTIONS
}

pub fn read_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("cannot parse {}: {error}", path.display()))
}

pub fn write_json_pretty(path: &Path, value: &Value) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    fs::write(path, format!("{text}\n"))
        .map_err(|error| format!("cannot write {}: {error}", path.display()))
}

pub fn find_repo_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    loop {
        if current.join("pnpm-workspace.yaml").is_file() && current.join("npm").is_dir() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn collect_workspace_package_versions(directory: &Path) -> Result<BTreeMap<String, Value>, String> {
    let mut versions = BTreeMap::new();
    collect_workspace_package_versions_inner(directory, &mut versions)?;
    Ok(versions)
}

fn collect_workspace_package_versions_inner(
    directory: &Path,
    versions: &mut BTreeMap<String, Value>,
) -> Result<(), String> {
    if !directory.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(directory)
        .map_err(|error| format!("cannot read {}: {error}", directory.display()))?
    {
        let entry = entry.map_err(|error| error.to_string())?;
        let name = entry.file_name();
        if matches!(name.to_str(), Some("node_modules" | ".git" | "_build")) {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            collect_workspace_package_versions_inner(&path, versions)?;
        } else if name == "package.json" {
            let package_json = read_json(&path)?;
            if let (Some(name), Some(version)) = (
                package_json.get("name").and_then(Value::as_str),
                package_json.get("version").and_then(Value::as_str),
            ) {
                versions.insert(name.to_string(), Value::String(version.to_string()));
            }
        }
    }
    Ok(())
}

fn yaml_scalar(value: &str) -> String {
    let trimmed = value.trim();
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        trimmed[1..trimmed.len().saturating_sub(1)].to_string()
    } else {
        trimmed.to_string()
    }
}

fn parse_catalog_versions(content: &str) -> BTreeMap<String, String> {
    let mut versions = BTreeMap::new();
    let mut in_catalogs = false;
    let mut current_catalog = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !in_catalogs {
            if trimmed == "catalogs:" {
                in_catalogs = true;
            }
            continue;
        }
        if !line.starts_with("  ") {
            break;
        }
        if let Some(catalog) = line
            .strip_prefix("  ")
            .and_then(|rest| rest.strip_suffix(':'))
            .filter(|rest| !rest.starts_with(' '))
        {
            current_catalog = yaml_scalar(catalog);
            continue;
        }
        if current_catalog.is_empty() || !line.starts_with("    ") {
            continue;
        }
        let Some((dependency, version)) = line.trim_start().split_once(':') else {
            continue;
        };
        versions.insert(
            format!("{}\0{}", current_catalog, yaml_scalar(dependency)),
            yaml_scalar(version),
        );
    }
    versions
}

fn normalize_workspace_spec(spec: &str, version: &str) -> String {
    match spec.strip_prefix("workspace:").unwrap_or(spec) {
        "" | "*" => version.to_string(),
        "^" => format!("^{version}"),
        "~" => format!("~{version}"),
        _ => version.to_string(),
    }
}

fn normalize_dependency_sections(
    package_json: &mut Value,
    workspace_versions: &BTreeMap<String, Value>,
    catalog_versions: &BTreeMap<String, String>,
    native_binary_version: &str,
) -> Vec<String> {
    let mut unresolved = Vec::new();
    let Some(package_object) = package_json.as_object_mut() else {
        return vec!["package.json must be an object".to_string()];
    };

    for section in DEPENDENCY_SECTIONS {
        let Some(dependencies) = package_object
            .get_mut(*section)
            .and_then(Value::as_object_mut)
        else {
            continue;
        };
        normalize_dependency_object(
            *section,
            dependencies,
            workspace_versions,
            catalog_versions,
            native_binary_version,
            &mut unresolved,
        );
    }
    unresolved
}

fn normalize_dependency_object(
    section: &str,
    dependencies: &mut Map<String, Value>,
    workspace_versions: &BTreeMap<String, Value>,
    catalog_versions: &BTreeMap<String, String>,
    native_binary_version: &str,
    unresolved: &mut Vec<String>,
) {
    let names = dependencies.keys().cloned().collect::<Vec<_>>();
    for dependency_name in names {
        let Some(version_spec) = dependencies.get(&dependency_name).and_then(Value::as_str) else {
            continue;
        };
        if version_spec.starts_with("workspace:") {
            if let Some(version) = workspace_versions
                .get(&dependency_name)
                .and_then(Value::as_str)
            {
                dependencies.insert(
                    dependency_name.clone(),
                    Value::String(normalize_workspace_spec(version_spec, version)),
                );
            } else {
                unresolved.push(format!(
                    "Missing workspace version for {dependency_name} referenced from {section}"
                ));
            }
            continue;
        }

        if let Some(catalog_name) = version_spec.strip_prefix("catalog:") {
            if catalog_name == "native-binaries" && dependency_name.starts_with("@vizejs/native-") {
                dependencies.insert(
                    dependency_name.clone(),
                    Value::String(native_binary_version.to_string()),
                );
                continue;
            }

            let key = format!("{catalog_name}\0{dependency_name}");
            if let Some(version) = catalog_versions.get(&key) {
                dependencies.insert(dependency_name.clone(), Value::String(version.clone()));
            } else {
                unresolved.push(format!(
                    "Missing catalog version for {dependency_name} from {catalog_name} in {section}"
                ));
            }
        }
    }
}

fn unresolved_protocols_without_repo_root(package_json: &Value) -> Vec<String> {
    let mut unresolved = Vec::new();
    for section in DEPENDENCY_SECTIONS {
        let Some(dependencies) = package_json.get(*section).and_then(Value::as_object) else {
            continue;
        };
        for (dependency_name, version_spec) in dependencies {
            if version_spec
                .as_str()
                .map(|version| version.starts_with("workspace:") || version.starts_with("catalog:"))
                .unwrap_or(false)
            {
                unresolved.push(format!(
                    "Cannot normalize {dependency_name} from {section} because the repository root could not be located"
                ));
            }
        }
    }
    unresolved
}
