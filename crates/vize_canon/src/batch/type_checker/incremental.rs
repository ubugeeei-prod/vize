//! Persistent project refresh orchestration.

use std::path::PathBuf;

use vize_carton::FxHashSet;

use super::{BatchTypeChecker, CorsaResult, TypeCheckResult};

impl BatchTypeChecker {
    pub(super) fn check_incremental_snapshot(
        &mut self,
        changed: &[PathBuf],
    ) -> CorsaResult<TypeCheckResult> {
        let effective_changes = self
            .incremental_paths
            .effective_changes(self.project.project_root(), changed);
        if effective_changes.is_empty() {
            return self.check_registered_project_incremental();
        }
        if self
            .project
            .configuration_inputs_changed(&effective_changes)
        {
            return self.refresh_configuration_snapshot();
        }

        let old_route_sources = self.project.package_route_source_paths();
        let old_route_source_paths = old_route_sources.iter().cloned().collect::<Vec<_>>();
        let roots = self.incremental_paths.refresh(
            &self.project,
            self.project.source_file_policy(),
            changed,
            &old_route_source_paths,
        )?;
        self.project.set_declaration_roots(&roots);

        let changed_paths = effective_changes
            .iter()
            .map(|path| {
                let absolute = if path.is_absolute() {
                    path.clone()
                } else {
                    self.project.project_root().join(path)
                };
                crate::package_route::stamp::canonicalize_changed_path(&absolute)
            })
            .collect::<FxHashSet<_>>();

        // Source edits must land before importer route reconciliation so the
        // occurrence kind and spelling come from the new authored bytes.
        let mut rebuilt_sources = Vec::new();
        for path in &changed_paths {
            let known_source = self.project.find_by_original(path).is_some()
                || roots.contains(path)
                || old_route_sources.contains(path);
            if !known_source {
                continue;
            }
            if path.is_file() {
                self.project.register_path(path)?;
                rebuilt_sources.push(path.clone());
            } else {
                self.project.remove_source_and_dependencies(path);
            }
        }

        let mut affected = self
            .project
            .package_route_keys_for_changes(&effective_changes);
        // Changed importers are replaced as a set below. Re-resolving their old
        // keys first would perform duplicate work and could briefly retain a
        // route whose specifier or occurrence mode was removed.
        affected.retain(|key| !changed_paths.contains(&key.importer_path));
        self.project.refresh_package_route_keys(affected);
        self.project
            .reconcile_package_routes_for_importers(&effective_changes);

        let new_route_sources = self.project.package_route_source_paths();
        let mut added_route_sources = new_route_sources
            .difference(&old_route_sources)
            .cloned()
            .collect::<Vec<_>>();
        added_route_sources.sort();
        self.project.register_paths(&added_route_sources)?;
        rebuilt_sources.extend(added_route_sources);
        rebuilt_sources.sort();
        rebuilt_sources.dedup();
        self.project
            .register_reachable_dependencies_from(&rebuilt_sources)?;

        let released = old_route_sources
            .difference(&new_route_sources)
            .cloned()
            .collect::<Vec<_>>();
        self.project.prune_unowned_sources(released);
        self.project.finalize_package_routes()?;

        let result = self.check_registered_project_incremental();
        if result.is_ok() {
            self.incremental_paths
                .commit_project_snapshot(&self.project);
        }
        result
    }

    /// Compiler configuration can change source membership, alias edges, and
    /// package-resolution context at once.  Reconcile that graph from its
    /// caller-owned roots while keeping the same project/session authority;
    /// attempting to patch only the tsconfig file would retain stale aliases,
    /// route keys, and dependency ownership.
    fn refresh_configuration_snapshot(&mut self) -> CorsaResult<TypeCheckResult> {
        let previous_sources = self.project.registered_original_paths_sorted();
        self.project.refresh_compiler_configuration();
        let roots = self
            .incremental_paths
            .refresh_for_configuration(&self.project, self.project.source_file_policy())?;
        self.project.set_declaration_roots(&roots);

        self.project.reset_dependency_ownership();
        self.project.clear_package_route_bindings_for_rebuild();
        self.project.prune_unowned_sources(previous_sources);
        self.project.register_paths(&roots)?;

        // Newly registered relative/alias dependencies can themselves import a
        // bare package.  Process each importer exactly once until the reachable
        // authored graph reaches a natural fixpoint; there is no arbitrary depth
        // cap and no whole-workspace scan beyond this explicit config event.
        let mut processed = FxHashSet::default();
        loop {
            let mut importers = self
                .project
                .registered_original_paths_sorted()
                .into_iter()
                .filter(|path| !processed.contains(path))
                .collect::<Vec<_>>();
            if importers.is_empty() {
                break;
            }
            importers.sort();
            processed.extend(importers.iter().cloned());
            self.project
                .reconcile_package_routes_for_importers(&importers);

            let mut route_sources = self
                .project
                .package_route_source_paths()
                .into_iter()
                .filter(|path| self.project.find_by_original(path).is_none())
                .collect::<Vec<_>>();
            route_sources.sort();
            self.project.register_paths(&route_sources)?;
            importers.extend(route_sources);
            importers.sort();
            importers.dedup();
            self.project
                .register_reachable_dependencies_from(&importers)?;
        }
        self.project.finalize_package_routes()?;

        let result = self.check_registered_project_incremental();
        if result.is_ok() {
            self.incremental_paths
                .commit_project_snapshot(&self.project);
        }
        result
    }
}
