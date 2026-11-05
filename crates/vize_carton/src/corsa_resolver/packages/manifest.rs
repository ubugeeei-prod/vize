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
    // An npm alias owns its package identity even when the dependency key is
    // the canonical name (e.g. typescript -> @typescript/typescript6).
    if let Some(aliased) = dependency_spec(project, name).and_then(|spec| spec.strip_prefix("npm:"))
    {
        return aliased == target
            || aliased
                .strip_prefix(target)
                .is_some_and(|version| version.starts_with('@'));
    }
    name == target
}

#[cfg(test)]
mod tests {
    use super::declaration_targets;

    #[test]
    fn npm_alias_identity_takes_precedence_over_the_dependency_key() {
        for section in ["dependencies", "devDependencies", "optionalDependencies"] {
            let project = serde_json::json!({(section): {
                "typescript": "npm:@typescript/typescript6@6.0.2",
                "native": "npm:typescript@7.0.2",
                "preview": "npm:@typescript/native-preview@7.0.0",
                "unrelated": "npm:typescript-other@7.0.2"
            }});
            assert!(!declaration_targets(&project, "typescript", "typescript"));
            assert!(declaration_targets(
                &project,
                "typescript",
                "@typescript/typescript6"
            ));
            assert!(declaration_targets(&project, "native", "typescript"));
            assert!(declaration_targets(
                &project,
                "preview",
                "@typescript/native-preview"
            ));
            assert!(!declaration_targets(&project, "unrelated", "typescript"));
        }
        for spec in [
            "6.0.3",
            "^7.0.2",
            "workspace:*",
            "npm:typescript@7.0.2",
            "npm:typescript",
        ] {
            let project = serde_json::json!({"devDependencies": {"typescript": spec}});
            assert!(declaration_targets(&project, "typescript", "typescript"));
        }
    }
}
