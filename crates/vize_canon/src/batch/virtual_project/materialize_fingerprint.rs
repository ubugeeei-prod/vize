//! Write and hash only the files a graph-fingerprint hit changed.
//!
//! The cold editor union still hashes every expected file. This path patches
//! the previous snapshot so a later full capture diffs the same bytes.

use std::path::Path;

use vize_carton::FxHashSet;

use crate::batch::error::CorsaResult;
use crate::batch::materialize_fs::{ensure_dir, ensure_materialize_root};

use super::{
    AUTO_IMPORT_STUBS_FILE, MODULE_AUGMENTATION_STUBS_FILE, PACKAGE_BOUNDARY_FILE,
    VUE_MODULE_STUBS_FILE, VirtualProject,
};

impl VirtualProject {
    /// The hit touched owned files only, and a previous snapshot still names
    /// the rest of the mirror.
    pub(crate) fn fingerprint_hit_is_file_patch(
        &self,
        previous: &super::MaterializedFileSnapshot,
    ) -> bool {
        !previous.is_empty()
            && !self.incremental_link_topology_dirty
            && !self.incremental_materialized_candidates.is_empty()
            && self
                .incremental_materialized_candidates
                .iter()
                .all(|path| self.candidate_is_owned_file(path))
    }

    pub(crate) fn incremental_candidate_paths(&self) -> Vec<std::path::PathBuf> {
        self.incremental_materialized_candidates
            .iter()
            .cloned()
            .collect()
    }

    fn candidate_is_owned_file(&self, path: &Path) -> bool {
        self.virtual_files.contains_key(path)
            || self.package_shadow_files.contains_key(path)
            || self.package_shadow_manifests.contains_key(path)
            || self.passthrough_files.contains_key(path)
            || self.is_current_generated_path(path)
    }

    /// Persist the candidate files and hash those paths alone.
    pub(crate) fn materialize_fingerprint_hit(
        &mut self,
        previous: &super::MaterializedFileSnapshot,
    ) -> CorsaResult<super::MaterializedFileSnapshot> {
        let _lock = super::super::materialize_lock::MaterializeLock::acquire(&self.virtual_root)?;
        let candidates = std::mem::take(&mut self.incremental_materialized_candidates);
        ensure_materialize_root(&self.virtual_root)?;
        self.write_generated_candidates(&candidates)?;
        for path in &candidates {
            if let Some(parent) = path.parent() {
                ensure_dir(parent)?;
            }
            self.write_owned_candidate(path)?;
        }
        let current = previous.revise_files(&candidates)?;
        self.discard_incremental_materialization();
        Ok(current)
    }

    fn write_generated_candidates(
        &self,
        candidates: &FxHashSet<std::path::PathBuf>,
    ) -> CorsaResult<()> {
        if candidates.contains(&self.virtual_root.join(PACKAGE_BOUNDARY_FILE)) {
            self.write_package_boundary()?;
        }
        if candidates.contains(&self.virtual_root.join(AUTO_IMPORT_STUBS_FILE)) {
            self.write_auto_import_stubs()?;
        }
        if candidates.contains(&self.virtual_root.join(MODULE_AUGMENTATION_STUBS_FILE)) {
            self.write_module_augmentation_stubs()?;
        }
        if candidates.contains(&self.virtual_root.join(VUE_MODULE_STUBS_FILE)) {
            self.write_vue_module_stubs()?;
        }
        if candidates.contains(&self.shared_helpers_path()) && self.uses_shared_helpers() {
            self.write_shared_helpers()?;
        }
        if candidates.contains(&self.generated_tsconfig_path()) {
            self.write_tsconfig_file(&self.generated_tsconfig_path(), None, false)?;
        }
        Ok(())
    }
}
