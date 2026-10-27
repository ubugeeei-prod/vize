use std::io;
use std::path::{Path, PathBuf};
use std::{fs, io::ErrorKind};

use vize_carton::{FxHashSet, profiler::global_profiler};

mod owned_write;
pub(super) use owned_write::{must_replace, replace as replace_file};

pub(super) fn ensure_dir(path: &Path) -> io::Result<()> {
    match fs::create_dir_all(path) {
        Ok(()) => {
            global_profiler().record_fs_create_dir_all();
            Ok(())
        }
        Err(error) => {
            global_profiler().record_fs_create_dir_all_failure();
            Err(error)
        }
    }
}

pub(super) fn ensure_materialize_root(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() => {
            Ok(())
        }
        Ok(_) => {
            remove_path(path)?;
            ensure_dir(path)
        }
        Err(error) if error.kind() == ErrorKind::NotFound => ensure_dir(path),
        Err(error) => Err(error),
    }
}

pub(super) fn write_if_changed(path: &Path, content: &[u8]) -> io::Result<()> {
    // Skipping same-content writes matters more than saving the write syscall
    // itself: TypeScript/Corsa watch file mtimes and may invalidate internal
    // state when `tsconfig.json`, stubs, or materialized sources are touched.
    // The length check avoids reading most stale files before the byte
    // comparison.
    match fs::symlink_metadata(path) {
        Ok(metadata) if !metadata.file_type().is_file() || metadata.file_type().is_symlink() => {
            remove_path(path)?;
        }
        Ok(metadata)
            if metadata.file_type().is_file()
                && metadata.len() == content.len() as u64
                && file_bytes_match(path, content)? =>
        {
            let profiler = global_profiler();
            profiler.record_counter("io.write.skipped.calls", 1);
            profiler.record_counter("io.write.skipped.bytes", content.len() as u64);
            return Ok(());
        }
        Ok(metadata) if owned_write::must_replace(&metadata) => {
            return owned_write::replace(path, content);
        }
        Ok(_) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }

    write_file(path, content)
}

pub(super) fn write_file(path: &Path, content: &[u8]) -> io::Result<()> {
    match fs::write(path, content) {
        Ok(()) => {
            global_profiler().record_fs_write(content.len());
            Ok(())
        }
        Err(error) => {
            global_profiler().record_fs_write_failure(content.len());
            Err(error)
        }
    }
}

fn file_bytes_match(path: &Path, expected: &[u8]) -> io::Result<bool> {
    match fs::read(path) {
        Ok(existing) => {
            let profiler = global_profiler();
            profiler.record_counter("io.read.calls", 1);
            profiler.record_counter("io.read.bytes", existing.len() as u64);
            profiler.record_counter("syscall.fs.read.calls", 1);
            Ok(existing == expected)
        }
        Err(error) => {
            let profiler = global_profiler();
            profiler.record_counter("io.read.calls", 1);
            profiler.record_counter("io.read.failures", 1);
            profiler.record_counter("syscall.fs.read.calls", 1);
            profiler.record_counter("syscall.fs.read.failures", 1);
            Err(error)
        }
    }
}

pub(super) fn remove_path(path: &Path) -> io::Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    let file_type = metadata.file_type();
    #[cfg(windows)]
    if (file_type.is_dir() || path.is_dir()) && fs::read_link(path).is_ok() {
        // Junctions report as directories rather than symlinks. Remove the
        // junction itself, never recursively delete its package-store target.
        return fs::remove_dir(path);
    }
    if file_type.is_dir() && !file_type.is_symlink() {
        match fs::remove_dir_all(path) {
            Ok(()) => {
                global_profiler().record_fs_remove_dir_all();
                Ok(())
            }
            Err(error) => {
                global_profiler().record_fs_remove_dir_all_failure();
                Err(error)
            }
        }
    } else {
        match fs::remove_file(path) {
            Ok(()) => {
                global_profiler().record_counter("syscall.fs.remove_file.calls", 1);
                Ok(())
            }
            Err(error) => {
                let profiler = global_profiler();
                profiler.record_counter("syscall.fs.remove_file.calls", 1);
                profiler.record_counter("syscall.fs.remove_file.failures", 1);
                Err(error)
            }
        }
    }
}

/// Retire one previously owned cache file without following a new package
/// link into an external dependency store. Directories can now own live
/// descendants and must be preserved as well.
pub(super) fn retire_owned_file(root: &Path, path: &Path) -> io::Result<()> {
    let relative = path.strip_prefix(root).map_err(|_| {
        io::Error::new(
            ErrorKind::InvalidInput,
            "retired file is outside its cache root",
        )
    })?;
    let mut components = relative.components().peekable();
    if components.peek().is_none() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "cannot retire the cache root",
        ));
    }
    let mut current = root.to_path_buf();
    for root_or_component in std::iter::once(None).chain(components.map(Some)) {
        if let Some(component) = root_or_component {
            if !matches!(component, std::path::Component::Normal(_)) {
                return Err(io::Error::new(
                    ErrorKind::InvalidInput,
                    "invalid retired cache path",
                ));
            }
            current.push(component);
        }
        let metadata = match fs::symlink_metadata(&current) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error),
        };
        if metadata.file_type().is_symlink() || (cfg!(windows) && fs::read_link(&current).is_ok()) {
            return Ok(());
        }
        if current == path {
            return if metadata.is_file() {
                remove_path(path)
            } else {
                Ok(())
            };
        }
        if !metadata.is_dir() {
            return Ok(());
        }
    }
    Ok(())
}

