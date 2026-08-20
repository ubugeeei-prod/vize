//! Ref-counted ownership for package shadow artifacts.

use std::path::PathBuf;

use crate::package_route::PackageRouteKey;

use super::package_shadow::PackageShadowTopology;
use super::{PackageShadowOwners, VirtualProject};

/// Insert one owner and report whether the deterministic winner changed.
fn insert_shadow_owner(
    owners: &mut PackageShadowOwners,
    key: PackageRouteKey,
    source: PathBuf,
) -> bool {
    let winner_changed = match owners.first_key_value() {
        None => true,
        Some((winner_key, winner_source)) => {
            key < *winner_key || (key == *winner_key && source != *winner_source)
        }
    };
    owners.insert(key, source);
    winner_changed
}

/// Remove one owner and report whether it was the deterministic winner.
fn remove_shadow_owner(owners: &mut PackageShadowOwners, key: &PackageRouteKey) -> bool {
    let winner_removed = owners
        .first_key_value()
        .is_some_and(|(winner_key, _)| winner_key == key);
    owners.remove(key);
    winner_removed
}

impl VirtualProject {
    pub(super) fn install_package_shadow_owner(
        &mut self,
        key: PackageRouteKey,
        topology: PackageShadowTopology,
    ) {
        let new_files = topology
            .files
            .keys()
            .filter(|path| !self.package_shadow_files.contains_key(*path))
            .cloned()
            .collect::<Vec<_>>();
        let new_manifests = topology
            .manifests
            .keys()
            .filter(|path| !self.package_shadow_manifests.contains_key(*path))
            .cloned()
            .collect::<Vec<_>>();
        self.incremental_materialized_candidates.extend(
            topology
                .files
                .keys()
                .chain(topology.manifests.keys())
                .cloned(),
        );
        for (path, source) in &topology.files {
            let winner_changed = insert_shadow_owner(
                self.package_shadow_file_owners
                    .entry(path.clone())
                    .or_default(),
                key.clone(),
                source.clone(),
            );
            if winner_changed {
                self.refresh_shadow_file(path);
            }
        }
        for (path, source) in &topology.manifests {
            let winner_changed = insert_shadow_owner(
                self.package_shadow_manifest_owners
                    .entry(path.clone())
                    .or_default(),
                key.clone(),
                source.clone(),
            );
            if winner_changed {
                self.refresh_shadow_manifest(path);
            }
        }
        for path in new_files.iter().chain(&new_manifests) {
            self.track_materialized_link_path(path);
        }
        self.install_package_shadow_link_scopes(&key, &topology);
        self.package_shadow_artifacts.insert(key, topology);
    }

    pub(super) fn remove_package_shadow_owner(&mut self, key: &PackageRouteKey) {
        let Some(topology) = self.package_shadow_artifacts.remove(key) else {
            return;
        };
        self.remove_package_shadow_link_scopes(key);
        self.incremental_materialized_candidates.extend(
            topology
                .files
                .keys()
                .chain(topology.manifests.keys())
                .cloned(),
        );
        for path in topology.files.keys() {
            let (winner_removed, empty) =
                self.package_shadow_file_owners
                    .get_mut(path)
                    .map_or((false, false), |owners| {
                        let winner_removed = remove_shadow_owner(owners, key);
                        (winner_removed, owners.is_empty())
                    });
            if empty {
                self.package_shadow_file_owners.remove(path);
            }
            if winner_removed {
                self.refresh_shadow_file(path);
            }
        }
        for path in topology.manifests.keys() {
            let (winner_removed, empty) = self.package_shadow_manifest_owners.get_mut(path).map_or(
                (false, false),
                |owners| {
                    let winner_removed = remove_shadow_owner(owners, key);
                    (winner_removed, owners.is_empty())
                },
            );
            if empty {
                self.package_shadow_manifest_owners.remove(path);
            }
            if winner_removed {
                self.refresh_shadow_manifest(path);
            }
        }
        for path in topology.files.keys() {
            if !self.package_shadow_files.contains_key(path) {
                self.untrack_materialized_link_path(path);
            }
        }
        for path in topology.manifests.keys() {
            if !self.package_shadow_manifests.contains_key(path) {
                self.untrack_materialized_link_path(path);
            }
        }
    }

