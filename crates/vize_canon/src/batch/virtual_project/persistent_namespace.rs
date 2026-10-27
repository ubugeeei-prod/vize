//! Reusable installed-source storage has an explicit owner and write boundary.

use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use vize_carton::FxHashSet;

use crate::batch::materialize_fs::{
    ensure_materialize_root, must_replace, remove_path, replace_file,
};

use super::VirtualProject;

const SCHEMA: &[u8] = b"vize-persistent-installed-batch:v1\0";

/// All private installed-source contexts owned by this invocation root.
/// This is separate from ordinary mirrors, whose package links are not
/// authority to write installed source files.
#[expect(
    clippy::expect_used,
    reason = "project storage paths have a fixed minted shape"
)]
pub fn persistent_batch_storage_root(owner: &Path) -> PathBuf {
    let ordinary = super::project_virtual_root(owner);
    ordinary
        .parent()
        .and_then(Path::parent)
        .expect("project storage has a Canon parent")
        .join("private-batch-v1")
        .join(ordinary.file_name().expect("project storage has a key"))
}

/// Preserve authored resolution topology while separating the storage owner.
pub fn persistent_batch_virtual_root(
    project_root: &Path,
    owner: &Path,
    config: Option<&Path>,
) -> PathBuf {
    let mut digest = Sha256::new();
    digest.update(SCHEMA);
    for path in [Some(project_root), Some(owner), config] {
        digest.update([u8::from(path.is_some())]);
        if let Some(path) = path {
            let path = vize_carton::path::canonicalize_non_verbatim(path);
            let bytes = path.as_os_str().as_encoded_bytes();
            digest.update((bytes.len() as u64).to_le_bytes());
            digest.update(bytes);
        }
    }
    persistent_batch_storage_root(owner)
        .join("canon/projects")
        .join(super::identity::encode_digest(digest.finalize()).as_str())
}

/// Only paths in a newly introduced private namespace may be reclaimed here.
pub(in crate::batch) fn prepare_persistent_batch_root(owner: &Path) -> io::Result<fs::File> {
    prepare_persistent_storage(
        persistent_batch_storage_root(owner),
        #[cfg(test)]
        |_| {},
    )
}

#[inline]
#[expect(
    clippy::expect_used,
    reason = "private storage paths have a fixed minted shape"
)]
fn prepare_persistent_storage(
    storage: PathBuf,
    #[cfg(test)] owned_root_probe: impl FnOnce(&Path),
) -> io::Result<fs::File> {
    ensure_materialize_root(storage.parent().expect("private storage has a parent"))?;
    // GC can retire an idle owner as soon as its first directory exists.
    // Hold the owner lease before creating or changing any owned directory.
    let lease = super::cache_gc::acquire_usage_lock(&storage)
        .ok_or_else(|| io::Error::other("Cannot lock the private batch storage owner"))?;
    ensure_materialize_root(&storage)?;
    #[cfg(test)]
    owned_root_probe(&storage);
    for path in [storage.join("canon"), storage.join("canon/projects")] {
        ensure_materialize_root(&path)?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&storage, fs::Permissions::from_mode(0o700))?;
    }
    // Bound both the number of owners and each owner's contexts. A live
    // context holds this owner lease, so outer GC cannot retire another check.
    super::cache_gc::maintain_project_cache(&storage);
    Ok(lease)
}

impl VirtualProject {
    /// The batch checker, rather than this project, holds the returned context
    /// lease until its native executor has dropped.
    pub(crate) fn scope_persistent_batch_namespace(
        &mut self,
        root: PathBuf,
    ) -> io::Result<fs::File> {
        debug_assert!(self.virtual_files.is_empty());
        debug_assert!(self.package_shadow_files.is_empty());
        let lease = super::cache_gc::acquire_usage_lock(&root)
            .ok_or_else(|| io::Error::other("Cannot lock the private batch context"))?;
        self.virtual_root = root;
        self.private_batch_namespace = true;
        self._cache_usage_lock = None;
        Ok(lease)
    }

    /// The caller holds MaterializeLock through publication and the native
    /// check. Old directory links must not authorize writes to their targets.
    pub(super) fn prepare_owned_batch_writes(&self, files: &FxHashSet<PathBuf>) -> io::Result<()> {
        if !self.private_batch_namespace {
            return Ok(());
        }
        ensure_materialize_root(&self.virtual_root)?;
        let mut directories = FxHashSet::default();
        directories.insert(self.virtual_root.join(super::CLI_BUILD_INFO_DIR));
        for file in files {
            let relative = file.strip_prefix(&self.virtual_root).map_err(|_| {
                io::Error::new(
                    ErrorKind::InvalidInput,
                    "owned batch file escapes its namespace",
                )
            })?;
            if relative.as_os_str().is_empty()
                || relative
                    .components()
                    .any(|part| !matches!(part, std::path::Component::Normal(_)))
            {
                return Err(io::Error::new(
                    ErrorKind::InvalidInput,
                    "invalid owned batch path",
                ));
            }
            for parent in file
                .ancestors()
                .skip(1)
                .take_while(|path| *path != self.virtual_root)
            {
                directories.insert(parent.to_path_buf());
            }
        }
        // A path prefix sorts before its descendants: inspect parents before
        // asking the filesystem about any child through a former link.
        let mut directories = directories.into_iter().collect::<Vec<_>>();
        directories.sort();
        for path in directories {
            match fs::symlink_metadata(&path) {
                Ok(metadata)
                    if metadata.is_dir()
                        && !metadata.file_type().is_symlink()
                        && !(cfg!(windows) && fs::read_link(&path).is_ok()) => {}
                Ok(_) => {
                    remove_path(&path)?;
                    fs::create_dir(&path)?;
                }
                Err(error) if error.kind() == ErrorKind::NotFound => fs::create_dir(&path)?,
                Err(error) => return Err(error),
            }
        }
        // Corsa writes build info itself, outside write_if_changed. Preserve
        // healthy incremental state while detaching forged linked outputs.
        for entry in fs::read_dir(self.virtual_root.join(super::CLI_BUILD_INFO_DIR))? {
            let path = entry?.path();
            let metadata = fs::symlink_metadata(&path)?;
            if metadata.file_type().is_symlink() || (cfg!(windows) && fs::read_link(&path).is_ok())
            {
                remove_path(&path)?;
            } else if metadata.is_file() && must_replace(&metadata) {
                // On Windows the stable metadata API cannot inspect link
                // counts: replace the leaf while keeping its build-info bytes.
                replace_file(&path, &fs::read(&path)?)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
