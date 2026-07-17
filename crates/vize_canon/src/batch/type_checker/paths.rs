//! Source discovery and refresh scope for batch type checking.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

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

    pub(super) fn after_explicit_scan(&mut self, paths: &[PathBuf]) {
        let was_project_wide = std::mem::replace(&mut self.allow_new_paths, false);
        let current = self
            .paths
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if was_project_wide {
            current.clear();
        }
        current.extend(
            paths
                .iter()
                .filter(|path| path.is_file())
                .map(|path| vize_carton::path::canonicalize_non_verbatim(path)),
        );
        current.sort();
        current.dedup();
    }

    pub(super) fn after_project_scan(&mut self, project: &VirtualProject) {
        self.allow_new_paths = true;
        self.replace_paths(project);
    }

    pub(super) fn refresh(
        &self,
        project: &VirtualProject,
        changed: &[PathBuf],
    ) -> CorsaResult<Vec<PathBuf>> {
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
        Ok(paths.clone())
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
    let mut refreshed_paths = paths.clone();
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
            && !refreshed_paths.iter().any(|path| path == &candidate)
            && candidate.is_file()
            && is_supported_input(&candidate)
        {
            refreshed_paths.push(candidate);
        }
    }

    refreshed_paths.retain(|path| path.is_file());
    refreshed_paths.sort();
    refreshed_paths.dedup();
    *paths = refreshed_paths;
    Ok(())
}

fn is_supported_input(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension, "vue" | "ts" | "tsx" | "mts" | "cts"))
        || is_declaration_file(path)
}

#[cfg(test)]
mod tests {
    use super::refresh_paths;

    #[test]
    fn validation_error_does_not_partially_grow_membership() {
        let project = tempfile::tempdir().expect("project tempdir should exist");
        let outside = tempfile::tempdir().expect("outside tempdir should exist");
        let existing = project.path().join("existing.ts");
        let added = project.path().join("added.ts");
        let outside_file = outside.path().join("outside.ts");
        std::fs::write(&existing, "export {}\n").expect("existing file should write");
        std::fs::write(&added, "export {}\n").expect("added file should write");
        std::fs::write(&outside_file, "export {}\n").expect("outside file should write");

        let project_root = vize_carton::path::canonicalize_non_verbatim(project.path());
        let existing = vize_carton::path::canonicalize_non_verbatim(&existing);
        let mut paths = vec![existing.clone()];
        let error = refresh_paths(&project_root, &mut paths, &[added, outside_file], true)
            .expect_err("outside path should reject the whole refresh");

        assert!(matches!(error, crate::batch::CorsaError::PathError { .. }));
        assert_eq!(paths, vec![existing]);
    }
}