    fn refresh_shadow_file(&mut self, path: &PathBuf) {
        if let Some(previous) = self.package_shadow_files.get(path) {
            let remove = self
                .package_shadow_source_paths
                .get_mut(previous)
                .is_some_and(|paths| {
                    paths.remove(path);
                    paths.is_empty()
                });
            if remove {
                self.package_shadow_source_paths.remove(previous);
            }
        }
        match self
            .package_shadow_file_owners
            .get(path)
            .and_then(PackageShadowOwners::first_key_value)
        {
            Some((_, source)) => {
                self.package_shadow_files
                    .insert(path.clone(), source.clone());
                self.package_shadow_source_paths
                    .entry(source.clone())
                    .or_default()
                    .insert(path.clone());
            }
            None => {
                self.package_shadow_files.remove(path);
            }
        }
    }

    pub(super) fn mark_package_shadow_source_changed(&mut self, canonical_path: &PathBuf) {
        if let Some(paths) = self.package_shadow_source_paths.get(canonical_path) {
            self.incremental_materialized_candidates
                .extend(paths.iter().cloned());
        }
    }

    fn refresh_shadow_manifest(&mut self, path: &PathBuf) {
        match self
            .package_shadow_manifest_owners
            .get(path)
            .and_then(PackageShadowOwners::first_key_value)
        {
            Some((_, source)) => {
                self.package_shadow_manifests
                    .insert(path.clone(), source.clone());
            }
            None => {
                self.package_shadow_manifests.remove(path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::PackageResolutionMode;
    use crate::package_route::PackageRouteKey;

    use super::{PackageShadowOwners, insert_shadow_owner, remove_shadow_owner};

    fn route_key(importer: impl Into<PathBuf>) -> PackageRouteKey {
        PackageRouteKey {
            importer_path: importer.into(),
            specifier: "@w/ui".into(),
            occurrence_mode: PackageResolutionMode::Import,
        }
    }

    #[test]
    fn ascending_fan_out_refreshes_only_the_first_winner() {
        let mut owners = PackageShadowOwners::default();
        let refreshes = (0..256)
            .filter(|index| {
                insert_shadow_owner(
                    &mut owners,
                    route_key(vize_carton::cstr!("/workspace/apps/app{index}/View.vue").as_str()),
                    PathBuf::from("/workspace/packages/ui/src/index.ts"),
                )
            })
            .count();

        assert_eq!(refreshes, 1);
        assert_eq!(owners.len(), 256);
    }

    #[test]
    fn an_earlier_owner_or_changed_winning_source_refreshes() {
        let mut owners = PackageShadowOwners::default();
        let late = route_key("/workspace/apps/z/View.vue");
        let early = route_key("/workspace/apps/a/View.vue");
        let first_source = PathBuf::from("/workspace/packages/ui/src/index.ts");
        let changed_source = PathBuf::from("/workspace/vendor/ui/src/index.ts");

        assert!(insert_shadow_owner(&mut owners, late, first_source.clone()));
        assert!(insert_shadow_owner(
            &mut owners,
            early.clone(),
            first_source
        ));
        assert!(insert_shadow_owner(
            &mut owners,
            early.clone(),
            changed_source.clone()
        ));
        assert!(!insert_shadow_owner(&mut owners, early, changed_source));
    }

    #[test]
    fn removal_refreshes_only_when_the_winner_is_removed() {
        let mut owners = PackageShadowOwners::default();
        let winner = route_key("/workspace/apps/a/View.vue");
        let other = route_key("/workspace/apps/z/View.vue");
        let source = PathBuf::from("/workspace/packages/ui/src/index.ts");
        insert_shadow_owner(&mut owners, winner.clone(), source.clone());
        insert_shadow_owner(&mut owners, other.clone(), source);

        assert!(!remove_shadow_owner(&mut owners, &other));
        assert!(remove_shadow_owner(&mut owners, &winner));
        assert!(owners.is_empty());
    }
}
