//! Fresh project-marker discovery. Source operation counts compile out of
//! ordinary builds; profiling retains the exact traversal and marker policy.

use std::path::{Path, PathBuf};

pub(super) fn document_root(path: &Path, workspace: &Path) -> PathBuf {
    vize_l0::micro_span!("maestro.project.document_root", {
        let mut directory = path.parent().unwrap_or(workspace);
        let mut package_root = None;
        let mut vite_root: Option<PathBuf> = None;
        while directory.starts_with(workspace) {
            let mut has_package = false;
            let mut has_vite = false;
            let mut has_project_config = false;
            vize_l0::micro_count!("fs.read_dir.calls", 1);
            if let Ok(entries) = std::fs::read_dir(directory) {
                for entry in entries.flatten() {
                    vize_l0::micro_count!("fs.read_dir.entries", 1);
                    let path = entry.path();
                    has_package |=
                        path.file_name().and_then(|name| name.to_str()) == Some("package.json");
                    has_vite |= is_vite_config_marker(&path);
                    has_project_config |= is_typescript_config_marker(&path);
                }
            }
            if has_project_config {
                if package_root.is_some()
                    && let Some(vite_root) = vite_root.as_ref()
                {
                    return vite_root.clone();
                }
                return directory.to_path_buf();
            }
            if vite_root.is_none() && package_root.is_none() && has_vite {
                vite_root = Some(directory.to_path_buf());
            }
            if package_root.is_none() && has_package {
                package_root = Some(directory.to_path_buf());
            }
            if directory == workspace {
                break;
            }
            let Some(parent) = directory.parent() else {
                break;
            };
            directory = parent;
        }
        vite_root
            .or(package_root)
            .unwrap_or_else(|| workspace.to_path_buf())
    })
}

fn is_vite_config_marker(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("vite.config."))
}

fn is_typescript_config_marker(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            (name.starts_with("tsconfig") && name.ends_with(".json")) || name == "jsconfig.json"
        })
}

pub(super) fn is_config_marker(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            is_vite_config_marker(path)
                || is_typescript_config_marker(path)
                || name == "package.json"
        })
}
