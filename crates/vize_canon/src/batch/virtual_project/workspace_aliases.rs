//! Owned links to existing authored workspace shadows, never fresh type copies.

use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, FxHashSet};

use super::VirtualProject;
use super::package_node_modules::PackageNodeModulesLink;
use super::package_shadow::PackageShadowTopology;
use crate::batch::error::CorsaResult;

impl VirtualProject {
    pub(super) fn mark_package_shadow_dependents(
        &mut self,
        keys: impl IntoIterator<Item = crate::package_route::PackageRouteKey>,
    ) {
        let mut pending = keys.into_iter().collect::<Vec<_>>();
        let mut visited = FxHashSet::default();
        while let Some(key) = pending.pop() {
            if !visited.insert(key.clone()) {
                continue;
            }
            self.package_shadow_dirty_keys.insert(key.clone());
            for ancestor in key.importer_path.ancestors() {
                if let Some(owners) = self.package_route_roots.get(ancestor) {
                    pending.extend(owners.iter().cloned());
                }
            }
        }
    }

    pub(super) fn workspace_alias_links(&self) -> Vec<PackageNodeModulesLink> {
        let mut targets = FxHashMap::<_, FxHashSet<_>>::default();
        for topology in self.package_shadow_artifacts.values() {
            for (alias, target) in &topology.aliases {
                targets
                    .entry(alias.clone())
                    .or_default()
                    .insert(target.clone());
            }
        }
        targets
            .into_iter()
            .filter_map(|(alias, targets)| {
                // Conflicting owners must not select a native identity by ordering.
                (targets.len() == 1).then(|| PackageNodeModulesLink {
                    virtual_dir: alias,
                    real_dir: targets.into_iter().next().unwrap(),
                })
            })
            .collect()
    }

    pub(super) fn workspace_alias_claims(&self) -> FxHashSet<PathBuf> {
        self.package_shadow_artifacts
            .values()
            .flat_map(|topology| {
                topology
                    .aliases
                    .keys()
                    .map(|alias| alias.join("package.json"))
            })
            .collect()
    }

    pub(super) fn install_workspace_alias_scopes(
        &mut self,
        topology: &PackageShadowTopology,
        scopes: &mut Vec<(PathBuf, PathBuf)>,
    ) {
        for (alias, target) in &topology.aliases {
            if !self.package_link_scope_targets.contains_key(alias) {
                // This is solely a link-selection claim. It is never a file
                // candidate/revision: writing through it could change a target.
                self.track_materialized_link_path(&alias.join("package.json"));
            }
            scopes.push((alias.clone(), target.clone()));
        }
    }

    pub(super) fn remove_workspace_alias_claims(&mut self, aliases: &FxHashMap<PathBuf, PathBuf>) {
        for alias in aliases.keys() {
            if !self.package_link_scope_targets.contains_key(alias) {
                self.untrack_materialized_link_path(&alias.join("package.json"));
            }
        }
    }

    pub(super) fn authored_workspace_alias_target(&self, target: &Path) -> bool {
        target.starts_with(&self.virtual_root)
            && self
                .package_shadow_manifests
                .contains_key(&target.join("package.json"))
    }

    pub(super) fn direct_workspace_alias_for_scope(
        &self,
        scope: &Path,
    ) -> Option<FxHashMap<PathBuf, PathBuf>> {
        let targets = self.package_link_scope_targets.get(scope)?;
        if !targets
            .keys()
            .any(|target| self.authored_workspace_alias_target(target))
        {
            return None;
        }
        if targets.len() != 1 {
            return Some(FxHashMap::default());
        }
        let target = targets.keys().next().unwrap();
        Some(FxHashMap::from_iter([(
            scope.to_path_buf(),
            target.clone(),
        )]))
    }

    pub(super) fn ensure_workspace_alias_targets(&self) -> CorsaResult<()> {
        let mut targets = FxHashMap::default();
        for topology in self.package_shadow_artifacts.values() {
            for (alias, target) in &topology.aliases {
                if targets
                    .insert(alias, target)
                    .is_some_and(|old| old != target)
                {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Conflicting workspace package alias targets",
                    )
                    .into());
                }
            }
        }
        for link in self.workspace_alias_links() {
            // Windows decides symlink_dir/file from target existence. The
            // planned authored root must exist before the first link is made.
            if self.authored_workspace_alias_target(&link.real_dir) {
                crate::batch::materialize_fs::ensure_dir(&link.real_dir)?;
            }
        }
        Ok(())
    }
}
