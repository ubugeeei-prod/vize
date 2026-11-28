//! Real paths of `/// <reference path>` targets, one directory at a time.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use vize_l0::{FxHashMap, FxHashSet};

use super::super::glob::normalize_input_path;

/// Canonicalizes reference targets through their directories.
///
/// A declaration package references many siblings (`@types/node`: 248
/// `<reference path>` lines over 82 files), and canonicalizing each spelled
/// path resolves every component again: a `getattrlist` per component on
/// macOS, a `readlink` per component on Linux. A target that is a regular
/// file (not a symlink) listed in its directory under exactly the spelled name
/// has the directory's real path joined with that name as its real path, so
/// each directory is canonicalized and listed once. A directory is listed
/// only once a second target in it is asked for: a lone reference costs what
/// it did. Anything else (a symlink,
/// a directory, a missing file, a name the listing spells differently such
/// as another case on a case-insensitive volume, a path ending in `..`) is
/// canonicalized whole, as before.
#[derive(Default)]
pub(super) struct ReferenceRealpaths {
    directories: FxHashMap<PathBuf, Listing>,
    pub(super) installed: FxHashSet<PathBuf>,
}

enum Listing {
    /// One target was asked for and canonicalized whole.
    Once,
    /// The directory's real path and regular files, or `None` when either
    /// could not be read.
    Read(Option<Directory>),
}

struct Directory {
    real: PathBuf,
    files: FxHashSet<OsString>,
}

impl ReferenceRealpaths {
    /// What `normalize_input_path(path)` returns.
    pub(super) fn resolve(&mut self, path: &Path) -> PathBuf {
        let (Some(parent), Some(name)) = (path.parent(), path.file_name()) else {
            return normalize_input_path(path);
        };
        let listing = match self.directories.get_mut(parent) {
            None => {
                self.directories.insert(parent.to_path_buf(), Listing::Once);
                return normalize_input_path(path);
            }
            Some(listing) => listing,
        };
        if matches!(listing, Listing::Once) {
            *listing = Listing::Read(Directory::read(parent));
        }
        match listing {
            Listing::Read(Some(directory)) if directory.files.contains(name) => {
                directory.real.join(name)
            }
            _ => normalize_input_path(path),
        }
    }
}

impl Directory {
    fn read(path: &Path) -> Option<Self> {
        let real = vize_l0::path::normalize_windows_verbatim_path(path.canonicalize().ok()?);
        let files = std::fs::read_dir(&real)
            .ok()?
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
            .map(|entry| entry.file_name())
            .collect();
        Some(Self { real, files })
    }
}

#[cfg(test)]
mod tests {
    use super::ReferenceRealpaths;
    use crate::commands::check::tsconfig_inputs::glob::normalize_input_path;

    #[test]
    fn every_spelling_resolves_to_what_canonicalizing_it_whole_gives() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        std::fs::create_dir_all(root.join("pkg/sub")).unwrap();
        std::fs::create_dir_all(root.join("real")).unwrap();
        std::fs::write(root.join("pkg/a.d.ts"), "").unwrap();
        std::fs::write(root.join("pkg/sub/b.d.ts"), "").unwrap();
        std::fs::write(root.join("real/target.d.ts"), "").unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(root.join("real"), root.join("linked")).unwrap();
            std::os::unix::fs::symlink(root.join("real/target.d.ts"), root.join("pkg/link.d.ts"))
                .unwrap();
        }
        let mut realpaths = ReferenceRealpaths::default();
        for spelled in [
            "pkg/a.d.ts",
            "pkg/./a.d.ts",
            "pkg/sub/../a.d.ts",
            "pkg/sub/b.d.ts",
            "pkg/missing.d.ts",
            "pkg/sub",
            "pkg/sub/..",
            "pkg/A.D.TS",
            "linked/target.d.ts",
            "pkg/link.d.ts",
            "pkg/a.d.ts",
        ] {
            let path = root.join(spelled);
            assert_eq!(
                realpaths.resolve(&path),
                normalize_input_path(&path),
                "{spelled}"
            );
        }
    }
}
