//! Filesystem identity for self-invalidating package-route caches.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use vize_carton::FxHashMap;

#[derive(Clone, Debug, Eq, PartialEq)]
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
        let installed_identity =
            (content.is_none() && !source_content && matches!(kind, Some(InputKind::File)))
                .then(|| metadata.as_ref().and_then(installed_identity))
                .flatten()
                .filter(|_| is_installed_path(&path));
        let content_digest = content.map(digest).or_else(|| {
            ((matches!(kind, Some(InputKind::File))
                && installed_identity.is_none()
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
mod tests {
    use super::{InputStamp, InputStampCache};
    #[cfg(feature = "native")]
    use super::{InputStampReadStats, InputStampValidation};

    #[test]
    fn stamp_cache_captures_each_path_once_per_epoch() {
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("package.json");
        std::fs::write(&file, r#"{"name":"pkg"}"#).unwrap();
        let stamp = InputStamp::capture(&file);
        let mut cache = InputStampCache::default();

        assert!(stamp.is_current_with_cache(&mut cache));
        assert!(stamp.is_current_with_cache(&mut cache));
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.captures(), 1);
    }

    #[test]
    fn registered_source_stamps_do_not_reuse_filesystem_cache_authority() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("source.ts");
        let content = b"export const value = 1;";
        std::fs::write(&path, content).unwrap();
        let stamp = InputStamp::capture_source(&path, content);
        let mut cache = InputStampCache::default();
        cache.capture(&path);
        assert!(stamp.is_current());
        assert!(stamp.is_current_with_cache(&mut cache));
        std::fs::write(&path, "export const value = 2;").unwrap();
        assert!(!stamp.is_current());
        assert!(!stamp.is_current_with_cache(&mut cache));
        assert_eq!(cache.captures(), 1);
    }

    #[test]
    fn same_size_and_mtime_edits_still_change_full_content_stamps() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.json");
        let mut content = vec![b' '; 8193];
        std::fs::write(&path, &content).unwrap();
        let physical = InputStamp::capture(&path);
        let source = InputStamp::capture_source(&path, &content);
        let original = std::fs::metadata(&path).unwrap();
        #[cfg(feature = "native")]
        let observed = crate::ConfigurationObservations::capture(std::slice::from_ref(&path));
        #[cfg(feature = "native")]
        assert!(observed.changed_paths().is_empty());
        assert!(physical.is_current());
        assert!(source.is_current());

        // Preserve metadata and the complete prefix; only the final byte changes.
        *content.last_mut().unwrap() = b'!';
        std::fs::write(&path, &content).unwrap();
        std::fs::File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_times(std::fs::FileTimes::new().set_modified(original.modified().unwrap()))
            .unwrap();
        let modified = std::fs::metadata(&path).unwrap();
        assert_eq!(modified.len(), original.len());
        assert_eq!(modified.modified().unwrap(), original.modified().unwrap());
        assert!(!physical.is_current());
        assert!(!source.is_current());
        #[cfg(feature = "native")]
        assert_eq!(observed.changed_paths(), vec![path.clone()]);
        assert!(InputStamp::capture(&path).is_current());
        assert!(InputStamp::capture_source(&path, &content).is_current());
    }

    #[cfg(unix)]
    #[test]
    fn source_stamps_track_symlink_content_and_target_without_becoming_always_dirty() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("target.ts");
        let link = root.path().join("source.ts");
        let content = b"export const value = 1;";
        std::fs::write(&target, content).unwrap();
        std::os::unix::fs::symlink(&target, &link).unwrap();
        let stamp = InputStamp::capture_source(&link, content);
        assert!(stamp.is_current());
        std::fs::write(&target, "export const value = 2;").unwrap();
        assert!(!stamp.is_current());
        let other = root.path().join("other.ts");
        std::fs::write(&other, content).unwrap();
        std::fs::remove_file(&link).unwrap();
        std::os::unix::fs::symlink(&other, &link).unwrap();
        assert!(!stamp.is_current());
        assert!(InputStamp::capture_source(&link, content).is_current());
    }

    #[cfg(feature = "native")]
    #[test]
    fn validation_compares_every_consumed_revision_to_fresh_bytes_in_both_orders() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("source.ts");
        std::fs::write(&path, b"omega").unwrap();
        let stale = InputStamp::capture_source(&path, b"alpha");
        let current = InputStamp::capture_source(&path, b"omega");
        for (stamps, expected) in [
            ([&stale, &current], [false, true]),
            ([&current, &stale], [true, false]),
        ] {
            let mut validation = InputStampValidation::default();
            let mut stats = InputStampReadStats::default();
            let actual = stamps.map(|stamp| validation.is_current(stamp, Some(&mut stats)));
            assert_eq!(actual, expected);
            assert_eq!(validation.len(), 1);
            assert_eq!(stats.reads, 1);
            assert_eq!(stats.bytes, 5);
        }
    }

    #[cfg(feature = "native")]
    #[test]
    fn validation_does_not_reuse_absence_across_calls() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("absent.json");
        let observed = crate::ConfigurationObservations::capture(std::slice::from_ref(&path));
        assert!(observed.changed_paths().is_empty());
        std::fs::write(&path, b"{}").unwrap();
        assert_eq!(observed.changed_paths(), vec![path]);
    }

    #[test]
    fn content_identity_retains_the_complete_standard_sha256_digest() {
        assert_eq!(
            super::digest(b""),
            [
                0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f,
                0xb9, 0x24, 0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99, 0x1b,
                0x78, 0x52, 0xb8, 0x55,
            ],
        );
        assert_eq!(
            super::digest(b"abc"),
            [
                0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
                0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
                0xf2, 0x00, 0x15, 0xad,
            ],
        );
    }

    /// Installed files are stamped by metadata, never read; a same-size,
    /// same-mtime rewrite still moves their status-change time. Authored
    /// sources keep a content digest.
    #[cfg(unix)]
    #[test]
    fn installed_files_use_metadata_identity_and_still_see_same_mtime_rewrites() {
        let root = tempfile::tempdir().unwrap();
        let installed = root.path().join("node_modules/pkg/index.d.ts");
        let authored = root.path().join("src/index.ts");
        for path in [&installed, &authored] {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, "export const a = 1;\n").unwrap();
        }
        let stamp = super::InputStamp::capture(&installed);
        assert!(stamp.content_digest.is_none());
        assert!(stamp.installed_identity.is_some());
        assert!(
            super::InputStamp::capture(&authored)
                .content_digest
                .is_some()
        );

        let modified = std::fs::metadata(&installed).unwrap().modified().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        std::fs::write(&installed, "export const b = 2;\n").unwrap();
        std::fs::File::options()
            .write(true)
            .open(&installed)
            .unwrap()
            .set_times(std::fs::FileTimes::new().set_modified(modified))
            .unwrap();
        assert!(!stamp.is_current());
    }
}
