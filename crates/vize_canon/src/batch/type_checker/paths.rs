//! Source discovery and refresh scope for batch type checking.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use vize_carton::FxHashMap;

use super::super::error::{CorsaError, CorsaResult};
use super::super::source_policy::SourceFilePolicy;
use super::super::virtual_project::VirtualProject;

/// Source membership carried across incremental checks.
///
/// The initial virtual project stays immutable so full checks preserve their
/// original scope. This state evolves separately as project-wide scans observe
/// creates and all scans observe deletes. Explicit `scan_paths` scopes never
/// grow from an out-of-scope change notification.
pub(super) struct IncrementalPaths {
    allow_new_paths: bool,
    state: Mutex<IncrementalPathState>,
}

#[derive(Default)]
struct IncrementalPathState {
    roots: Vec<PathBuf>,
    known_source_paths: Vec<PathBuf>,
    source_stamps: FxHashMap<PathBuf, crate::package_route::stamp::InputStamp>,
}

impl IncrementalPaths {
    pub(super) fn new() -> Self {
        Self {
            allow_new_paths: false,
            state: Mutex::new(IncrementalPathState::default()),
        }
    }

    pub(super) fn after_explicit_scan(&mut self, project: &VirtualProject, paths: &[PathBuf]) {
        self.allow_new_paths = false;
        self.replace_snapshot(project, paths);
    }

    pub(super) fn after_project_scan(&mut self, project: &VirtualProject, roots: &[PathBuf]) {
        self.allow_new_paths = true;
        self.replace_snapshot(project, roots);
    }

    fn replace_snapshot(&mut self, project: &VirtualProject, roots: &[PathBuf]) {
        let state = self
            .state
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.roots = roots
            .iter()
            .filter(|path| path.is_file())
            .map(|path| vize_carton::path::canonicalize_non_verbatim(path))
            .collect();
        state.roots.sort();
        state.roots.dedup();
        state.known_source_paths = project.registered_original_paths_sorted();
        state.source_stamps = stamp_project_inputs(project, &state.known_source_paths);
    }

    pub(super) fn effective_changes(
        &self,
        project_root: &Path,
        changed: &[PathBuf],
    ) -> Vec<PathBuf> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        changed
            .iter()
            .filter(|path| {
                let logical = if path.is_absolute() {
                    (*path).clone()
                } else {
                    project_root.join(path)
                };
                let canonical = crate::package_route::stamp::canonicalize_changed_path(&logical);
                state
                    .source_stamps
                    .get(&logical)
                    .or_else(|| state.source_stamps.get(&canonical))
                    .is_none_or(|stamp| !stamp.is_current())
            })
            .cloned()
            .collect()
    }

    pub(super) fn refresh(
        &self,
        project: &VirtualProject,
        source_policy: SourceFilePolicy,
        changed: &[PathBuf],
        package_source_paths: &[PathBuf],
    ) -> CorsaResult<Vec<PathBuf>> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut known_source_paths = project.known_source_paths();
        known_source_paths.extend(state.known_source_paths.iter().cloned());
        known_source_paths.extend_from_slice(package_source_paths);
        known_source_paths.sort();
        known_source_paths.dedup();
        refresh_paths(
            project.project_root(),
            &mut state.roots,
            changed,
            self.allow_new_paths,
            source_policy,
            &known_source_paths,
        )?;
        Ok(state.roots.clone())
    }

    pub(super) fn refresh_for_configuration(
        &self,
        project: &VirtualProject,
        source_policy: SourceFilePolicy,
    ) -> CorsaResult<Vec<PathBuf>> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if self.allow_new_paths {
            state.roots = collect_project_paths(project.project_root(), source_policy)?;
        } else {
            state.roots.retain(|path| path.is_file());
        }
        state.roots = state
            .roots
            .iter()
            .map(|path| vize_carton::path::canonicalize_non_verbatim(path))
            .collect();
        state.roots.sort();
        state.roots.dedup();
        Ok(state.roots.clone())
    }

    /// Commit the complete successful graph, not just the paths that existed
    /// when the persistent checker started. Roots stay caller-owned; this
    /// closure is used only to validate later external watcher notifications.
    pub(super) fn commit_project_snapshot(&self, project: &VirtualProject) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.known_source_paths = project.registered_original_paths_sorted();
        state.source_stamps = stamp_project_inputs(project, &state.known_source_paths);
    }
}

