//! Filesystem identity for self-invalidating package-route caches.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use vize_carton::FxHashMap;

#[derive(Clone, Debug, Eq)]
pub(crate) struct InputStamp {
    path: PathBuf,
    modified: Option<std::time::SystemTime>,
    len: Option<u64>,
    kind: Option<InputKind>,
    content_digest: Option<[u8; 32]>,
    /// Device, inode and status-change time of an installed file. Any write
    /// moves its ctime even when mtime and size are restored, so installed
    /// packages need no content hash on every editor revision.
    installed_identity: Option<(u64, u64, i64, i64)>,
    symlink_target: Option<PathBuf>,
    source_content: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InputKind {
    File,
    Directory,
    Symlink,
    Other,
}

/// Local counters for one opt-in validation; never shared across calls.
#[derive(Default)]
pub(crate) struct InputStampReadStats {
    pub(crate) reads: usize,
    pub(crate) bytes: u64,
    pub(crate) read_time: Duration,
    pub(crate) digest_time: Duration,
}

#[derive(Default)]
pub(crate) struct InputStampCache {
    snapshots: FxHashMap<PathBuf, InputStamp>,
    #[cfg(test)]
    captures: usize,
}

impl InputStampCache {
    pub(crate) fn clear(&mut self) {
        self.snapshots.clear();
        #[cfg(test)]
        {
            self.captures = 0;
        }
    }

    pub(crate) fn capture(&mut self, path: &Path) -> InputStamp {
        if let Some(stamp) = self.snapshots.get(path) {
            return stamp.clone();
        }
        #[cfg(test)]
        {
            self.captures += 1;
        }
        let stamp = InputStamp::capture(path);
        self.snapshots.insert(path.to_path_buf(), stamp.clone());
        stamp
    }

    #[cfg(test)]
    pub(crate) fn captures(&self) -> usize {
        self.captures
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.snapshots.len()
    }
}

/// Fresh physical snapshots shared only within one configuration validation.
/// Every retained expected revision is compared; no expected stamp seeds this map.
#[cfg(feature = "native")]
#[derive(Default)]
pub(crate) struct InputStampValidation {
    current: FxHashMap<(PathBuf, bool), InputStamp>,
}

#[cfg(feature = "native")]
impl InputStampValidation {
    pub(crate) fn is_current(
        &mut self,
        expected: &InputStamp,
        stats: Option<&mut InputStampReadStats>,
    ) -> bool {
        let (path, source_content) = expected.observation_identity();
        let current = self
            .current
            .entry((path.to_path_buf(), source_content))
            .or_insert_with(|| {
                InputStamp::capture_input_observed(path.to_path_buf(), None, source_content, stats)
            });
        expected == &*current
    }

    pub(crate) fn len(&self) -> usize {
        self.current.len()
    }
}

impl InputStamp {
    pub(crate) fn capture(path: impl Into<PathBuf>) -> Self {
        Self::capture_input(path.into(), None, false)
    }

    /// Keep the bytes registered in the semantic project as the authority.
    /// Re-reading disk here could acknowledge an edit the project never saw.
    pub(crate) fn capture_source(path: impl Into<PathBuf>, content: &[u8]) -> Self {
        Self::capture_input(path.into(), Some(content), true)
    }

    fn capture_input(path: PathBuf, content: Option<&[u8]>, source_content: bool) -> Self {
        Self::capture_input_observed(path, content, source_content, None)
    }

