//! Filesystem identity for self-invalidating package-route caches.

use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InputStamp {
    path: PathBuf,
    modified: Option<std::time::SystemTime>,
    len: Option<u64>,
    kind: Option<InputKind>,
    content_digest: Option<u64>,
    symlink_target: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InputKind {
    File,
    Directory,
    Symlink,
    Other,
}

impl InputStamp {
    pub(crate) fn capture(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let metadata = std::fs::symlink_metadata(&path).ok();
        let modified = metadata
            .as_ref()
            .and_then(|metadata| metadata.modified().ok());
        let len = metadata.as_ref().map(std::fs::Metadata::len);
        let kind = metadata.as_ref().map(|metadata| {
            let file_type = metadata.file_type();
            if file_type.is_file() {
                InputKind::File
            } else if file_type.is_dir() {
                InputKind::Directory
            } else if file_type.is_symlink() {
                InputKind::Symlink
            } else {
                InputKind::Other
            }
        });
        // Large lockfiles are graph-change triggers, never route authorities.
        // Metadata keeps warm lookup O(depth) without rehashing megabytes;
        // actual package links/manifests and source inputs retain strong
        // same-mtime content stamps.
        let content_digest = (matches!(kind, Some(InputKind::File))
            && !super::graph_inputs::is_large_lockfile(&path))
        .then(|| std::fs::read(&path).ok().map(|content| digest(&content)))
        .flatten();
        let symlink_target = matches!(kind, Some(InputKind::Symlink))
            .then(|| std::fs::read_link(&path).ok())
            .flatten();
        Self {
            path,
            modified,
            len,
            kind,
            content_digest,
            symlink_target,
        }
    }

    pub(crate) fn is_current(&self) -> bool {
        *self == Self::capture(&self.path)
    }
}

/// Canonicalize a changed path even after its final components were deleted.
///
/// Watch notifications commonly arrive after a manifest or symlink target has
/// disappeared. `Path::canonicalize` cannot resolve that leaf, but route input
/// stamps were captured through its canonical physical spelling. Resolve the
/// nearest existing ancestor and append the missing suffix so the reverse
/// invalidation index keeps the same identity across delete/recreate events.
#[cfg(feature = "native")]
pub(crate) fn canonicalize_changed_path(path: &Path) -> PathBuf {
    if let Ok(canonical) = path.canonicalize() {
        return vize_carton::path::normalize_windows_verbatim_path(canonical);
    }

    let mut ancestor = path;
    let mut suffix = Vec::new();
    while let Some(name) = ancestor.file_name() {
        suffix.push(name.to_owned());
        let Some(parent) = ancestor.parent() else {
            break;
        };
        ancestor = parent;
        if let Ok(mut canonical) = ancestor.canonicalize() {
            for component in suffix.iter().rev() {
                canonical.push(component);
            }
            return vize_carton::path::normalize_windows_verbatim_path(canonical);
        }
    }
    vize_carton::path::normalize_windows_verbatim_path(path.to_path_buf())
}

fn digest(content: &[u8]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::hash::DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

pub(super) fn stamp_paths(paths: &[PathBuf]) -> Vec<InputStamp> {
    paths.iter().cloned().map(InputStamp::capture).collect()
}

pub(super) fn stamps_are_current(stamps: &[InputStamp]) -> bool {
    stamps.iter().all(InputStamp::is_current)
}

pub(super) fn manifest_path(root: &Path) -> PathBuf {
    root.join("package.json")
}