fn stamp_project_inputs(
    project: &VirtualProject,
    paths: &[PathBuf],
) -> FxHashMap<PathBuf, crate::package_route::stamp::InputStamp> {
    paths
        .iter()
        .cloned()
        .chain(project.governing_config_paths())
        .map(|path| {
            let stamp = crate::package_route::stamp::InputStamp::capture(path.clone());
            (path, stamp)
        })
        .collect()
}

pub(super) fn collect_project_paths(
    project_root: &Path,
    source_policy: SourceFilePolicy,
) -> CorsaResult<Vec<PathBuf>> {
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
        if path.is_file() && source_policy.accepts_project_source(path) {
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
    source_policy: SourceFilePolicy,
    known_source_paths: &[PathBuf],
) -> CorsaResult<()> {
    let mut refreshed_paths = paths.clone();
    for changed_path in changed {
        let logical_candidate = if changed_path.is_absolute() {
            changed_path.clone()
        } else {
            project_root.join(changed_path)
        };
        if !logical_candidate.exists() {
            continue;
        }

        let logical_in_project = logical_candidate.starts_with(project_root);
        let candidate = vize_carton::path::canonicalize_non_verbatim(&logical_candidate);
        let already_registered = refreshed_paths.iter().any(|path| path == &candidate);
        let known_source = known_source_paths
            .iter()
            .any(|path| path == &candidate || candidate.starts_with(path));
        if !logical_in_project
            && !candidate.starts_with(project_root)
            && !already_registered
            && !known_source
        {
            return Err(CorsaError::PathError { path: candidate });
        }
        if allow_new_paths
            && !already_registered
            && candidate.is_file()
            && source_policy.accepts_project_source(&candidate)
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

#[cfg(test)]
mod tests {
    use super::refresh_paths;
    use crate::batch::source_policy::SourceFilePolicy;

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
        let error = refresh_paths(
            &project_root,
            &mut paths,
            &[added, outside_file],
            true,
            SourceFilePolicy::default(),
            &[],
        )
        .expect_err("outside path should reject the whole refresh");

        assert!(matches!(error, crate::batch::CorsaError::PathError { .. }));
        assert_eq!(paths, vec![existing]);
    }

    #[test]
    fn a_registered_out_of_root_dependency_can_refresh() {
        let project = tempfile::tempdir().expect("project tempdir should exist");
        let outside = tempfile::tempdir().expect("outside tempdir should exist");
        let inside_file = project.path().join("inside.ts");
        let outside_file = outside.path().join("Workspace.vue");
        std::fs::write(&inside_file, "export {}\n").expect("inside file should write");
        std::fs::write(&outside_file, "<template />\n").expect("outside file should write");

        let project_root = vize_carton::path::canonicalize_non_verbatim(project.path());
        let inside_file = vize_carton::path::canonicalize_non_verbatim(&inside_file);
        let outside_file = vize_carton::path::canonicalize_non_verbatim(&outside_file);
        let mut paths = vec![inside_file.clone(), outside_file.clone()];

        refresh_paths(
            &project_root,
            &mut paths,
            std::slice::from_ref(&outside_file),
            true,
            SourceFilePolicy::default(),
            std::slice::from_ref(&outside_file),
        )
        .expect("an already-registered external dependency should refresh");
        paths.sort();
        let mut expected = vec![inside_file, outside_file];
        expected.sort();
        assert_eq!(paths, expected);
    }
}
