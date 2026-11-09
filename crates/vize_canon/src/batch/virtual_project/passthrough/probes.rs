//! Directory listings shared by one build's passthrough probes.
//!
//! Resolving one relative import tries up to a dozen candidate names and
//! almost all of them are missing: a `stat` each, and a round trip each on a
//! WSL2 Windows mount. Each probed directory is listed once per build instead,
//! and a name it does not hold is missing. Links, case-insensitive near
//! misses, non-ASCII names and unreadable or very large directories keep the
//! direct `stat`.

use std::ffi::{OsStr, OsString};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use vize_carton::{FxHashMap, FxHashSet};

/// Entries beyond which a directory is probed name by name instead.
const MAX_LISTED_ENTRIES: usize = 4_096;

#[derive(Default)]
pub(in crate::batch::virtual_project) struct PassthroughProbes {
    directories: Mutex<FxHashMap<PathBuf, Listing>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Entry {
    File,
    Directory,
    Link,
    Other,
}

enum Listing {
    /// Missing or not a directory: nothing exists below it.
    Absent,
    /// Unreadable or too large: probe its names directly.
    Unlisted,
    Listed {
        entries: FxHashMap<OsString, Entry>,
        /// ASCII-lowercased names, to spot a case-insensitive near miss.
        folded: FxHashSet<Vec<u8>>,
        /// A non-ASCII name can fold or normalize onto an ASCII one.
        ascii: bool,
    },
}

#[derive(Debug, PartialEq, Eq)]
enum Lookup {
    Missing,
    Found(Entry),
    Unknown,
}

impl Listing {
    fn lookup(&self, name: &OsStr) -> Lookup {
        match self {
            Self::Absent => Lookup::Missing,
            Self::Unlisted => Lookup::Unknown,
            Self::Listed {
                entries,
                folded,
                ascii,
            } => {
                if let Some(entry) = entries.get(name) {
                    return Lookup::Found(*entry);
                }
                match name.to_str() {
                    Some(text) if *ascii && is_listable(text) => {
                        if folded.contains(&text.as_bytes().to_ascii_lowercase()) {
                            Lookup::Unknown
                        } else {
                            Lookup::Missing
                        }
                    }
                    _ => Lookup::Unknown,
                }
            }
        }
    }
}

/// Whether a listing can rule this name out. Windows also resolves short
/// (8.3) names, streams and trailing dots or spaces, so those keep `stat`.
fn is_listable(name: &str) -> bool {
    name.is_ascii() && !(cfg!(windows) && (name.contains(['~', ':']) || name.ends_with(['.', ' '])))
}

impl PassthroughProbes {
    /// `path.is_file()`, answered from its directory's listing when possible.
    pub(in crate::batch::virtual_project) fn is_file(&self, path: &Path) -> bool {
        match self.lookup(path) {
            Lookup::Missing | Lookup::Found(Entry::Directory | Entry::Other) => false,
            Lookup::Found(Entry::File) => true,
            Lookup::Found(Entry::Link) | Lookup::Unknown => path.is_file(),
        }
    }

    /// False only when the listing proves that nothing is named `path`.
    pub(in crate::batch::virtual_project) fn may_exist(&self, path: &Path) -> bool {
        self.lookup(path) != Lookup::Missing
    }

    fn lookup(&self, path: &Path) -> Lookup {
        let (Some(directory), Some(name)) = (path.parent(), path.file_name()) else {
            return Lookup::Unknown;
        };
        if directory.as_os_str().is_empty() {
            return Lookup::Unknown;
        }
        if let Some(found) = self.cached(directory, name) {
            return found;
        }
        let listing = self.list(directory);
        let found = listing.lookup(name);
        self.directories()
            .entry(directory.to_path_buf())
            .or_insert(listing);
        found
    }

    fn directories(&self) -> std::sync::MutexGuard<'_, FxHashMap<PathBuf, Listing>> {
        self.directories
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn cached(&self, directory: &Path, name: &OsStr) -> Option<Lookup> {
        self.directories()
            .get(directory)
            .map(|listing| listing.lookup(name))
    }

