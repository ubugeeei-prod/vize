//! Fresh project-marker discovery. Source operation counts compile out of
//! ordinary builds; profiling retains the exact traversal and marker policy.

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

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
                    match config_marker_name(&entry.file_name()) {
                        Some(ConfigMarker::Package) => has_package = true,
                        Some(ConfigMarker::Vite) => has_vite = true,
                        Some(ConfigMarker::TypeScript) => {
                            has_project_config = true;
                            break;
                        }
                        None => {}
                    }
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

#[derive(Debug, Eq, PartialEq)]
enum ConfigMarker {
    Package,
    Vite,
    TypeScript,
}

// Most entries are source files. Reject them using borrowed OS bytes, without
// constructing a full path or validating their names as UTF8. Candidate marker
// names retain the previous UTF8 requirement, including non-UTF8 suffixes.
fn config_marker_name(name: &OsStr) -> Option<ConfigMarker> {
    let bytes = name.as_encoded_bytes();
    if bytes == b"package.json" {
        return Some(ConfigMarker::Package);
    }
    if bytes == b"jsconfig.json" {
        return Some(ConfigMarker::TypeScript);
    }
    let vite = bytes.starts_with(b"vite.config.");
    let typescript = bytes.starts_with(b"tsconfig") && bytes.ends_with(b".json");
    if !(vite || typescript) || name.to_str().is_none() {
        return None;
    }
    Some(if vite {
        ConfigMarker::Vite
    } else {
        ConfigMarker::TypeScript
    })
}

pub(super) fn is_config_marker(path: &Path) -> bool {
    path.file_name().and_then(config_marker_name).is_some()
}

#[cfg(test)]
mod tests;
