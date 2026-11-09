//! Canonical directories reached from a directory this snapshot already
//! canonicalized, one `lstat` per remaining component.
//!
//! A candidate directory is an importer's canonical directory joined with the
//! specifier's directory part (`../../kit`). Linux `realpath` keeps every
//! spelled name and only follows links, so from a canonical prefix `..` is the
//! lexical parent, `.` is nothing, and a name that is no link is kept as
//! spelled. That is one `lstat` for `../../kit` instead of a `readlink` per
//! component of the whole path (a 9P round trip each on a WSL2 Windows mount).
//! macOS `realpath` also respells names in their on-disk case, so other
//! systems keep the full canonicalization.

use std::path::{Path, PathBuf};

use super::TypeSourceSnapshot;

impl TypeSourceSnapshot {
    /// Remember a directory path that is its own `realpath`.
    pub(super) fn note_canonical_directory(&self, directory: &Path) {
        if let Ok(mut known) = self.canonical_directories.lock() {
            known.insert(directory.to_path_buf());
        }
    }

    /// `realpath(directory)`, from its nearest known canonical ancestor
    /// spelling. `None` hands the directory to a full canonicalization: no
    /// known ancestor, a link on the way, or a component that is no directory.
    // Tests on other Unix systems exercise the walk with exactly cased paths.
    #[cfg(any(target_os = "linux", all(test, unix)))]
    pub(super) fn walk_canonical_directory(&self, directory: &Path) -> Option<PathBuf> {
        use std::path::Component;
        let anchor = {
            let known = self.canonical_directories.lock().ok()?;
            directory
                .ancestors()
                .find(|ancestor| known.contains(*ancestor))?
                .to_path_buf()
        };
        let mut current = anchor;
        for component in directory.strip_prefix(&current).ok()?.components() {
            match component {
                Component::CurDir => {}
                // The parent of a canonical path is canonical; `/..` is `/`.
                Component::ParentDir => {
                    current.pop();
                }
                Component::Normal(name) => {
                    current.push(name);
                    let metadata = std::fs::symlink_metadata(&current).ok()?;
                    if !metadata.is_dir() {
                        return None;
                    }
                }
                Component::RootDir | Component::Prefix(_) => return None,
            }
        }
        Some(current)
    }

    #[cfg(not(any(target_os = "linux", all(test, unix))))]
    pub(super) fn walk_canonical_directory(&self, _directory: &Path) -> Option<PathBuf> {
        None
    }
}