    fn capture_input_observed(
        path: PathBuf,
        content: Option<&[u8]>,
        source_content: bool,
        stats: Option<&mut InputStampReadStats>,
    ) -> Self {
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
        // Installed package files are identified by metadata: an editor
        // revision may consult ~10^5 of them, and hashing every byte on each
        // edit dominated latency. Authored sources keep content digests.
        // Manifests and configs stay hashed: they are few and decide routes.
        let installed_identity =
            (content.is_none() && !source_content && matches!(kind, Some(InputKind::File)))
                .then(|| metadata.as_ref().and_then(installed_identity))
                .flatten()
                .filter(|_| is_installed_path(&path) && !is_route_manifest(&path));
        // A file changed within the timestamp granularity can share size,
        // mtime and ctime with its previous bytes ("racy" metadata, as in Git),
        // so a recent installed file is hashed as well.
        let racy = installed_identity.is_some() && metadata.as_ref().is_none_or(changed_recently);
        let content_digest = content.map(digest).or_else(|| {
            ((matches!(kind, Some(InputKind::File))
                && (installed_identity.is_none() || racy)
                && !super::graph_inputs::is_large_lockfile(&path))
                || (source_content && matches!(kind, Some(InputKind::Symlink))))
            .then(|| read_digest(&path, stats))
            .flatten()
        });
        let symlink_target = matches!(kind, Some(InputKind::Symlink))
            .then(|| std::fs::read_link(&path).ok())
            .flatten();
        Self {
            path,
            modified,
            len,
            kind,
            content_digest,
            installed_identity,
            symlink_target,
            source_content,
        }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn is_current(&self) -> bool {
        *self == Self::capture_input(self.path.clone(), None, self.source_content)
    }

    #[cfg(feature = "native")]
    pub(crate) fn observation_identity(&self) -> (&Path, bool) {
        (&self.path, self.source_content)
    }

    pub(crate) fn is_current_with_cache(&self, cache: &mut InputStampCache) -> bool {
        // Route-cache epochs hold filesystem stamps, not registered-source stamps.
        if self.source_content {
            return self.is_current();
        }
        *self == cache.capture(&self.path)
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

fn read_digest(path: &Path, stats: Option<&mut InputStampReadStats>) -> Option<[u8; 32]> {
    let Some(stats) = stats else {
        return std::fs::read(path).ok().map(|content| digest(&content));
    };
    let started = Instant::now();
    let content = std::fs::read(path);
    stats.read_time += started.elapsed();
    stats.reads += 1;
    let content = content.ok()?;
    stats.bytes += content.len() as u64;
    let started = Instant::now();
    let result = digest(&content);
    stats.digest_time += started.elapsed();
    Some(result)
}

impl PartialEq for InputStamp {
    fn eq(&self, other: &Self) -> bool {
        // An installed file hashed only while its metadata was racy compares by
        // content when both sides hashed it and by metadata otherwise.
        let digests_agree = match (&self.content_digest, &other.content_digest) {
            (Some(left), Some(right)) => left == right,
            (None, None) => true,
            _ => self.installed_identity.is_some() && other.installed_identity.is_some(),
        };
        digests_agree
            && self.path == other.path
            && self.modified == other.modified
            && self.len == other.len
            && self.kind == other.kind
            && self.installed_identity == other.installed_identity
            && self.symlink_target == other.symlink_target
            && self.source_content == other.source_content
    }
}

/// Package manifests and compiler configs keep content digests.
fn is_route_manifest(path: &Path) -> bool {
    path.file_name()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|name| {
            name == "package.json"
                || ((name.starts_with("tsconfig") || name.starts_with("jsconfig"))
                    && name.ends_with(".json"))
        })
}

/// Within this window, metadata alone may not see a same-size rewrite.
const RACY_WINDOW: Duration = Duration::from_secs(2);

fn changed_recently(metadata: &std::fs::Metadata) -> bool {
    let now = std::time::SystemTime::now();
    let recent = |time: std::io::Result<std::time::SystemTime>| {
        time.ok().is_none_or(|time| {
            now.duration_since(time)
                .map_or(true, |age| age < RACY_WINDOW)
        })
    };
    recent(metadata.modified()) || recent(status_changed(metadata))
}

#[cfg(unix)]
fn status_changed(metadata: &std::fs::Metadata) -> std::io::Result<std::time::SystemTime> {
    use std::os::unix::fs::MetadataExt;
    let seconds = u64::try_from(metadata.ctime()).map_err(std::io::Error::other)?;
    let nanos = u32::try_from(metadata.ctime_nsec()).map_err(std::io::Error::other)?;
    Ok(std::time::UNIX_EPOCH + Duration::new(seconds, nanos))
}

#[cfg(not(unix))]
fn status_changed(metadata: &std::fs::Metadata) -> std::io::Result<std::time::SystemTime> {
    metadata.modified()
}

fn is_installed_path(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == "node_modules")
}

#[cfg(unix)]
fn installed_identity(metadata: &std::fs::Metadata) -> Option<(u64, u64, i64, i64)> {
    use std::os::unix::fs::MetadataExt;
    Some((
        metadata.dev(),
        metadata.ino(),
        metadata.ctime(),
        metadata.ctime_nsec(),
    ))
}

/// Without a status-change time, installed files keep content digests.
#[cfg(not(unix))]
fn installed_identity(_metadata: &std::fs::Metadata) -> Option<(u64, u64, i64, i64)> {
    None
}

// Private, process-local change detection, never a persisted/public artifact key.
// Hash every byte; sha2 selects hardware acceleration when the target supports it.
fn digest(content: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    Sha256::digest(content).into()
}

pub(super) fn stamp_paths(paths: &[PathBuf]) -> Vec<InputStamp> {
    paths.iter().cloned().map(InputStamp::capture).collect()
}

pub(super) fn stamp_paths_with_cache(
    paths: &[PathBuf],
    cache: &mut InputStampCache,
) -> Vec<InputStamp> {
    paths.iter().map(|path| cache.capture(path)).collect()
}

pub(super) fn stamps_are_current(stamps: &[InputStamp]) -> bool {
    stamps.iter().all(InputStamp::is_current)
}

pub(super) fn stamps_are_current_with_cache(
    stamps: &[InputStamp],
    cache: &mut InputStampCache,
) -> bool {
    stamps
        .iter()
        .all(|stamp| stamp.is_current_with_cache(cache))
}

pub(super) fn manifest_path(root: &Path) -> PathBuf {
    root.join("package.json")
}

#[cfg(test)]
mod tests;
