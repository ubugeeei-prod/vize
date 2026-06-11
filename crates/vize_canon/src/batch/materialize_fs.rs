use std::io;
use std::path::{Path, PathBuf};
use std::{fs, io::ErrorKind};

use vize_carton::{FxHashSet, profiler::global_profiler};

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
    for file in expected_files {
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
