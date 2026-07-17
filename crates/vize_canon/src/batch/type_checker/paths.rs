//! Source discovery and refresh scope for batch type checking.

use std::path::{Path, PathBuf};

use super::super::declaration_path::is_declaration_file;
use super::super::error::{CorsaError, CorsaResult};
use super::super::virtual_project::VirtualProject;

pub(super) fn collect_project_paths(project_root: &Path) -> CorsaResult<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for entry in walkdir::WalkDir::new(project_root)
        .into_iter()
        .filter_entry(|entry| {
            if entry.path() == project_root {
                return true;
            }
            let name = entry.file_name().to_string_lossy();
            !name.starts_with('.') && name != "node_modules"
        })
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && is_supported_input(path) {
            paths.push(path.to_path_buf());
        }
    }
    Ok(paths)
}

pub(super) fn refreshed_paths(
    project: &VirtualProject,
    changed: &[PathBuf],
) -> CorsaResult<Vec<PathBuf>> {
    let project_root = project.project_root();
    let mut paths = project.registered_original_paths_sorted();

    for changed_path in changed {
        let candidate = if changed_path.is_absolute() {
            changed_path.clone()
        } else {
            project_root.join(changed_path)
        };
        if paths.iter().any(|path| path == &candidate) || !candidate.exists() {
            continue;
        }

        let candidate = vize_carton::path::canonicalize_non_verbatim(&candidate);
        if !candidate.starts_with(project_root) {
            return Err(CorsaError::PathError { path: candidate });
        }
        if candidate.is_file() && is_supported_input(&candidate) {
            paths.push(candidate);
        }
    }

    paths.retain(|path| path.is_file());
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn is_supported_input(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension, "vue" | "ts" | "tsx" | "mts" | "cts"))
        || is_declaration_file(path)
}
