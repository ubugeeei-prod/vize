//! Authoritative package-route indexes and bounded refresh.

use std::path::{Path, PathBuf};

use vize_carton::FxHashSet;

use crate::batch::error::CorsaResult;

use super::super::VirtualProject;

impl VirtualProject {
    pub(crate) fn clear_package_route_bindings_for_rebuild(&mut self) {
        let keys = self.package_routes.keys().cloned().collect::<Vec<_>>();
        for key in keys {
            self.remove_package_route_binding(&key);
        }
        self.package_route_refresh_keys.clear();
        self.package_routes_need_refresh = false;
    }

    pub(crate) fn replace_package_route_bindings(
        &mut self,
        bindings: impl IntoIterator<Item = crate::PackageRouteBinding>,
    ) {
        let previous_roots = self
            .package_route_roots
            .keys()
            .cloned()
            .collect::<FxHashSet<_>>();
        self.package_routes.clear();
        self.package_route_reverse.clear();
        self.package_route_importers.clear();
        self.package_route_source_owners.clear();
        self.package_route_manifests.clear();
        self.package_route_roots.clear();
        self.package_shadow_dirty_keys.clear();
        for binding in bindings {
            self.insert_package_route_binding(binding);
        }
        let current_roots = self
            .package_route_roots
            .keys()
            .cloned()
            .collect::<FxHashSet<_>>();
        self.package_source_index
            .retain(|root, _| current_roots.contains(root));
        let added_roots = current_roots
            .difference(&previous_roots)
            .cloned()
            .collect::<Vec<_>>();
        if !added_roots.is_empty() {
            let registered = self
                .original_index
                .iter()
                .map(|(source, virtual_path)| (source.clone(), virtual_path.clone()))
                .collect::<Vec<_>>();
            for (source, virtual_path) in registered {
                for root in &added_roots {
                    let Ok(relative) = source.strip_prefix(root) else {
                        continue;
                    };
                    self.package_source_index
                        .entry(root.clone())
                        .or_default()
                        .insert(
                            source.clone(),
                            (relative.to_path_buf(), virtual_path.clone()),
                        );
                }
            }
        }
    }

    pub(crate) fn insert_package_route_binding(&mut self, binding: crate::PackageRouteBinding) {
        let key = binding.key();
        self.remove_package_route_binding(&key);
        for path in &binding.invalidation_paths {
            self.package_route_reverse
                .entry(path.clone())
                .or_default()
                .insert(key.clone());
        }
        self.package_route_importers
            .entry(binding.importer_path.clone())
            .or_default()
            .insert(key.clone());
        if let Some(route) = binding.route.as_ref() {
            for route in route.all_routes() {
                let package_root =
                    vize_carton::path::canonicalize_non_verbatim(&route.package_root);
                self.package_route_roots
                    .entry(package_root)
                    .or_default()
                    .insert(key.clone());
                if let Some(keys) = self.package_route_manifests.get(&route.manifest_path) {
                    self.package_shadow_dirty_keys.extend(keys.iter().cloned());
                }
                self.package_route_manifests
                    .entry(route.manifest_path.clone())
                    .or_default()
                    .insert(key.clone());
            }
            for source in route.all_source_paths() {
                let source = vize_carton::path::canonicalize_non_verbatim(source);
                *self.package_route_source_owners.entry(source).or_default() += 1;
            }
            self.index_existing_package_route_sources(route);
        }
        self.package_shadow_dirty_keys.insert(key.clone());
        self.package_routes.insert(key, binding);
    }

    pub(crate) fn remove_package_route_binding(
        &mut self,
        key: &crate::package_route::PackageRouteKey,
    ) -> Option<crate::PackageRouteBinding> {
        let binding = self.package_routes.remove(key)?;
        self.package_shadow_dirty_keys.insert(key.clone());
        self.remove_route_reverse_indexes(key, &binding);
        self.remove_route_topology_indexes(key, &binding);
        Some(binding)
    }

    fn remove_route_reverse_indexes(
        &mut self,
        key: &crate::package_route::PackageRouteKey,
        binding: &crate::PackageRouteBinding,
    ) {
        for path in &binding.invalidation_paths {
            let remove = self
                .package_route_reverse
                .get_mut(path)
                .is_some_and(|keys| {
                    keys.remove(key);
                    keys.is_empty()
                });
            if remove {
                self.package_route_reverse.remove(path);
            }
        }
        let remove = self
            .package_route_importers
            .get_mut(&binding.importer_path)
            .is_some_and(|keys| {
                keys.remove(key);
                keys.is_empty()
            });
        if remove {
            self.package_route_importers.remove(&binding.importer_path);
        }
    }