    fn list(&self, directory: &Path) -> Listing {
        // A directory its parent's listing already rules out needs no read.
        if let (Some(parent), Some(name)) = (directory.parent(), directory.file_name())
            && !parent.as_os_str().is_empty()
            && let Some(found) = self.cached(parent, name)
            && matches!(
                found,
                Lookup::Missing | Lookup::Found(Entry::File | Entry::Other)
            )
        {
            return Listing::Absent;
        }
        let entries = match std::fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(error)
                if matches!(error.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) =>
            {
                return Listing::Absent;
            }
            Err(_) => return Listing::Unlisted,
        };
        let mut listed = FxHashMap::default();
        let mut folded = FxHashSet::default();
        let mut ascii = true;
        for entry in entries {
            let Ok(entry) = entry else {
                return Listing::Unlisted;
            };
            if listed.len() == MAX_LISTED_ENTRIES {
                return Listing::Unlisted;
            }
            let Ok(file_type) = entry.file_type() else {
                return Listing::Unlisted;
            };
            let kind = if file_type.is_symlink() {
                Entry::Link
            } else if file_type.is_file() {
                Entry::File
            } else if file_type.is_dir() {
                Entry::Directory
            } else {
                Entry::Other
            };
            let name = entry.file_name();
            match name.to_str().filter(|text| text.is_ascii()) {
                Some(text) => {
                    folded.insert(text.as_bytes().to_ascii_lowercase());
                }
                None => ascii = false,
            }
            listed.insert(name, kind);
        }
        Listing::Listed {
            entries: listed,
            folded,
            ascii,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::PassthroughProbes;

    fn write(root: &Path, relative: &str) {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "export {}\n").unwrap();
    }

    #[test]
    fn listed_answers_match_stat() {
        let case = tempfile::tempdir().unwrap();
        let root = case.path();
        write(root, "src/model.ts");
        write(root, "src/Child.vue");
        write(root, "src/lib/index.js");
        std::fs::create_dir_all(root.join("src/empty")).unwrap();
        let probes = PassthroughProbes::default();
        for relative in [
            "src/model.ts",
            "src/model.js",
            "src/Child.vue",
            "src/Child.vue.js",
            "src/Child.vue/index.js",
            "src/Child.vue/package.json",
            "src/lib",
            "src/lib/index.js",
            "src/lib/index.ts",
            "src/empty",
            "src/missing/index.js",
            "src/../src/model.ts",
            "src/lib/../model.ts",
            "nowhere/deeper/name.js",
        ] {
            let path = root.join(relative);
            assert_eq!(probes.is_file(&path), path.is_file(), "{relative}");
            assert_eq!(probes.may_exist(&path), path.exists(), "{relative}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn links_follow_their_target() {
        let case = tempfile::tempdir().unwrap();
        let root = case.path();
        write(root, "real/model.ts");
        std::os::unix::fs::symlink(root.join("real/model.ts"), root.join("linked.ts")).unwrap();
        std::os::unix::fs::symlink(root.join("real"), root.join("dir")).unwrap();
        std::os::unix::fs::symlink(root.join("gone.ts"), root.join("dangling.ts")).unwrap();
        let probes = PassthroughProbes::default();
        assert!(probes.is_file(&root.join("linked.ts")));
        assert!(!probes.is_file(&root.join("dir")));
        assert!(probes.is_file(&root.join("dir/model.ts")));
        assert!(!probes.is_file(&root.join("dir/missing.ts")));
        assert!(!probes.is_file(&root.join("dangling.ts")));
        assert!(probes.may_exist(&root.join("dangling.ts")));
    }

    #[test]
    fn case_near_misses_and_non_ascii_names_keep_stat() {
        let case = tempfile::tempdir().unwrap();
        let root = case.path();
        write(root, "src/Model.ts");
        write(root, "intl/café.ts");
        let probes = PassthroughProbes::default();
        // Exact on a case-sensitive volume, found on a case-insensitive one.
        let near = root.join("src/model.ts");
        assert_eq!(probes.is_file(&near), near.is_file());
        let accented = root.join("intl/cafe\u{301}.ts");
        assert_eq!(probes.is_file(&accented), accented.is_file());
        let other = root.join("intl/plain.ts");
        assert_eq!(probes.is_file(&other), other.is_file());
    }

    #[test]
    fn each_directory_is_listed_once() {
        let case = tempfile::tempdir().unwrap();
        let root = case.path();
        write(root, "src/model.ts");
        let probes = PassthroughProbes::default();
        for extension in ["js", "jsx", "mjs", "cjs", "json"] {
            let candidate = root.join(vize_carton::cstr!("src/model.{extension}").as_str());
            assert!(!probes.is_file(&candidate));
        }
        assert_eq!(probes.directories().len(), 1);
        // A child of a listed regular file is ruled out without a read.
        assert!(!probes.is_file(&root.join("src/model.ts/index.js")));
        assert!(matches!(
            probes.directories().get(&root.join("src/model.ts")),
            Some(super::Listing::Absent)
        ));
    }
}
