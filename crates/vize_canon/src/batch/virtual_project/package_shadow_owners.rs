//! Ref-counted ownership for package shadow artifacts.

use std::path::PathBuf;

use crate::package_route::PackageRouteKey;

use super::VirtualProject;
use super::package_shadow::PackageShadowTopology;

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
            self.package_shadow_file_owners
                .entry(path.clone())
                .or_default()
                .insert(key.clone(), source.clone());
            self.refresh_shadow_file(path);
        }
        for (path, source) in &topology.manifests {
            self.package_shadow_manifest_owners
                .entry(path.clone())
                .or_default()
                .insert(key.clone(), source.clone());
            self.refresh_shadow_manifest(path);
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
            let empty = self
                .package_shadow_file_owners
                .get_mut(path)
                .is_some_and(|owners| {
                    owners.remove(key);
                    owners.is_empty()
                });
            if empty {
                self.package_shadow_file_owners.remove(path);
            }
            self.refresh_shadow_file(path);
        }
        for path in topology.manifests.keys() {
            let empty = self
                .package_shadow_manifest_owners
                .get_mut(path)
                .is_some_and(|owners| {
                    owners.remove(key);
                    owners.is_empty()
                });
            if empty {
                self.package_shadow_manifest_owners.remove(path);
            }
            self.refresh_shadow_manifest(path);
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
            .and_then(|owners| owners.iter().min_by_key(|(key, _)| *key))
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
            .and_then(|owners| owners.iter().min_by_key(|(key, _)| *key))
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
