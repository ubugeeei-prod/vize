//! Share source mappings while a project retains the same exact revision.

use std::path::Path;

use super::SessionCache;
use crate::corsa_bridge::{CorsaMaterializedSource, CorsaSourceCatalog};

impl SessionCache {
    pub(super) fn project_revision_is_current(&self, root: &Path, overlay_identity: u64) -> bool {
        self.project_revision_is_current_with_cache(root, overlay_identity, &mut Default::default())
    }

    pub(super) fn project_revision_is_current_with_cache(
        &self,
        root: &Path,
        overlay_identity: u64,
        observed: &mut crate::package_route::stamp::InputStampCache,
    ) -> bool {
        self.project_overlay_identities.get(root) == Some(&overlay_identity)
            && self.project_members.get(root).is_none_or(|members| {
                members.values().all(|member| {
                    member
                        .stamps
                        .iter()
                        .all(|stamp| stamp.is_current_with_cache(observed))
                })
            })
    }

    pub(in crate::corsa_bridge::vue_dependencies_alias::context) fn source_catalog(
        &self,
        root: &Path,
    ) -> CorsaSourceCatalog {
        self.project_catalogs.get(root).cloned().unwrap_or_default()
    }

    pub(in crate::corsa_bridge::vue_dependencies_alias::context) fn include_source_catalog(
        &mut self,
        root: &Path,
        overlay_identity: u64,
        sources: Vec<CorsaMaterializedSource>,
    ) -> CorsaSourceCatalog {
        if !self.project_revision_is_current(root, overlay_identity) {
            self.project_catalogs
                .insert(root.to_path_buf(), CorsaSourceCatalog::default());
        }
        let catalog = self.project_catalogs.entry(root.to_path_buf()).or_default();
        *catalog = catalog.include(sources);
        catalog.clone()
    }
}
