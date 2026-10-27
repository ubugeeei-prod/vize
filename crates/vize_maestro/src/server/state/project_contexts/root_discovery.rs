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
            vize_l0::micro_count!("fs.directory_scan.calls", 1);
            let markers = directory_markers(directory);
            if markers.typescript {
                if package_root.is_some()
                    && let Some(vite_root) = vite_root.as_ref()
                {
                    return vite_root.clone();
                }
                return directory.to_path_buf();
            }
            if vite_root.is_none() && package_root.is_none() && markers.vite {
                vite_root = Some(directory.to_path_buf());
            }
            if package_root.is_none() && markers.package {
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

#[derive(Debug, Default, Eq, PartialEq)]
struct DirectoryMarkers {
    package: bool,
    vite: bool,
    typescript: bool,
}

impl DirectoryMarkers {
    // A TypeScript marker decides this directory's ownership independently of
    // its other markers. Normalize those irrelevant flags for either reader.
    fn observe(&mut self, name: &OsStr) -> bool {
        match config_marker_name(name) {
            Some(ConfigMarker::Package) => self.package = true,
            Some(ConfigMarker::Vite) => self.vite = true,
            Some(ConfigMarker::TypeScript) => {
                *self = Self {
                    typescript: true,
                    ..Self::default()
                };
                return true;
            }
            None => {}
        }
        false
    }
}

fn directory_markers(directory: &Path) -> DirectoryMarkers {
    #[cfg(target_os = "linux")]
    if let Ok(markers) = linux::directory_markers(directory) {
        return markers;
    }
    standard_directory_markers(directory)
}

fn standard_directory_markers(directory: &Path) -> DirectoryMarkers {
    vize_l0::micro_count!("fs.read_dir.calls", 1);
    let mut markers = DirectoryMarkers::default();
    if let Ok(entries) = std::fs::read_dir(directory) {
        for entry in entries.flatten() {
            vize_l0::micro_count!("fs.read_dir.entries", 1);
            if markers.observe(&entry.file_name()) {
                break;
            }
        }
    }
    markers
}

#[cfg(target_os = "linux")]
mod linux;

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
