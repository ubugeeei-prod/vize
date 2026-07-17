//! Source discovery and refresh scope for batch type checking.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use super::super::declaration_path::is_declaration_file;
use super::super::error::{CorsaError, CorsaResult};
use super::super::virtual_project::VirtualProject;

/// Source membership carried across incremental checks.
///
/// The initial virtual project stays immutable so full checks preserve their
/// original scope. This state evolves separately as project-wide scans observe
/// creates and all scans observe deletes. Explicit `scan_paths` scopes never
/// grow from an out-of-scope change notification.
pub(super) struct IncrementalPaths {
    allow_new_paths: bool,
    paths: Mutex<Vec<PathBuf>>,
}

impl IncrementalPaths {
    pub(super) fn new() -> Self {
        Self {
            allow_new_paths: false,
            paths: Mutex::new(Vec::new()),
        }
    }

    pub(super) fn after_explicit_scan(&mut self, project: &VirtualProject) {
        self.replace_paths(project);
    }

    pub(super) fn after_project_scan(&mut self, project: &VirtualProject) {
        self.allow_new_paths = true;
        self.replace_paths(project);
    }

    pub(super) fn refresh<'a>(
        &'a self,
        project: &VirtualProject,
        changed: &[PathBuf],
    ) -> CorsaResult<MutexGuard<'a, Vec<PathBuf>>> {
        let mut paths = self
            .paths
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        refresh_paths(
            project.project_root(),
            &mut paths,
            changed,
            self.allow_new_paths,
        )?;
        Ok(paths)
    }

    fn replace_paths(&mut self, project: &VirtualProject) {
        *self
            .paths
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            project.registered_original_paths_sorted();
    }
}

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

pub(super) fn refresh_paths(
    project_root: &Path,
    paths: &mut Vec<PathBuf>,
    changed: &[PathBuf],
    allow_new_paths: bool,
) -> CorsaResult<()> {
    for changed_path in changed {
        let candidate = if changed_path.is_absolute() {
            changed_path.clone()
        } else {
            project_root.join(changed_path)
        };
        if !candidate.exists() {
            continue;
        }

        let candidate = vize_carton::path::canonicalize_non_verbatim(&candidate);
        if !candidate.starts_with(project_root) {
            return Err(CorsaError::PathError { path: candidate });
        }
        if allow_new_paths
            && !paths.iter().any(|path| path == &candidate)
            && candidate.is_file()
            && is_supported_input(&candidate)
        {
            paths.push(candidate);
        }
    }

    paths.retain(|path| path.is_file());
    paths.sort();
    paths.dedup();
    Ok(())
}

fn is_supported_input(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension, "vue" | "ts" | "tsx" | "mts" | "cts"))
        || is_declaration_file(path)
}
