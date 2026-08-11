//! Incremental authored-source ownership.
//!
//! Roots, package-route targets, and resolved dependency edges are separate
//! owners.  Removing one importer therefore prunes only the now-unreachable
//! external/package closure instead of retaining stale files or rescanning the
//! whole project to rediscover reachability.

use std::path::{Path, PathBuf};

use vize_carton::FxHashSet;

use super::VirtualProject;

impl VirtualProject {
    pub(crate) fn reset_dependency_ownership(&mut self) {
        self.dependency_edges.clear();
        self.dependency_inbound.clear();
    }

    pub(crate) fn replace_dependency_edges(
        &mut self,
        importer: &Path,
        targets: FxHashSet<PathBuf>,
    ) -> Vec<PathBuf> {
        self.incremental_dependency_nodes_reconciled += 1;
        let importer = vize_carton::path::canonicalize_non_verbatim(importer);
        let previous = self
            .dependency_edges
            .insert(importer, targets.clone())
            .unwrap_or_default();
        let mut released = Vec::new();
        for target in previous.difference(&targets) {
            let remove = self
                .dependency_inbound
                .get_mut(target)
                .is_some_and(|count| {
                    *count = count.saturating_sub(1);
                    *count == 0
                });
            if remove {
                self.dependency_inbound.remove(target);
                released.push(target.clone());
            }
        }
        for target in targets.difference(&previous) {
            *self.dependency_inbound.entry(target.clone()).or_default() += 1;
        }
        released
    }

    pub(crate) fn package_route_source_paths(&self) -> FxHashSet<PathBuf> {
        self.package_route_source_owners.keys().cloned().collect()
    }

    pub(crate) fn prune_unowned_sources(
        &mut self,
        candidates: impl IntoIterator<Item = PathBuf>,
    ) -> Vec<PathBuf> {
        let mut queue = candidates.into_iter().collect::<Vec<_>>();
        let mut visited = FxHashSet::default();
        let mut removed_artifacts = Vec::new();
        while let Some(path) = queue.pop() {
            let path = vize_carton::path::canonicalize_non_verbatim(&path);
            if !visited.insert(path.clone()) || self.source_is_owned(&path) {
                continue;
            }
            let released = self.replace_dependency_edges(&path, FxHashSet::default());
            self.dependency_edges.remove(&path);
            let removed = self.remove_registered_source(&path);
            if !removed.is_empty() {
                self.mark_incremental_config_file();
            }
            removed_artifacts.extend(removed);
            queue.extend(released);
        }
        removed_artifacts
    }

    pub(crate) fn remove_source_and_dependencies(&mut self, path: &Path) -> Vec<PathBuf> {
        let path = vize_carton::path::canonicalize_non_verbatim(path);
        let released = self.replace_dependency_edges(&path, FxHashSet::default());
        self.dependency_edges.remove(&path);
        let mut removed = self.remove_registered_source(&path);
        if !removed.is_empty() {
            self.mark_incremental_config_file();
        }
        removed.extend(self.prune_unowned_sources(released));
        removed
    }

    fn source_is_owned(&self, path: &Path) -> bool {
        self.declaration_roots
            .as_ref()
            .is_some_and(|roots| roots.contains(path))
            || self
                .package_route_source_owners
                .get(path)
                .is_some_and(|count| *count > 0)
            || self
                .dependency_inbound
                .get(path)
                .is_some_and(|count| *count > 0)
    }
}
