//! One immutable editor overlay and memoized disk view shared by type loaders.

#![expect(
    clippy::disallowed_types,
    reason = "immutable editor snapshots share Arc source text without copying open documents"
)]

mod canonical_walk;
mod directory_index;
#[cfg(test)]
mod directory_index_tests;
#[cfg(target_os = "linux")]
mod held;
#[cfg(test)]
mod probe_tests;
mod public_identity;
mod realpath;
mod resolution;
#[cfg(test)]
mod resolution_tests;
#[cfg(all(test, unix))]
mod symlink_package_tests;
#[cfg(test)]
mod tests;

use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use vize_carton::{FxHashMap, FxHashSet, String};

use super::external_types::FileTypeSummary;
use super::type_world::ResolvedDependency;

type ResolvedCell = Arc<OnceLock<Option<Arc<ResolvedDependency>>>>;
type ResolutionCell = Arc<OnceLock<Option<PathBuf>>>;
type DirectoryCell = Arc<directory_index::ProbedDirectory>;
type PhysicalDirectoryCell = Arc<OnceLock<Option<PathBuf>>>;

#[derive(Debug, Hash, PartialEq, Eq)]
enum ResolutionScope {
    Directory(PathBuf),
    File(PathBuf),
}

#[derive(Debug, Default)]
pub struct TypeSourceSnapshot {
    project: Option<Arc<super::SfcTypeProject>>,
    overlays: FxHashMap<PathBuf, Arc<str>>,
    disk: Mutex<FxHashMap<PathBuf, Option<Arc<str>>>>,
    resolutions: Mutex<FxHashMap<(ResolutionScope, String), ResolutionCell>>,
    /// Candidate paths found to exist, with their identity. Importers in
    /// different directories probe the same modules. Misses are probed again.
    existing: Mutex<FxHashMap<PathBuf, PathBuf>>,
    /// Physical directories a probe hit, by the spelling it was reached
    /// through: their canonical path and the regular files listed once.
    directories: Mutex<FxHashMap<PathBuf, DirectoryCell>>,
    /// Canonical paths of the physical directories caller-supplied roots
    /// reached, by device and inode.
    #[cfg_attr(
        not(unix),
        expect(dead_code, reason = "Unix keys directories by inode")
    )]
    physical_directories: Mutex<FxHashMap<(u64, u64), PhysicalDirectoryCell>>,
    /// Directory paths known to be their own `realpath` in this snapshot.
    canonical_directories: Mutex<FxHashSet<PathBuf>>,
    /// Handles on the root, directories and files a request resolved.
    #[cfg(target_os = "linux")]
    held: held::HeldHandles,
    pub(super) summaries: Mutex<FxHashMap<PathBuf, FileTypeSummary>>,
    /// Dependency module facts, parsed once per snapshot with their targets
    /// resolved against it, plus the modules a world walk follows from them.
    /// Every world built from this snapshot shares them by `Arc`; root text
    /// never enters this cache.
    pub(super) resolved: Mutex<FxHashMap<PathBuf, ResolvedCell>>,
}

impl TypeSourceSnapshot {
    /// Bind a fresh source view to one immutable host-selected project. Cache
    /// cells belong to this snapshot and cannot leak between project revisions.
    pub fn with_project(
        overlays: impl IntoIterator<Item = (PathBuf, Arc<str>)>,
        project: Arc<super::SfcTypeProject>,
    ) -> Self {
        Self {
            project: Some(project),
            ..Self::new(overlays)
        }
    }

    /// A snapshot for one request: on Linux it holds the handles of the roots,
    /// directories and files it resolves until it drops, so a mount without a
    /// dentry cache walks each path once per request.
    pub fn for_request() -> Self {
        Self {
            #[cfg(target_os = "linux")]
            held: held::HeldHandles::holding(),
            ..Self::default()
        }
    }

    /// Capture editor sources once per editor revision. Cloning an Arc shares
    /// the immutable text; subsequent analyses borrow this same snapshot.
    pub fn new(overlays: impl IntoIterator<Item = (PathBuf, Arc<str>)>) -> Self {
        Self {
            overlays: overlays
                .into_iter()
                .map(|(path, source)| (source_path(&path), source))
                .collect(),
            ..Self::default()
        }
    }

    /// Overlay text wins over disk. Disk reads and misses are captured on first
    /// use, so compatibility props and the public type world read identical text.
    pub fn read(&self, path: &Path) -> Option<Arc<str>> {
        self.read_identity(source_path(path))
    }

    /// `realpath(path)`, holding the handle that proved a canonical spelling.
    pub(super) fn canonicalize_held(&self, path: &Path) -> Option<PathBuf> {
        #[cfg(target_os = "linux")]
        if let Some(handle) = realpath::canonical_handle(path) {
            self.held.hold(handle);
            return Some(path.to_path_buf());
        }
        std::fs::canonicalize(path).ok()
    }

    /// Read a source through an identity this snapshot already produced,
    /// without canonicalizing it again.
    pub(super) fn read_identity(&self, path: PathBuf) -> Option<Arc<str>> {
        if let Some(source) = self.overlays.get(&path) {
            return Some(Arc::clone(source));
        }
        let mut disk = self.disk.lock().ok()?;
        disk.entry(path.clone())
            .or_insert_with(|| std::fs::read_to_string(path).ok().map(Arc::from))
            .clone()
    }
}

/// Existing files use canonical identities; new unsaved files use the same
/// canonical parent identity followed by their lexically normalized path.
pub(super) fn source_path(path: &Path) -> PathBuf {
    if let Ok(canonical) = realpath::canonicalize(path) {
        return canonical;
    }
    let normalized = lexical_path(path);
    if let Some(parent) = normalized.parent()
        && let Ok(canonical_parent) = realpath::canonicalize(parent)
        && let Some(name) = normalized.file_name()
    {
        return canonical_parent.join(name);
    }
    normalized
}

/// The absolute path with `.` and `..` applied lexically.
pub(super) fn lexical_path(path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            component => normalized.push(component.as_os_str()),
        }
    }
    normalized
}
