//! Installed-source storage whose native build info outlives the run (#10620).
//!
//! An installed-source mirror stays out of the shared namespace, so its writes
//! below `node_modules/` never meet a package link a previous command left
//! there (#8274). It used to live in a throwaway directory, so every run
//! checked the whole program again. Each program now owns one stable
//! directory instead. Only plain build info files outlive their owner: the
//! mirror is emptied when the directory is claimed and when its owner drops,
//! so no earlier link or file can redirect a write, and Corsa reuses the
//! diagnostics of unchanged files.

use std::fs::{self, File, OpenOptions, TryLockError};
use std::io;
use std::path::{Path, PathBuf};

const KEPT_BUILD_INFO: &str = ".kept-build-info";
const BUILD_INFO_EXTENSION: &str = "tsbuildinfo";
const OWNER_LOCK_SUFFIX: &str = ".active.lock";

pub(super) enum InstalledStorage {
    /// This program's stable directory, held exclusively.
    Stable {
        path: PathBuf,
        build_info: Option<PathBuf>,
        _owner: File,
    },
    /// Another run owns the stable directory; this one keeps a private copy.
    Private(tempfile::TempDir),
}

impl InstalledStorage {
    /// Claim the stable directory `key` below `base`, or a private one while
    /// another owner holds it.
    pub(super) fn claim(base: &Path, key: &str) -> io::Result<Self> {
        let path = base.join(key);
        if let Some(owner) = try_own(&path)? {
            fs::create_dir_all(&path)?;
            restrict(&path)?;
            // The namespace below is keyed by the canonical spelling.
            let path = vize_carton::path::canonicalize_non_verbatim(&path);
            return Ok(Self::Stable {
                path,
                build_info: None,
                _owner: owner,
            });
        }
        let storage = tempfile::Builder::new()
            .prefix("batch-installed-")
            .tempdir_in(base)?;
        restrict(storage.path())?;
        Ok(Self::Private(storage))
    }

    pub(super) fn path(&self) -> &Path {
        match self {
            Self::Stable { path, .. } => path,
            Self::Private(storage) => storage.path(),
        }
    }

    /// Empty the claimed directory except the build info files in
    /// `build_info`, which its owner keeps across runs.
    pub(super) fn reset(&mut self, build_info_dir: &Path) -> io::Result<()> {
        let Self::Stable {
            path, build_info, ..
        } = self
        else {
            return Ok(());
        };
        *build_info = Some(build_info_dir.to_path_buf());
        clear_except_build_info(path, build_info_dir)
    }
}

impl Drop for InstalledStorage {
    fn drop(&mut self) {
        if let Self::Stable {
            path,
            build_info: Some(build_info),
            ..
        } = self
        {
            let _ = clear_except_build_info(path, build_info);
            // Native sessions need no build info. Keep their original cleanup
            // contract while retaining directories that hold reusable build info.
            // The owner lock lives beside this directory and is still held here.
            let _ = fs::remove_dir(path);
        }
    }
}

/// An exclusive owner lock, or `None` while another owner holds it. Cache
/// maintenance never evicts an entry whose lock is held.
fn try_own(path: &Path) -> io::Result<Option<File>> {
    let Some(name) = path.file_name() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "installed-source storage has no name",
        ));
    };
    let mut lock_name = name.to_os_string();
    lock_name.push(OWNER_LOCK_SUFFIX);
    let lock = OpenOptions::new()
        .create(true)
        .read(true)
        .truncate(false)
        .write(true)
        .open(path.with_file_name(lock_name))?;
    match lock.try_lock() {
        Ok(()) => Ok(Some(lock)),
        Err(TryLockError::WouldBlock) => Ok(None),
        Err(TryLockError::Error(error)) => Err(error),
    }
}

