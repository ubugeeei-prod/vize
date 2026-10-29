use std::path::Path;

use super::valid_package_name;

pub(super) fn read_manifest(package: &Path) -> Option<serde_json::Value> {
    serde_json::from_str(&std::fs::read_to_string(package.join("package.json")).ok()?).ok()
}

pub(super) fn declared_dependency_names(project: &serde_json::Value) -> Vec<&str> {
    let mut names = Vec::new();
    for section in ["dependencies", "devDependencies", "optionalDependencies"] {
        if let Some(dependencies) = project.get(section).and_then(serde_json::Value::as_object) {
            for name in dependencies.keys() {
                if valid_package_name(name) {
                    names.push(name.as_str());
                }
            }
        }
    }
    names.sort_unstable();
    names.dedup();
    names
}

fn dependency_spec<'a>(project: &'a serde_json::Value, name: &str) -> Option<&'a str> {
    ["dependencies", "devDependencies", "optionalDependencies"]
        .iter()
        .find_map(|section| project.get(section)?.get(name)?.as_str())
}

pub(super) fn declaration_targets(project: &serde_json::Value, name: &str, target: &str) -> bool {
    if name == target {
        return true;
    }
    dependency_spec(project, name).is_some_and(|spec| {
        spec.strip_prefix("npm:").is_some_and(|aliased| {
            aliased == target
                || aliased
                    .strip_prefix(target)
                    .is_some_and(|version| version.starts_with('@'))
        })
    })
}
