//! One immutable editor overlay and memoized disk view shared by type loaders.

#![expect(
    clippy::disallowed_types,
    reason = "immutable editor snapshots share Arc source text without copying open documents"
)]

mod resolution;
#[cfg(test)]
mod resolution_tests;
#[cfg(all(test, unix))]
mod symlink_package_tests;
#[cfg(test)]
mod tests;

use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use vize_carton::{FxHashMap, String};

use super::external_types::FileTypeSummary;
use super::type_world::ParsedTypeModule;

type ModuleCell = Arc<OnceLock<Option<ParsedTypeModule>>>;
type ResolutionCell = Arc<OnceLock<Option<PathBuf>>>;

#[derive(Debug, Hash, PartialEq, Eq)]
enum ResolutionScope {
    Directory(PathBuf),
    File(PathBuf),
}

#[derive(Debug, Default)]
pub struct TypeSourceSnapshot {
    overlays: FxHashMap<PathBuf, Arc<str>>,
    disk: Mutex<FxHashMap<PathBuf, Option<Arc<str>>>>,
    resolutions: Mutex<FxHashMap<(ResolutionScope, String), ResolutionCell>>,
    pub(super) summaries: Mutex<FxHashMap<PathBuf, FileTypeSummary>>,
    /// Unresolved dependency facts; each world resolves targets on its own clone.
    pub(super) modules: Mutex<FxHashMap<PathBuf, ModuleCell>>,
}

impl TypeSourceSnapshot {
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
        let path = source_path(path);
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
    if let Ok(canonical) = path.canonicalize() {
        return canonical;
    }
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
    if let Some(parent) = normalized.parent()
        && let Ok(canonical_parent) = parent.canonicalize()
        && let Some(name) = normalized.file_name()
    {
        return canonical_parent.join(name);
    }
    normalized
}