pub(super) fn prune_unexpected_entries(
    root: &Path,
    expected_files: &FxHashSet<PathBuf>,
    preserved_roots: &[PathBuf],
) -> io::Result<()> {
    // Build the complete expected directory set once, then walk the cache tree
    // recursively. This removes stale generated files without the old "delete the
    // entire materialize root and recreate it" pattern, preserving hot dependency
    // mirrors and avoiding large remove/create storms between check runs.
    let mut expected_dirs = FxHashSet::default();
    expected_dirs.insert(root.to_path_buf());
    // Keep the directories that only hold preserved entries (`@scope` above a
    // preserved `@scope/pkg` link) and prune their other children, instead
    // of deleting the preserved entry with its parent.
    for file in expected_files.iter().chain(preserved_roots) {
        let mut parent = file.parent();
        while let Some(dir) = parent {
            if !dir.starts_with(root) {
                break;
            }
            expected_dirs.insert(dir.to_path_buf());
            if dir == root {
                break;
            }
            parent = dir.parent();
        }
    }

    prune_dir(root, expected_files, &expected_dirs, preserved_roots)
}

pub(super) fn prune_dir_entries(
    root: &Path,
    expected_files: &FxHashSet<PathBuf>,
) -> io::Result<()> {
    prune_unexpected_entries(root, expected_files, &[])
}

fn prune_dir(
    dir: &Path,
    expected_files: &FxHashSet<PathBuf>,
    expected_dirs: &FxHashSet<PathBuf>,
    preserved_roots: &[PathBuf],
) -> io::Result<()> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if preserved_roots.contains(&path) {
            continue;
        }

        let file_type = entry.file_type()?;
        if file_type.is_dir() && !file_type.is_symlink() {
            if expected_files.contains(&path) {
                remove_path(&path)?;
                continue;
            }
            if !expected_dirs.contains(&path) {
                remove_path(&path)?;
                continue;
            }
            prune_dir(&path, expected_files, expected_dirs, preserved_roots)?;
        } else if !expected_files.contains(&path) || file_type.is_symlink() {
            remove_path(&path)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{remove_path, retire_owned_file};

    #[test]
    fn snapshot_retirement_removes_only_its_owned_leaf_and_is_idempotent() {
        let root = tempfile::tempdir().unwrap();
        let directory = root.path().join("node_modules/@scope/pkg");
        std::fs::create_dir_all(&directory).unwrap();
        let retired = directory.join("Retired.d.vue.ts");
        let retained = directory.join("retained.d.ts");
        std::fs::write(&retired, "export {};\n").unwrap();
        std::fs::write(&retained, "real dependency bytes\n").unwrap();
        retire_owned_file(root.path(), &retired).unwrap();
        retire_owned_file(root.path(), &retired).unwrap();
        assert!(!retired.exists());
        assert_eq!(
            std::fs::read_to_string(&retained).unwrap(),
            "real dependency bytes\n"
        );
        assert!(retire_owned_file(root.path(), root.path()).is_err());
        assert!(retire_owned_file(root.path(), &root.path().join("../outside.ts")).is_err());
    }

    #[test]
    fn snapshot_retirement_preserves_external_store_links_and_new_directory_owners() {
        let root = tempfile::tempdir().unwrap();
        let cache = root.path().join("cache");
        let store = root.path().join("store");
        std::fs::create_dir_all(&cache).unwrap();
        std::fs::create_dir_all(&store).unwrap();
        let file = store.join("Retired.d.vue.ts");
        std::fs::write(&file, "external store bytes\n").unwrap();
        let link = cache.join("package");
        crate::batch::runtime_deps::symlink_package_dir(&store, &link).unwrap();
        retire_owned_file(&cache, &link.join("Retired.d.vue.ts")).unwrap();
        assert_eq!(
            std::fs::read_to_string(&file).unwrap(),
            "external store bytes\n"
        );
        assert!(std::fs::read_link(&link).is_ok());
        let replacement = cache.join("old-file.ts");
        std::fs::create_dir_all(&replacement).unwrap();
        std::fs::write(replacement.join("live.ts"), "new live owner\n").unwrap();
        retire_owned_file(&cache, &replacement).unwrap();
        assert_eq!(
            std::fs::read_to_string(replacement.join("live.ts")).unwrap(),
            "new live owner\n"
        );
    }

    #[test]
    fn removing_a_directory_link_preserves_its_target() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("store/package");
        let link = temp.path().join("mirror/package");
        std::fs::create_dir_all(&source).unwrap();
        std::fs::write(source.join("sentinel.txt"), "user data").unwrap();
        crate::batch::runtime_deps::symlink_package_dir(&source, &link).unwrap();

        remove_path(&link).unwrap();

        assert!(std::fs::symlink_metadata(&link).is_err());
        assert_eq!(
            std::fs::read_to_string(source.join("sentinel.txt")).unwrap(),
            "user data"
        );
    }
}
