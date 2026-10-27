//! The import graph the walk records, and the authored sources it reaches.

use std::path::{Path, PathBuf};

use vize_canon::PackageRouteBinding;
use vize_l0::{FxHashMap, FxHashSet};

use super::is_declaration_file;

#[derive(Default)]
pub(super) struct ImportReachabilityGraph {
    pub(super) edges: FxHashMap<PathBuf, FxHashSet<PathBuf>>,
    pub(super) direct_materialization: FxHashSet<PathBuf>,
}

impl ImportReachabilityGraph {
    pub(super) fn add_edge(&mut self, importer: &Path, dependency: &Path) {
        self.edges
            .entry(importer.to_path_buf())
            .or_default()
            .insert(dependency.to_path_buf());
    }

    pub(super) fn mark_direct_materialization(&mut self, path: &Path) {
        self.direct_materialization.insert(path.to_path_buf());
    }

    pub(super) fn reaches_materialized_source(&self) -> FxHashSet<PathBuf> {
        let mut reverse: FxHashMap<&Path, Vec<&Path>> = FxHashMap::default();
        let mut reached = self.direct_materialization.clone();
        let mut queue = self
            .direct_materialization
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        for (importer, dependencies) in &self.edges {
            if importer
                .extension()
                .is_some_and(|extension| extension == "vue")
                && reached.insert(importer.clone())
            {
                queue.push(importer.clone());
            }
            for dependency in dependencies {
                reverse.entry(dependency).or_default().push(importer);
                if dependency
                    .extension()
                    .is_some_and(|extension| extension == "vue")
                    && reached.insert(dependency.clone())
                {
                    queue.push(dependency.clone());
                }
            }
        }
        while let Some(path) = queue.pop() {
            if let Some(importers) = reverse.get(path.as_path()) {
                for importer in importers {
                    if reached.insert((*importer).to_path_buf()) {
                        queue.push((*importer).to_path_buf());
                    }
                }
            }
        }
        reached
    }

    pub(super) fn reachable_files(
        &self,
        roots: impl IntoIterator<Item = PathBuf>,
    ) -> FxHashSet<PathBuf> {
        let mut reached = FxHashSet::default();
        let mut queue = roots.into_iter().collect::<Vec<_>>();
        while let Some(path) = queue.pop() {
            if !reached.insert(path.clone()) {
                continue;
            }
            if let Some(dependencies) = self.edges.get(&path) {
                queue.extend(dependencies.iter().cloned());
            }
        }
        reached
    }

    /// Drop authored sources the roots reach only through declaration files.
    /// The walk enters package-graph declarations so workspace barrels can
    /// reach their SFCs (#10015), but an ambient re-export such as Nuxt's
    /// `.nuxt/imports.d.ts` never made its targets authored (#10889). Package
    /// routes own their source and dependency paths as before.
    pub(super) fn retain_declaration_free(
        &self,
        authored: &mut Vec<PathBuf>,
        roots: Vec<PathBuf>,
        bindings: &[PackageRouteBinding],
        workspace_owners: &[(PathBuf, PathBuf)],
    ) {
        let mut owned: FxHashMap<&Path, Vec<&Path>> = FxHashMap::default();
        for binding in bindings {
            if let Some(route) = &binding.route {
                owned
                    .entry(binding.importer_path.as_path())
                    .or_default()
                    .extend(route.all_source_paths().into_iter().map(PathBuf::as_path));
            }
        }
        for (importer, source) in workspace_owners {
            owned
                .entry(importer.as_path())
                .or_default()
                .push(source.as_path());
        }
        let mut reached = FxHashSet::default();
        let mut queue = roots;
        while let Some(path) = queue.pop() {
            if is_declaration_file(&path) || !reached.insert(path.clone()) {
                continue;
            }
            if let Some(dependencies) = self.edges.get(&path) {
                queue.extend(dependencies.iter().cloned());
            }
            if let Some(sources) = owned.get(path.as_path()) {
                queue.extend(sources.iter().map(|source| source.to_path_buf()));
            }
        }
        authored.retain(|path| reached.contains(path));
    }
}
