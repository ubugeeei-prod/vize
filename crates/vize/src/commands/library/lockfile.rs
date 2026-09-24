//! `vize-lib.lock.json`: what was pulled, from which package version, and the
//! SHA-256 of every file at pull time. The recorded digests are the merge base
//! that lets `status`, `diff`, and `update` tell local edits from upstream ones.
//!
//! Schema: `npm/cli/schemas/vize-lib-lock.schema.json`.

use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use serde::{Deserialize, Serialize};
use vize_s0::{String, cstr};

use super::error::{LibError, LibResult};
use super::fs_ops::{ensure_project_path, write_file};

/// Lockfile format version.
pub const LOCKFILE_VERSION: u32 = 1;

/// Default lockfile name, relative to the project root.
pub const DEFAULT_LOCKFILE: &str = "vize-lib.lock.json";

/// Root of `vize-lib.lock.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lockfile {
    pub lockfile_version: u32,
    /// Pulled items sorted by `(kind, name)`.
    pub items: Vec<LockedItem>,
}

impl Default for Lockfile {
    fn default() -> Self {
        Self {
            lockfile_version: LOCKFILE_VERSION,
            items: Vec::new(),
        }
    }
}

/// One pulled item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockedItem {
    pub name: String,
    pub kind: String,
    /// npm package that published the pulled version.
    pub package: String,
    /// Exact package version the files were pulled from.
    pub version: String,
    /// Registry `contentHash` of the pulled version.
    pub content_hash: String,
    /// Project-relative POSIX directory the registry layout was copied into.
    pub dir: String,
    /// `true` when requested by name; `false` when pulled only as a dependency.
    pub direct: bool,
    /// Registry dependency closure at pull time.
    pub registry_dependencies: Vec<String>,
    /// Registry-relative path -> SHA-256 of the bytes written at pull time.
    pub files: BTreeMap<String, String>,
}

impl LockedItem {
    /// Display form: `ui:switch` or `@acme/button`.
    pub fn label(&self) -> String {
        if self.kind.starts_with('@') {
            cstr!("{}/{}", self.kind, self.name)
        } else {
            cstr!("{}:{}", self.kind, self.name)
        }
    }
}

impl Lockfile {
    /// Read the lockfile, or an empty one when it does not exist.
    pub fn read(root: &Path, path: &Path) -> LibResult<Self> {
        ensure_project_path(root, path)?;
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(LibError::io("read", path, &error)),
        };
        let lockfile: Self = serde_json::from_slice(&bytes)
            .map_err(|error| LibError::new(cstr!("invalid {}: {error}", path.display())))?;
        if lockfile.lockfile_version != LOCKFILE_VERSION {
            return Err(LibError::new(cstr!(
                "{} has lockfileVersion {}; this vize understands {LOCKFILE_VERSION}",
                path.display(),
                lockfile.lockfile_version
            )));
        }
        Ok(lockfile)
    }

    /// Serialize deterministically (sorted, 2-space JSON, trailing newline).
    pub fn to_bytes(&self) -> LibResult<Vec<u8>> {
        let mut sorted = self.clone();
        sorted.items.sort_by(|left, right| {
            (left.kind.as_str(), left.name.as_str())
                .cmp(&(right.kind.as_str(), right.name.as_str()))
        });
        let mut bytes = serde_json::to_vec_pretty(&sorted)
            .map_err(|error| LibError::new(cstr!("failed to serialize lockfile: {error}")))?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    /// Atomically write the lockfile, or delete it when no items remain.
    pub fn write(&self, root: &Path, path: &Path) -> LibResult<()> {
        ensure_project_path(root, path)?;
        if self.items.is_empty() {
            return match fs::remove_file(path) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
                Err(error) => Err(LibError::io("remove", path, &error)),
            };
        }
        write_file(path, &self.to_bytes()?)
    }

    pub fn get(&self, kind: &str, name: &str) -> Option<&LockedItem> {
        self.items
            .iter()
            .find(|item| item.kind == kind && item.name == name)
    }

    /// Insert or replace the entry with the same `(kind, name)`.
    pub fn upsert(&mut self, item: LockedItem) {
        self.items
            .retain(|existing| !(existing.kind == item.kind && existing.name == item.name));
        self.items.push(item);
    }

    pub fn remove(&mut self, kind: &str, name: &str) {
        self.items
            .retain(|item| !(item.kind == kind && item.name == name));
    }

    /// Locked items whose name or `kind:name` matches `query`.
    pub fn find(&self, query: &str) -> Vec<&LockedItem> {
        let split = if query.starts_with('@') {
            query.split_once('/')
        } else {
            query.split_once(':')
        };
        let (kind, name) = match split {
            Some((kind, name)) => (Some(kind), name),
            None => (None, query),
        };
        self.items
            .iter()
            .filter(|item| item.name == name && kind.is_none_or(|kind| item.kind == kind))
            .collect()
    }

    /// Locked items of the same source that depend on `name`.
    pub fn dependents<'a>(&'a self, kind: &str, name: &str) -> Vec<&'a LockedItem> {
        self.items
            .iter()
            .filter(|item| {
                item.kind == kind
                    && item.name != name
                    && item
                        .registry_dependencies
                        .iter()
                        .any(|dependency| dependency == name)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{LockedItem, Lockfile};
    use std::collections::BTreeMap;

    fn item(kind: &str, name: &str) -> LockedItem {
        LockedItem {
            name: name.into(),
            kind: kind.into(),
            package: "@vizejs/ui".into(),
            version: "1.0.0".into(),
            content_hash: "0".repeat(64).into(),
            dir: "src/components/vize".into(),
            direct: true,
            registry_dependencies: Vec::new(),
            files: BTreeMap::new(),
        }
    }

    #[test]
    fn serializes_sorted_and_round_trips() {
        let mut lockfile = Lockfile::default();
        lockfile.upsert(item("ui", "switch"));
        lockfile.upsert(item("composable", "use-toggle"));
        lockfile.upsert(item("ui", "id"));
        let bytes = lockfile.to_bytes().unwrap();
        let parsed: Lockfile = serde_json::from_slice(&bytes).unwrap();
        let names: Vec<&str> = parsed.items.iter().map(|item| item.name.as_str()).collect();
        assert_eq!(names, ["use-toggle", "id", "switch"]);
        assert!(bytes.ends_with(b"}\n"));
    }

    #[test]
    fn finds_by_name_or_kind_prefixed_name() {
        let mut lockfile = Lockfile::default();
        lockfile.upsert(item("ui", "locale"));
        lockfile.upsert(item("composable", "locale"));
        assert_eq!(lockfile.find("locale").len(), 2);
        assert_eq!(lockfile.find("composable:locale").len(), 1);
    }
}