    fn remove_route_topology_indexes(
        &mut self,
        key: &crate::package_route::PackageRouteKey,
        binding: &crate::PackageRouteBinding,
    ) {
        let Some(route) = binding.route.as_ref() else {
            return;
        };
        for route in route.all_routes() {
            let root = vize_carton::path::canonicalize_non_verbatim(&route.package_root);
            let remove = self.package_route_roots.get_mut(&root).is_some_and(|keys| {
                keys.remove(key);
                keys.is_empty()
            });
            if remove {
                self.package_route_roots.remove(&root);
                self.package_source_index.remove(&root);
            }
            if let Some(keys) = self.package_route_manifests.get(&route.manifest_path) {
                self.package_shadow_dirty_keys.extend(keys.iter().cloned());
            }
            let remove = self
                .package_route_manifests
                .get_mut(&route.manifest_path)
                .is_some_and(|keys| {
                    keys.remove(key);
                    keys.is_empty()
                });
            if remove {
                self.package_route_manifests.remove(&route.manifest_path);
            }
        }
        for source in route.all_source_paths() {
            let source = vize_carton::path::canonicalize_non_verbatim(source);
            let remove = self
                .package_route_source_owners
                .get_mut(&source)
                .is_some_and(|count| {
                    *count = count.saturating_sub(1);
                    *count == 0
                });
            if remove {
                self.package_route_source_owners.remove(&source);
            }
        }
    }

    pub(crate) fn package_route_keys_for_changes(
        &self,
        changed: &[PathBuf],
    ) -> FxHashSet<crate::package_route::PackageRouteKey> {
        let mut affected = FxHashSet::default();
        for path in changed {
            let logical = if path.is_absolute() {
                path.clone()
            } else {
                self.project_root.join(path)
            };
            if let Some(keys) = self.package_route_reverse.get(&logical) {
                affected.extend(keys.iter().cloned());
            }
            let canonical = crate::package_route::stamp::canonicalize_changed_path(&logical);
            if canonical != logical
                && let Some(keys) = self.package_route_reverse.get(&canonical)
            {
                affected.extend(keys.iter().cloned());
            }
            if let Some(keys) = self.package_route_importers.get(&canonical) {
                affected.extend(keys.iter().cloned());
            }
        }
        affected
    }

    pub(crate) fn register_package_route_targets(&mut self) -> CorsaResult<()> {
        if self.package_routes_need_refresh {
            self.refresh_package_routes();
            self.package_routes_need_refresh = false;
        }
        let mut targets = self
            .package_route_source_paths()
            .into_iter()
            .collect::<Vec<_>>();
        targets.sort();
        self.register_paths(&targets)
    }

    pub(crate) fn refresh_package_route_keys(
        &mut self,
        keys: FxHashSet<crate::package_route::PackageRouteKey>,
    ) {
        self.package_route_refresh_keys = keys;
        self.package_routes_need_refresh = !self.package_route_refresh_keys.is_empty();
        if self.package_routes_need_refresh {
            self.refresh_package_routes();
            self.package_routes_need_refresh = false;
        }
    }

    fn refresh_package_routes(&mut self) {
        let resolution = self.package_resolution_settings();
        let mut resolver = match self.package_route_resolver.lock() {
            Ok(resolver) => resolver,
            Err(_) => return,
        };
        let total = self.package_routes.len();
        let refresh_keys = std::mem::take(&mut self.package_route_refresh_keys);
        let mut refreshed = Vec::with_capacity(refresh_keys.len());
        for key in &refresh_keys {
            let Some(mut binding) = self.package_routes.get(key).cloned() else {
                continue;
            };
            let importer_dir = binding
                .importer_path
                .parent()
                .unwrap_or(binding.importer_path.as_path());
            let (context, context_inputs) = resolution.context(
                &mut resolver,
                &binding.importer_path,
                binding.occurrence_mode,
            );
            let lookup = resolver.lookup_with_context(
                importer_dir,
                &binding.specifier,
                crate::PackageSourceOptions::new(
                    self.source_policy.allows_javascript(),
                    self.jsx_typecheck,
                ),
                context.clone(),
            );
            let (route, mut invalidation_paths) = lookup.into_parts();
            invalidation_paths.extend(resolution.input_paths().iter().cloned());
            invalidation_paths.extend(context_inputs);
            invalidation_paths.push(binding.importer_path.clone());
            binding.context = context;
            if route.is_none() {
                invalidation_paths.extend(binding.invalidation_paths.iter().cloned());
            }
            binding.route = route;
            invalidation_paths.sort();
            invalidation_paths.dedup();
            binding.invalidation_paths = invalidation_paths;
            refreshed.push(binding);
        }
        let considered = refreshed.len();
        resolver.record_refresh_scope(total, considered, considered);
        drop(resolver);
        for binding in refreshed {
            self.insert_package_route_binding(binding);
        }
    }

    pub(crate) fn package_route_metrics(&self) -> crate::PackageRouteMetrics {
        self.package_route_resolver
            .lock()
            .map(|resolver| resolver.metrics())
            .unwrap_or_default()
    }

    #[cfg(test)]
    pub(crate) fn package_routes_snapshot(&self) -> Vec<crate::PackageRouteBinding> {
        let mut bindings = self.package_routes.values().cloned().collect::<Vec<_>>();
        bindings.sort_by(|left, right| left.key().cmp(&right.key()));
        bindings
    }

    pub(crate) fn workspace_package_route_identity(
        &self,
        importer: &Path,
        specifier: &str,
    ) -> Option<&Path> {
        self.package_routes
            .values()
            .find(|binding| {
                binding.importer_path == importer
                    && binding.specifier == specifier
                    && binding
                        .route
                        .as_ref()
                        .is_some_and(|route| route.workspace_source)
            })
            .and_then(|binding| binding.route.as_ref())
            .map(|route| route.package_root.as_path())
    }

    pub(crate) fn finalize_package_routes(&mut self) -> CorsaResult<()> {
        self.rebuild_package_shadows()
    }
}
