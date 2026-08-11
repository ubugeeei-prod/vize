//! Affected-scope ownership for materialized package dependency links.

use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, FxHashSet};

use crate::package_route::PackageRouteKey;

use super::VirtualProject;
use super::package_node_modules::merge_aware_node_modules_links;
use super::package_shadow::PackageShadowTopology;

pub(super) struct PackageLinkPatch {
    scope_links: FxHashMap<PathBuf, FxHashMap<PathBuf, PathBuf>>,
    pub(super) previous: FxHashMap<PathBuf, PathBuf>,
    pub(super) desired: FxHashMap<PathBuf, PathBuf>,
    pub(super) candidates: FxHashSet<PathBuf>,
}

impl VirtualProject {
    pub(super) fn track_materialized_link_path(&mut self, path: &Path) {
        let (scopes, targets) = self.package_link_scopes_for_path(path);
        for scope in scopes {
            if self
                .package_link_scope_files
                .entry(scope.clone())
                .or_default()
                .insert(path.to_path_buf())
            {
                self.incremental_package_link_scopes.insert(scope);
            }
        }
        for (scope, target) in targets {
            *self
                .package_link_scope_targets
                .entry(scope.clone())
                .or_default()
                .entry(target)
                .or_default() += 1;
            self.incremental_package_link_scopes.insert(scope);
        }
    }

    pub(super) fn untrack_materialized_link_path(&mut self, path: &Path) {
        let (scopes, targets) = self.package_link_scopes_for_path(path);
        for scope in scopes {
            let remove = self
                .package_link_scope_files
                .get_mut(&scope)
                .is_some_and(|files| {
                    files.remove(path);
                    files.is_empty()
                });
            if remove {
                self.package_link_scope_files.remove(&scope);
            }
            self.incremental_package_link_scopes.insert(scope);
        }
        for (scope, target) in targets {
            let remove_scope =
                self.package_link_scope_targets
                    .get_mut(&scope)
                    .is_some_and(|targets| {
                        let remove_target = targets.get_mut(&target).is_some_and(|count| {
                            *count = count.saturating_sub(1);
                            *count == 0
                        });
                        if remove_target {
                            targets.remove(&target);
                        }
                        targets.is_empty()
                    });
            if remove_scope {
                self.package_link_scope_targets.remove(&scope);
            }
            self.incremental_package_link_scopes.insert(scope);
        }
    }

    pub(super) fn install_package_shadow_link_scopes(
        &mut self,
        key: &PackageRouteKey,
        topology: &PackageShadowTopology,
    ) {
        let Some(route) = self
            .package_routes
            .get(key)
            .and_then(|binding| binding.route.as_ref())
        else {
            return;
        };
        let mut scopes = Vec::new();
        for nested in route.all_routes() {
            let real_dir = nested.package_root.join("node_modules");
            for (manifest, original) in &topology.manifests {
                if original != &nested.manifest_path {
                    continue;
                }
                let Some(shadow_root) = manifest.parent() else {
                    continue;
                };
                scopes.push((shadow_root.join("node_modules"), real_dir.clone()));
            }
        }
        scopes.sort();
        scopes.dedup();
        for (scope, target) in &scopes {
            self.add_package_link_target(scope, target);
        }
        self.package_shadow_link_scopes.insert(key.clone(), scopes);
    }

    pub(super) fn remove_package_shadow_link_scopes(&mut self, key: &PackageRouteKey) {
        let Some(scopes) = self.package_shadow_link_scopes.remove(key) else {
            return;
        };
        for (scope, target) in scopes {
            self.remove_package_link_target(&scope, &target);
        }
    }

    pub(super) fn prepare_incremental_package_link_patch(&self) -> PackageLinkPatch {
        let mut scope_links = FxHashMap::default();
        let mut candidates = FxHashSet::default();
        for scope in &self.incremental_package_link_scopes {
            let desired = self.package_links_for_scope(scope);
            if let Some(previous) = self.materialized_package_link_scopes.get(scope) {
                candidates.extend(previous.keys().cloned());
            }
            candidates.extend(desired.keys().cloned());
            scope_links.insert(scope.clone(), desired);
        }

        let mut previous = FxHashMap::default();
        let mut desired = FxHashMap::default();
        for path in &candidates {
            if let Some(target) = self.materialized_package_links.get(path) {
                previous.insert(path.clone(), target.clone());
            }
            let next = self.effective_link_target(path, &scope_links);
            if let Some(target) = next {
                desired.insert(path.clone(), target);
            }
        }
        PackageLinkPatch {
            scope_links,
            previous,
            desired,
            candidates,
        }
    }