fn restrict(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

/// Remove everything below `storage` but the regular `.tsbuildinfo` files
/// directly in `build_info`. Links are removed, never followed.
fn clear_except_build_info(storage: &Path, build_info: &Path) -> io::Result<()> {
    let kept = storage.join(KEPT_BUILD_INFO);
    remove_entry(&kept)?;
    let mut keeps_build_info = false;
    if build_info.starts_with(storage)
        && fs::symlink_metadata(build_info).is_ok_and(|metadata| metadata.is_dir())
    {
        fs::rename(build_info, &kept)?;
        for entry in fs::read_dir(&kept)? {
            let entry = entry?;
            let path = entry.path();
            let build_info_file = entry.file_type()?.is_file()
                && path
                    .extension()
                    .is_some_and(|extension| extension == BUILD_INFO_EXTENSION);
            if build_info_file {
                keeps_build_info = true;
            } else {
                remove_entry(&path)?;
            }
        }
    }
    for entry in fs::read_dir(storage)? {
        let path = entry?.path();
        if path != kept {
            remove_entry(&path)?;
        }
    }
    if keeps_build_info {
        if let Some(parent) = build_info.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(&kept, build_info)?;
    } else {
        remove_entry(&kept)?;
    }
    Ok(())
}

fn remove_entry(path: &Path) -> io::Result<()> {
    let result = match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() => fs::remove_dir_all(path),
        Ok(_) => fs::remove_file(path),
        Err(error) => Err(error),
    };
    match result {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        result => result,
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::InstalledStorage;

    const KEY: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    fn files_below(root: &Path) -> Vec<PathBuf> {
        let mut found = Vec::new();
        let mut pending = vec![root.to_path_buf()];
        while let Some(dir) = pending.pop() {
            for entry in std::fs::read_dir(dir).unwrap() {
                let entry = entry.unwrap();
                if entry.file_type().unwrap().is_dir() {
                    pending.push(entry.path());
                } else {
                    found.push(entry.path().strip_prefix(root).unwrap().to_path_buf());
                }
            }
        }
        found.sort();
        found
    }

    fn write(path: &Path, content: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn only_build_info_outlives_an_owner_and_earlier_links_are_never_followed() {
        let base = tempfile::tempdir().unwrap();
        let installed = tempfile::tempdir().unwrap();
        let package = installed.path().join("node_modules/selected");
        write(&package.join("index.ts"), "export const raw = 1;\n");

        // A run that dies before cleanup leaves its mirror, links included.
        let first = InstalledStorage::claim(base.path(), KEY).unwrap();
        let storage = first.path().to_path_buf();
        let build_info = storage.join("projects/p/.vize-buildinfo");
        write(&storage.join("projects/p/src/App.vue.ts"), "mirror\n");
        let link = storage.join("projects/p/node_modules/selected");
        std::fs::create_dir_all(link.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(&package, &link).unwrap();
        write(
            &build_info.join("tsconfig.0.tsbuildinfo"),
            "{\"version\":1}",
        );
        write(&build_info.join("package-links.json"), "{}");
        std::os::unix::fs::symlink(&package, build_info.join("linked.tsbuildinfo")).unwrap();
        drop(first);

        let mut next = InstalledStorage::claim(base.path(), KEY).unwrap();
        assert_eq!(next.path(), storage);
        next.reset(&build_info).unwrap();
        assert_eq!(
            files_below(&storage),
            [PathBuf::from(
                "projects/p/.vize-buildinfo/tsconfig.0.tsbuildinfo"
            )]
        );
        // The mirror now owns a real directory where the stale link was.
        write(&link.join("index.ts"), "export const mirrored = 1;\n");
        drop(next);
        assert_eq!(
            files_below(&storage),
            [PathBuf::from(
                "projects/p/.vize-buildinfo/tsconfig.0.tsbuildinfo"
            )]
        );
        assert_eq!(
            files_below(installed.path()),
            [PathBuf::from("node_modules/selected/index.ts")]
        );
        assert_eq!(
            std::fs::read_to_string(package.join("index.ts")).unwrap(),
            "export const raw = 1;\n"
        );
    }

    #[test]
    fn an_owner_without_build_info_removes_its_storage() {
        let base = tempfile::tempdir().unwrap();
        let mut owner = InstalledStorage::claim(base.path(), KEY).unwrap();
        let storage = owner.path().to_path_buf();
        let build_info = storage.join("projects/p/.vize-buildinfo");
        owner.reset(&build_info).unwrap();
        write(&storage.join("projects/p/src/App.vue.ts"), "mirror\n");
        assert!(storage.is_dir());

        drop(owner);
        assert!(!storage.exists());

        let mut next = InstalledStorage::claim(base.path(), KEY).unwrap();
        assert!(matches!(next, InstalledStorage::Stable { .. }));
        assert_eq!(next.path(), storage);
        next.reset(&build_info).unwrap();
        drop(next);
        assert!(!storage.exists());
    }

    #[test]
    fn a_concurrent_owner_keeps_private_storage() {
        let base = tempfile::tempdir().unwrap();
        let stable = InstalledStorage::claim(base.path(), KEY).unwrap();
        let private = InstalledStorage::claim(base.path(), KEY).unwrap();
        assert!(matches!(stable, InstalledStorage::Stable { .. }));
        assert!(matches!(private, InstalledStorage::Private(_)));
        let path = private.path().to_path_buf();
        drop(private);
        assert!(!path.exists());
        drop(stable);
        assert!(matches!(
            InstalledStorage::claim(base.path(), KEY).unwrap(),
            InstalledStorage::Stable { .. }
        ));
    }
}
