//! CLI storage ownership is independent of the authored module-resolution root.

use std::path::{Path, PathBuf};

use super::super::{BatchTypeChecker, CorsaResult};
use crate::batch::virtual_project::{persistent_batch_virtual_root, prepare_persistent_batch_root};

pub(in crate::batch::type_checker) struct PersistentStorageOwner {
    root: PathBuf,
    config: Option<PathBuf>,
}

impl BatchTypeChecker {
    /// Reuse a private, owner-keyed namespace when this program needs an
    /// expanded resolution root or explicitly registered installed files.
    /// Set before scanning. Default embedding callers keep their existing
    /// installed-source lifetime; ordinary single-root namespaces stay unchanged.
    pub fn set_persistent_storage_owner(
        &mut self,
        owner: &Path,
        identity_config: Option<&Path>,
    ) -> CorsaResult<()> {
        if self.scanned || self.project.file_count() != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Persistent storage ownership must be selected before scanning",
            )
            .into());
        }
        self.persistent_storage_owner = Some(PersistentStorageOwner {
            root: vize_carton::path::canonicalize_non_verbatim(owner),
            config: identity_config.map(vize_carton::path::canonicalize_non_verbatim),
        });
        Ok(())
    }

    pub(super) fn scope_persistent_initial_storage(
        &mut self,
        installed: bool,
    ) -> CorsaResult<bool> {
        let Some(owner) = self
            .persistent_storage_owner
            .as_ref()
            .filter(|owner| installed || owner.root.as_path() != self.project.project_root())
        else {
            return Ok(false);
        };
        if self.project.file_count() != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Persistent installed storage requires a fresh checker",
            )
            .into());
        }
        let root = persistent_batch_virtual_root(
            self.project.project_root(),
            &owner.root,
            owner.config.as_deref(),
        );
        // An owner-specific cache redirect can still sit inside the expanded
        // semantic workspace. Keep the existing isolated lifetime in that
        // case, rather than minting new source-glob inputs inside the program.
        if root.starts_with(self.project.project_root()) || root.starts_with(&owner.root) {
            return Ok(false);
        }
        let lease = prepare_persistent_batch_root(&owner.root)?;
        let context_lease = self.project.scope_persistent_batch_namespace(root)?;
        self._persistent_storage_leases = Some((lease, context_lease));
        Ok(true)
    }
}
