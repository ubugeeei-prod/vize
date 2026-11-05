//! Share source mappings while a project retains the same exact revision.

use std::path::Path;

use super::SessionCache;
use crate::corsa_bridge::{CorsaMaterializedSource, CorsaSourceCatalog};

impl SessionCache {
    pub(in crate::corsa_bridge::vue_dependencies_alias::context) fn has_project_member(
        &self,
        root: &Path,
        key: &super::RevisionKey,
    ) -> bool {
        self.project_members
            .get(root)
            .is_some_and(|members| members.contains_key(key))
    }

    pub(in crate::corsa_bridge::vue_dependencies_alias::context) fn project_overlay_epoch_changed(
        &self,
        root: &Path,
        identity: u64,
    ) -> bool {
        self.project_overlay_identities.get(root) != Some(&identity)
    }

    pub(in crate::corsa_bridge::vue_dependencies_alias::context) fn has_project_overlay_epoch(
        &self,
        root: &Path,
    ) -> bool {
        self.project_overlay_identities.contains_key(root)
    }

    pub(super) fn project_revision_is_current(&self, root: &Path, overlay_identity: u64) -> bool {
        self.project_revision_is_current_with_cache(root, overlay_identity, &mut Default::default())
    }

    pub(super) fn project_revision_is_current_with_cache(
        &self,
        root: &Path,
        overlay_identity: u64,
        observed: &mut crate::package_route::stamp::InputStampValidation,
    ) -> bool {
        self.project_overlay_identities.get(root) == Some(&overlay_identity)
            && self.project_members.get(root).is_none_or(|members| {
                self.trust_watched_disk_inputs
                    || members
                        .values()
                        .all(|member| member.fingerprint.stamps_still_valid(observed))
            })
    }

    pub(in crate::corsa_bridge::vue_dependencies_alias::context) fn source_catalog(
        &self,
        root: &Path,
    ) -> CorsaSourceCatalog {
        self.project_catalogs.get(root).cloned().unwrap_or_default()
    }

    pub(in crate::corsa_bridge::vue_dependencies_alias::context) fn set_source_catalog(
        &mut self,
        root: &Path,
        catalog: CorsaSourceCatalog,
    ) {
        self.project_catalogs.insert(root.to_path_buf(), catalog);
    }

    pub(in crate::corsa_bridge::vue_dependencies_alias::context) fn bind_materialized_source_catalog(
        &mut self,
        root: &Path,
        snapshot: &crate::batch::virtual_project::MaterializedFileSnapshot,
    ) -> CorsaSourceCatalog {
        let catalog = self.project_catalogs.entry(root.to_path_buf()).or_default();
        *catalog = catalog.with_native_snapshot(snapshot);
        catalog.clone()
    }

    pub(in crate::corsa_bridge::vue_dependencies_alias::context) fn include_source_catalog(
        &mut self,
        root: &Path,
        sources: Vec<CorsaMaterializedSource>,
    ) -> CorsaSourceCatalog {
        // A new overlay epoch changes individual leaves, never every source.
        // Exact live-file retention below retires closed/deleted members while
        // immutable old query handles keep their captured mappings.
        let catalog = self.project_catalogs.entry(root.to_path_buf()).or_default();
        *catalog = catalog.include(sources);
        catalog.clone()
    }
}