    pub(super) fn commit_incremental_package_link_patch(&mut self, patch: PackageLinkPatch) {
        for scope in patch.scope_links.keys() {
            if let Some(previous) = self.materialized_package_link_scopes.remove(scope) {
                for path in previous.keys() {
                    let remove = self
                        .materialized_package_link_owners
                        .get_mut(path)
                        .is_some_and(|owners| {
                            owners.remove(scope);
                            owners.is_empty()
                        });
                    if remove {
                        self.materialized_package_link_owners.remove(path);
                    }
                }
            }
        }
        for (scope, links) in patch.scope_links {
            for (path, target) in &links {
                self.materialized_package_link_owners
                    .entry(path.clone())
                    .or_default()
                    .insert(scope.clone(), target.clone());
            }
            if !links.is_empty() {
                self.materialized_package_link_scopes.insert(scope, links);
            }
        }
        for path in patch.candidates {
            match patch.desired.get(&path) {
                Some(target) => {
                    self.materialized_package_links.insert(path, target.clone());
                }
                None => {
                    self.materialized_package_links.remove(&path);
                }
            }
        }
        self.incremental_package_link_scopes.clear();
    }

    pub(super) fn rebuild_materialized_package_link_owners(&mut self) {
        self.materialized_package_link_scopes.clear();
        self.materialized_package_link_owners.clear();
        let scopes = self
            .package_link_scope_targets
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        for scope in scopes {
            let links = self.package_links_for_scope(&scope);
            for (path, target) in &links {
                self.materialized_package_link_owners
                    .entry(path.clone())
                    .or_default()
                    .insert(scope.clone(), target.clone());
            }
            if !links.is_empty() {
                self.materialized_package_link_scopes.insert(scope, links);
            }
        }
        self.incremental_package_link_scopes.clear();
    }

    fn effective_link_target(
        &self,
        path: &Path,
        replacements: &FxHashMap<PathBuf, FxHashMap<PathBuf, PathBuf>>,
    ) -> Option<PathBuf> {
        let mut targets = self
            .materialized_package_link_owners
            .get(path)
            .into_iter()
            .flatten()
            .filter(|(scope, _)| !replacements.contains_key(*scope))
            .map(|(_, target)| target.clone())
            .collect::<Vec<_>>();
        targets.extend(
            replacements
                .values()
                .filter_map(|links| links.get(path).cloned()),
        );
        targets.into_iter().min()
    }

    fn package_links_for_scope(&self, scope: &Path) -> FxHashMap<PathBuf, PathBuf> {
        let Some(targets) = self.package_link_scope_targets.get(scope) else {
            return FxHashMap::default();
        };
        let Some(real_dir) = targets
            .keys()
            .filter(|target| target.is_dir())
            .map(|target| canonical_link_target(target))
            .min()
        else {
            return FxHashMap::default();
        };
        let files = self.package_link_scope_files.get(scope);
        merge_aware_node_modules_links(
            &real_dir,
            scope,
            files.into_iter().flatten().map(PathBuf::as_path),
        )
        .into_iter()
        .map(|link| (link.virtual_dir, canonical_link_target(&link.real_dir)))
        .collect()
    }

    fn add_package_link_target(&mut self, scope: &Path, target: &Path) {
        *self
            .package_link_scope_targets
            .entry(scope.to_path_buf())
            .or_default()
            .entry(target.to_path_buf())
            .or_default() += 1;
        self.incremental_package_link_scopes
            .insert(scope.to_path_buf());
    }

    fn remove_package_link_target(&mut self, scope: &Path, target: &Path) {
        let remove_scope = self
            .package_link_scope_targets
            .get_mut(scope)
            .is_some_and(|targets| {
                let remove_target = targets.get_mut(target).is_some_and(|count| {
                    *count = count.saturating_sub(1);
                    *count == 0
                });
                if remove_target {
                    targets.remove(target);
                }
                targets.is_empty()
            });
        if remove_scope {
            self.package_link_scope_targets.remove(scope);
        }
        self.incremental_package_link_scopes
            .insert(scope.to_path_buf());
    }

    fn package_link_scopes_for_path(&self, path: &Path) -> (Vec<PathBuf>, Vec<(PathBuf, PathBuf)>) {
        let Some(parent) = path
            .strip_prefix(&self.virtual_root)
            .ok()
            .and_then(Path::parent)
        else {
            return (Vec::new(), Vec::new());
        };
        let mut relative = PathBuf::new();
        let mut before_node_modules = true;
        let mut scopes = Vec::new();
        let mut targets = Vec::new();
        for component in parent.components() {
            relative.push(component.as_os_str());
            if component.as_os_str() == "node_modules" {
                before_node_modules = false;
                scopes.push(self.virtual_root.join(&relative));
                continue;
            }
            if before_node_modules {
                let scope = self.virtual_root.join(&relative).join("node_modules");
                let target = self.project_root.join(&relative).join("node_modules");
                scopes.push(scope.clone());
                targets.push((scope, target));
            }
        }
        scopes.sort();
        scopes.dedup();
        targets.sort();
        targets.dedup();
        (scopes, targets)
    }
}

fn canonical_link_target(path: &Path) -> PathBuf {
    std::fs::canonicalize(path)
        .map(vize_carton::path::normalize_windows_verbatim_path)
        .unwrap_or_else(|_| vize_carton::path::canonicalize_non_verbatim(path))
}
