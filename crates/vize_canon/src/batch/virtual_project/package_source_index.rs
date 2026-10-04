//! Physical package-root -> generated source topology index.

use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, FxHashSet};

use crate::package_route::PackageRouteKey;

use super::VirtualProject;

type PackageSources = FxHashMap<PathBuf, (PathBuf, PathBuf)>;

impl VirtualProject {
    pub(crate) fn insert_package_route_binding(&mut self, binding: crate::PackageRouteBinding) {
        let retained = self.take_rebound_package_sources(&FxHashSet::from_iter([binding.key()]));
        self.insert_route_binding_inner(binding);
        self.restore_rebound_package_sources(retained);
    }

    pub(crate) fn reconcile_package_routes_for_importers(&mut self, changed: &[PathBuf]) {
        let changed = changed
            .iter()
            .map(|path| {
                let absolute = if path.is_absolute() {
                    path.clone()
                } else {
                    self.project_root.join(path)
                };
                vize_carton::path::canonicalize_non_verbatim(&absolute)
            })
            .collect::<FxHashSet<_>>();
        let keys = changed
            .iter()
            .flat_map(|importer| {
                self.package_route_importers
                    .get(importer)
                    .into_iter()
                    .flatten()
            })
            .cloned()
            .collect();
        let retained = self.take_rebound_package_sources(&keys);
        self.reconcile_package_routes_for_importers_inner(&changed);
        self.restore_rebound_package_sources(retained);
    }

    // Route-owned entry points are only a subset of a package's registered
    // relative sources. Keep that bounded topology across an atomic rebind
    // when removing its final owner would otherwise discard the whole index.
    fn take_rebound_package_sources(
        &mut self,
        keys: &FxHashSet<PackageRouteKey>,
    ) -> FxHashMap<PathBuf, PackageSources> {
        let roots = keys
            .iter()
            .filter_map(|key| self.package_routes.get(key)?.route.as_ref())
            .flat_map(crate::PackageRoute::all_routes)
            .map(|route| vize_carton::path::canonicalize_non_verbatim(&route.package_root))
            .filter(|root| {
                self.package_route_roots
                    .get(root)
                    .is_some_and(|owners| owners.iter().all(|owner| keys.contains(owner)))
            })
            .collect::<FxHashSet<_>>();
        roots
            .into_iter()
            .filter_map(|root| {
                self.package_source_index
                    .remove(&root)
                    .map(|sources| (root, sources))
            })
            .collect()
    }

    fn restore_rebound_package_sources(&mut self, retained: FxHashMap<PathBuf, PackageSources>) {
        for (root, sources) in retained {
            let Some(owners) = self.package_route_roots.get(&root) else {
                continue;
            };
            let live_sources = sources.into_iter().filter(|(source, (_, virtual_path))| {
                self.original_index.get(source) == Some(virtual_path)
                    && self.virtual_files.contains_key(virtual_path)
            });
            self.package_source_index
                .entry(root)
                .or_default()
                .extend(live_sources);
            self.package_shadow_dirty_keys
                .extend(owners.iter().cloned());
        }
    }

    pub(super) fn index_package_source(&mut self, source: &Path, virtual_path: &Path) {
        let source = vize_carton::path::canonicalize_non_verbatim(source);
        let roots = source
            .ancestors()
            .filter(|ancestor| self.package_route_roots.contains_key(*ancestor))
            .map(Path::to_path_buf)
            .collect::<Vec<_>>();
        for root in roots {
            let Ok(relative) = source.strip_prefix(&root) else {
                continue;
            };
            let indexed = (relative.to_path_buf(), virtual_path.to_path_buf());
            let changed = self
                .package_source_index
                .entry(root.clone())
                .or_default()
                .insert(source.clone(), indexed.clone())
                .as_ref()
                != Some(&indexed);
            if changed
                && self.package_shadows_initialized
                && let Some(keys) = self.package_route_roots.get(&root)
            {
                self.package_shadow_dirty_keys.extend(keys.iter().cloned());
            }
        }
    }

    pub(super) fn remove_package_source_from_index(&mut self, source: &Path) {
        let source = vize_carton::path::canonicalize_non_verbatim(source);
        for ancestor in source.ancestors() {
            if self
                .package_source_index
                .get_mut(ancestor)
                .is_some_and(|sources| sources.remove(&source).is_some())
                && self.package_shadows_initialized
                && let Some(keys) = self.package_route_roots.get(ancestor)
            {
                self.package_shadow_dirty_keys.extend(keys.iter().cloned());
            }
        }
    }

    pub(super) fn index_existing_package_route_sources(&mut self, route: &crate::PackageRoute) {
        for route in route.all_routes() {
            let root = vize_carton::path::canonicalize_non_verbatim(&route.package_root);
            self.package_route_roots.entry(root.clone()).or_default();
            for source in route.all_source_paths() {
                let source = vize_carton::path::canonicalize_non_verbatim(source);
                let Some(virtual_path) = self.original_index.get(&source).cloned() else {
                    continue;
                };
                let Ok(relative) = source.strip_prefix(&root) else {
                    continue;
                };
                let relative = relative.to_path_buf();
                self.package_source_index
                    .entry(root.clone())
                    .or_default()
                    .insert(source, (relative, virtual_path));
            }
        }
    }
}
