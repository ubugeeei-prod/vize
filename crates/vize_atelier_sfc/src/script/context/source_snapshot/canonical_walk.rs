//! Canonical directories reached from a directory this snapshot already
//! canonicalized, one walked name per remaining component.
//!
//! A candidate directory is an importer's canonical directory joined with the
//! specifier's directory part (`../../kit`). Linux `realpath` keeps every
//! spelled name and only follows links, so from a canonical prefix `..` is the
//! lexical parent, `.` is nothing, and a name that is no link is kept as
//! spelled. That is one lookup for `../../kit` instead of a `readlink` per
//! component of the whole path (a 9P round trip each on a WSL2 Windows mount).
//! For a request's snapshot on Linux, an `O_PATH | O_NOFOLLOW | O_DIRECTORY`
//! open is that lookup: it fails on a link or a non-directory, asks a Windows
//! mount for no attributes, and its handle keeps the walked directory resolved
//! for the snapshot.
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
        let mut walked = None;
        for component in directory.strip_prefix(&current).ok()?.components() {
            match component {
                Component::CurDir => {}
                // The parent of a canonical path is canonical; `/..` is `/`.
                Component::ParentDir => {
                    current.pop();
                    walked = None;
                }
                Component::Normal(name) => {
                    current.push(name);
                    walked = Some(self.unlinked_directory(&current)?);
                }
                Component::RootDir | Component::Prefix(_) => return None,
            }
        }
        // `..` alone stays under handles the anchor already keeps.
        #[cfg(target_os = "linux")]
        if let Some(Some(handle)) = walked {
            self.held.hold(handle);
        }
        #[cfg(not(target_os = "linux"))]
        let _ = walked;
        Some(current)
    }

    #[cfg(not(any(target_os = "linux", all(test, unix))))]
    pub(super) fn walk_canonical_directory(&self, _directory: &Path) -> Option<PathBuf> {
        None
    }

    /// `Some` when `path` is a directory and no link, with a handle to hold
    /// for a request's snapshot.
    #[cfg(target_os = "linux")]
    fn unlinked_directory(&self, path: &Path) -> Option<Option<std::fs::File>> {
        if self.held.enabled() {
            return super::realpath::open_path(path, libc::O_NOFOLLOW | libc::O_DIRECTORY)
                .ok()
                .map(Some);
        }
        is_unlinked_directory(path).then_some(None)
    }

    #[cfg(all(test, unix, not(target_os = "linux")))]
    fn unlinked_directory(&self, path: &Path) -> Option<()> {
        is_unlinked_directory(path).then_some(())
    }
}

#[cfg(any(target_os = "linux", all(test, unix)))]
fn is_unlinked_directory(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_dir())
}
