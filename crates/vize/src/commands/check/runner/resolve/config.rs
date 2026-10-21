//! Locating the nearest TypeScript or JavaScript project configuration.

use std::path::{Path, PathBuf};

pub(in super::super) fn project_config_path(dir: &Path) -> Option<PathBuf> {
    ["tsconfig.json", "jsconfig.json"]
        .into_iter()
        .map(|name| dir.join(name))
        .find(|candidate| candidate.exists())
}

pub(in super::super) fn find_nearest_tsconfig_path(path: &Path) -> Option<PathBuf> {
    let mut current = if path.is_dir() {
        Some(path)
    } else {
        path.parent()
    };

    while let Some(dir) = current {
        if let Some(config) = project_config_path(dir) {
            return Some(config);
        }
        current = dir.parent();
    }

    None
}

pub(in super::super) fn find_nearest_tsconfig_path_within(
    path: &Path,
    boundary: &Path,
) -> Option<PathBuf> {
    let mut current = if path.is_dir() {
        Some(path)
    } else {
        path.parent()
    };

    while let Some(dir) = current.filter(|dir| dir.starts_with(boundary)) {
        if let Some(config) = project_config_path(dir) {
            return Some(config);
        }
        if dir == boundary {
            break;
        }
        current = dir.parent();
    }

    None
}

pub(in super::super) fn find_nearest_tsconfig_dir(path: &Path) -> Option<PathBuf> {
    find_nearest_tsconfig_path(path).and_then(|config| config.parent().map(Path::to_path_buf))
}

#[cfg(test)]
mod tests {
    use super::super::{resolve_project_root, resolve_tsconfig_path};
    use super::{find_nearest_tsconfig_dir, find_nearest_tsconfig_path, project_config_path};

    #[test]
    fn discovers_jsconfig_from_default_cwd_and_plain_javascript_inputs() {
        let project = tempfile::tempdir().unwrap();
        let root = project.path();
        let source = root.join("src/value.js");
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::write(&source, "export const value = 1;\n").unwrap();
        let config = root.join("jsconfig.json");
        std::fs::write(&config, r#"{"compilerOptions":{"checkJs":true}}"#).unwrap();

        for inputs in [Vec::new(), vec![source.clone()]] {
            let resolved = resolve_project_root(None, root, &inputs);
            assert_eq!(resolved, root);
            assert_eq!(
                resolve_tsconfig_path(None, root, &resolved, &inputs),
                Some(config.clone())
            );
        }
        assert_eq!(find_nearest_tsconfig_dir(&source), Some(root.to_path_buf()));
        assert_eq!(find_nearest_tsconfig_path(&source), Some(config));
    }

    #[test]
    fn nearest_package_jsconfig_precedes_an_ancestor_tsconfig() {
        let project = tempfile::tempdir().unwrap();
        let root = project.path();
        std::fs::write(root.join("tsconfig.json"), "{}").unwrap();
        let package = root.join("packages/app");
        let source = package.join("src/value.mjs");
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::write(&source, "export const value = 1;\n").unwrap();
        let config = package.join("jsconfig.json");
        std::fs::write(&config, "{}").unwrap();

        assert_eq!(find_nearest_tsconfig_path(&source), Some(config.clone()));
        assert_eq!(
            resolve_project_root(None, source.parent().unwrap(), &[]),
            package
        );
        assert_eq!(
            resolve_tsconfig_path(None, root, &package, &[source]),
            Some(config)
        );
    }

    #[test]
    fn same_directory_tsconfig_precedence_preserves_explicit_jsconfig_selection() {
        let project = tempfile::tempdir().unwrap();
        let root = project.path();
        let tsconfig = root.join("tsconfig.json");
        let jsconfig = root.join("jsconfig.json");
        std::fs::write(&jsconfig, "{}").unwrap();
        std::fs::write(&tsconfig, "{}").unwrap();

        assert_eq!(project_config_path(root), Some(tsconfig.clone()));
        assert_eq!(resolve_tsconfig_path(None, root, root, &[]), Some(tsconfig));
        assert_eq!(
            resolve_tsconfig_path(Some(std::path::Path::new("jsconfig.json")), root, root, &[]),
            Some(jsconfig)
        );
    }
}
