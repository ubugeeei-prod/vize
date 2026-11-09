//! Re-observe unchanged Canon files without reopening their complete contents.
//!
//! Only settled regular-file metadata with an unforgeable Unix change time can
//! certify a previous exact read. Recent files and other platforms keep the
//! original full read; desired text is always compared by its strong digest.

#[cfg(test)]
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
#[cfg(any(unix, test))]
use std::time::Duration;
use std::time::SystemTime;
#[cfg(unix)]
use std::time::UNIX_EPOCH;
use vize_l0::FxHashMap;

#[derive(Default)]
pub(super) struct CanonFileCache {
    files: FxHashMap<PathBuf, Observation>,
    #[cfg(test)]
    reads: usize,
}

struct Observation {
    identity: FileIdentity,
    desired: [u8; 32],
}

#[cfg(unix)]
#[derive(Clone, PartialEq, Eq)]
struct FileIdentity {
    device: u64,
    inode: u64,
    length: u64,
    modified: (i64, i64),
    changed: (i64, i64),
}

#[cfg(not(unix))]
#[derive(Clone, PartialEq, Eq)]
struct FileIdentity;

impl FileIdentity {
    #[cfg(unix)]
    fn capture(path: &Path) -> Option<Self> {
        Self::from_metadata(&std::fs::symlink_metadata(path).ok()?)
    }

    #[cfg(not(unix))]
    fn capture(_path: &Path) -> Option<Self> {
        None
    }

    /// The identity of an `lstat` already taken for this path.
    #[cfg(unix)]
    fn from_metadata(metadata: &std::fs::Metadata) -> Option<Self> {
        use std::os::unix::fs::MetadataExt;
        if !metadata.file_type().is_file() || metadata.ctime() <= 0 || metadata.ino() == 0 {
            return None;
        }
        Some(Self {
            device: metadata.dev(),
            inode: metadata.ino(),
            length: metadata.len(),
            modified: (metadata.mtime(), metadata.mtime_nsec()),
            changed: (metadata.ctime(), metadata.ctime_nsec()),
        })
    }

    #[cfg(not(unix))]
    fn from_metadata(_metadata: &std::fs::Metadata) -> Option<Self> {
        None
    }

    #[cfg(unix)]
    fn settled(&self, now: SystemTime) -> bool {
        let Some(fence) = now.checked_sub(Duration::from_secs(1)) else {
            return false;
        };
        [self.modified, self.changed]
            .into_iter()
            .all(|(seconds, nanos)| {
                let Ok(seconds) = u64::try_from(seconds) else {
                    return false;
                };
                let Ok(nanos) = u32::try_from(nanos) else {
                    return false;
                };
                nanos < 1_000_000_000
                    && UNIX_EPOCH
                        .checked_add(Duration::new(seconds, nanos))
                        .is_some_and(|time| time < fence)
            })
    }

    #[cfg(not(unix))]
    fn settled(&self, _now: SystemTime) -> bool {
        false
    }
}

impl CanonFileCache {
    #[cfg(test)]
    pub(super) fn exact_reads_for_test(&self) -> usize {
        self.reads
    }

    /// Only an unchanged immutable batch and retained-text checkpoint may
    /// supply its previously captured desired digest without scanning text.
    /// `metadata` is this path's own `lstat` when the caller already took it.
    pub(super) fn observation_is_current(
        &self,
        path: &Path,
        desired: [u8; 32],
        now: SystemTime,
        metadata: Option<&std::fs::Metadata>,
    ) -> bool {
        self.files.get(path).is_some_and(|old| {
            old.desired == desired
                && metadata
                    .map_or_else(|| FileIdentity::capture(path), FileIdentity::from_metadata)
                    .is_some_and(|identity| identity.settled(now) && old.identity == identity)
        })
    }

    pub(super) fn clear(&mut self) {
        self.files.clear();
    }

    pub(super) fn forget(&mut self, path: &Path) {
        self.files.remove(path);
    }

    #[cfg(test)]
    pub(super) fn synchronize_with_digest(
        &mut self,
        path: &Path,
        expected: &str,
        desired: impl FnOnce() -> [u8; 32],
    ) -> std::io::Result<bool> {
        self.synchronize_at_with_digest(path, expected, SystemTime::now(), desired)
    }

    /// [`Self::synchronize_with_digest`] starting from this path's own
    /// `lstat`, already taken by the caller.
    pub(super) fn synchronize_observed_with_digest(
        &mut self,
        path: &Path,
        expected: &str,
        metadata: Option<&std::fs::Metadata>,
        desired: impl FnOnce() -> [u8; 32],
    ) -> std::io::Result<bool> {
        let before =
            metadata.map_or_else(|| FileIdentity::capture(path), FileIdentity::from_metadata);
        self.synchronize_from(path, expected, SystemTime::now(), before, desired)
    }

    #[cfg(test)]
    fn synchronize_at_with_digest(
        &mut self,
        path: &Path,
        expected: &str,
        now: SystemTime,
        desired: impl FnOnce() -> [u8; 32],
    ) -> std::io::Result<bool> {
        self.synchronize_from(path, expected, now, FileIdentity::capture(path), desired)
    }

    fn synchronize_from(
        &mut self,
        path: &Path,
        expected: &str,
        now: SystemTime,
        before: Option<FileIdentity>,
        desired: impl FnOnce() -> [u8; 32],
    ) -> std::io::Result<bool> {
        let desired = before
            .as_ref()
            .filter(|identity| identity.settled(now))
            .map(|_| desired());
        if let (Some(identity), Some(desired)) = (before.as_ref(), desired)
            && self
                .files
                .get(path)
                .is_some_and(|old| old.identity == *identity && old.desired == desired)
        {
            return Ok(false);
        }
        #[cfg(test)]
        {
            self.reads += 1;
        }
        if std::fs::read_to_string(path).ok().as_deref() != Some(expected) {
            self.files.remove(path);
            std::fs::write(path, expected)?;
            // A write alone cannot acknowledge bytes written concurrently by
            // someone else. Only a later stable, exact read can seed the cache.
            return Ok(true);
        }
        if let Some(desired) = desired {
            let after = FileIdentity::capture(path);
            self.retain_verified_observation(path, before, after, desired, now);
        } else {
            // Unsupported identities and recent files retain the original
            // full read without paying for an unusable content digest.
            self.files.remove(path);
        }
        Ok(false)
    }

    #[cfg(test)]
    fn synchronize(&mut self, path: &Path, expected: &str) -> std::io::Result<bool> {
        self.synchronize_at(path, expected, SystemTime::now())
    }

    #[cfg(test)]
    fn synchronize_at(
        &mut self,
        path: &Path,
        expected: &str,
        now: SystemTime,
    ) -> std::io::Result<bool> {
        self.synchronize_at_with_digest(path, expected, now, || {
            Sha256::digest(expected.as_bytes()).into()
        })
    }
    fn retain_verified_observation(
        &mut self,
        path: &Path,
        before: Option<FileIdentity>,
        after: Option<FileIdentity>,
        desired: [u8; 32],
        now: SystemTime,
    ) {
        if let Some(identity) =
            after.filter(|after| Some(after) == before.as_ref() && after.settled(now))
        {
            self.files
                .insert(path.to_path_buf(), Observation { identity, desired });
        } else {
            // An observation made in a timestamp's racy interval cannot later
            // become authority merely because wall-clock time has passed.
            self.files.remove(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ORIGINAL: &str = "// 雪😀\r\nexport const value = 'before';\r\n";
    const CHANGED: &str = "// 雪😀\r\nexport const value = 'after!';\r\n";

    fn settled_clock() -> SystemTime {
        SystemTime::now() + Duration::from_secs(3)
    }

    #[test]
    fn exact_unchanged_text_keeps_a_settled_observation_without_reopening_it() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("A.vue.ts");
        std::fs::write(&path, ORIGINAL).unwrap();
        let mut cache = CanonFileCache::default();
        let now = settled_clock();
        for _ in 0..32 {
            assert!(!cache.synchronize_at(&path, ORIGINAL, now).unwrap());
        }
        assert_eq!(cache.reads, if cfg!(unix) { 1 } else { 32 });
        assert_eq!(std::fs::read(&path).unwrap(), ORIGINAL.as_bytes());
        cache.clear();
        assert!(!cache.synchronize_at(&path, ORIGINAL, now).unwrap());
        assert_eq!(cache.reads, if cfg!(unix) { 2 } else { 33 });
    }

    #[test]
    fn recent_observations_keep_complete_reads_and_different_desired_bytes_are_written() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("A.vue.ts");
        std::fs::write(&path, ORIGINAL).unwrap();
        let mut cache = CanonFileCache::default();
        let now = SystemTime::now();
        for _ in 0..3 {
            assert!(!cache.synchronize_at(&path, ORIGINAL, now).unwrap());
        }
        assert_eq!(cache.reads, 3);
        assert!(
            cache.files.is_empty(),
            "recent exact reads are never retained as settled authority"
        );
        assert!(
            cache
                .synchronize_at(&path, CHANGED, settled_clock())
                .unwrap()
        );
        assert_eq!(std::fs::read(&path).unwrap(), CHANGED.as_bytes());
        assert!(
            cache.files.is_empty(),
            "a write never acknowledges an unseen physical revision"
        );
    }

    #[test]
    fn same_mtime_same_length_tampering_and_atomic_replacement_cannot_reuse_an_observation() {
        for replace in [false, true] {
            let root = tempfile::tempdir().unwrap();
            let path = root.path().join("A.vue.ts");
            std::fs::write(&path, ORIGINAL).unwrap();
            let modified = std::fs::metadata(&path).unwrap().modified().unwrap();
            let mut cache = CanonFileCache::default();
            // Keep a real settled observation here: a future clock would let
            // an immediate same-tick rewrite impersonate an old file, unlike
            // synchronize's actual wall clock and its one-second racy fence.
            std::thread::sleep(Duration::from_millis(1100));
            assert!(!cache.synchronize(&path, ORIGINAL).unwrap());
            assert_eq!(ORIGINAL.len(), CHANGED.len());
            if replace {
                let next = root.path().join("next");
                std::fs::write(&next, CHANGED).unwrap();
                std::fs::rename(next, &path).unwrap();
            } else {
                std::fs::write(&path, CHANGED).unwrap();
            }
            std::fs::File::options()
                .write(true)
                .open(&path)
                .unwrap()
                .set_times(std::fs::FileTimes::new().set_modified(modified))
                .unwrap();
            assert!(cache.synchronize(&path, ORIGINAL).unwrap());
            assert_eq!(std::fs::read(&path).unwrap(), ORIGINAL.as_bytes());
            assert_eq!(cache.reads, 2);
        }
    }

    #[test]
    fn retaining_a_new_desired_revision_before_a_failed_batch_does_not_acknowledge_old_disk_bytes()
    {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("A.vue.ts");
        let uri = "file:///workspace/A.vue.ts";
        std::fs::write(&path, ORIGINAL).unwrap();
        let mut documents = crate::lsp_client::tracked_documents::TrackedDocumentTexts::default();
        let mut cache = CanonFileCache::default();
        let now = settled_clock();
        documents.insert(uri.into(), ORIGINAL.into());
        assert!(
            !cache
                .synchronize_at_with_digest(&path, ORIGINAL, now, || {
                    documents.desired_digest(uri, ORIGINAL)
                })
                .unwrap()
        );
        // A namespace or write failure after retain leaves this new desired
        // revision in memory, while the physical observation still names old
        // bytes. Repeating the equal retain must not reuse that old digest.
        documents.insert(uri.into(), CHANGED.into());
        let expected_digest: [u8; 32] = Sha256::digest(CHANGED.as_bytes()).into();
        assert_eq!(documents.desired_digest(uri, CHANGED), expected_digest);
        documents.insert(uri.into(), CHANGED.into());
        assert!(
            cache
                .synchronize_at_with_digest(&path, CHANGED, now, || {
                    documents.desired_digest(uri, CHANGED)
                })
                .unwrap()
        );
        assert_eq!(std::fs::read(&path).unwrap(), CHANGED.as_bytes());
    }

    #[test]
    fn one_physical_path_keeps_every_duplicate_and_encoded_uri_row_on_its_own_desired_bytes() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("A.vue.ts");
        let uri = "file:///workspace/A.vue.ts";
        let alias = "file:///workspace/%41.vue.ts";
        std::fs::write(&path, ORIGINAL).unwrap();
        let mut documents = crate::lsp_client::tracked_documents::TrackedDocumentTexts::default();
        let rows = [(uri, ORIGINAL), (uri, CHANGED), (alias, ORIGINAL)];
        for (uri, content) in rows {
            documents.insert(uri.into(), content.into());
        }
        let mut cache = CanonFileCache::default();
        let now = settled_clock();
        for (uri, content) in rows {
            cache
                .synchronize_at_with_digest(&path, content, now, || {
                    documents.desired_digest(uri, content)
                })
                .unwrap();
            assert_eq!(std::fs::read(&path).unwrap(), content.as_bytes());
        }
    }

    #[cfg(unix)]
    #[test]
    fn a_coarse_change_time_cannot_promote_a_recent_read_into_later_authority() {
        let mut cache = CanonFileCache::default();
        let path = Path::new("/private/A.vue.ts");
        let identity = FileIdentity {
            device: 1,
            inode: 2,
            length: ORIGINAL.len() as u64,
            modified: (100, 0),
            changed: (100, 0),
        };
        let read_time = UNIX_EPOCH + Duration::from_millis(100_100);
        let later = UNIX_EPOCH + Duration::from_secs(102);
        assert!(!identity.settled(read_time));
        assert!(identity.settled(later));
        let digest = Sha256::digest(ORIGINAL.as_bytes()).into();
        cache.retain_verified_observation(
            path,
            Some(identity.clone()),
            Some(identity),
            digest,
            read_time,
        );
        // A same-length write in the same coarse second can retain this exact
        // metadata and restored mtime. It must encounter an absent cache later.
        assert!(!cache.files.contains_key(path));
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_never_supply_cached_regular_file_authority() {
        let root = tempfile::tempdir().unwrap();
        let a = root.path().join("A");
        let b = root.path().join("B");
        let link = root.path().join("view");
        std::fs::write(&a, ORIGINAL).unwrap();
        std::fs::write(&b, CHANGED).unwrap();
        std::os::unix::fs::symlink(&a, &link).unwrap();
        let mut cache = CanonFileCache::default();
        let now = settled_clock();
        assert!(!cache.synchronize_at(&link, ORIGINAL, now).unwrap());
        assert!(!cache.synchronize_at(&link, ORIGINAL, now).unwrap());
        assert_eq!(cache.reads, 2);
        assert!(cache.files.is_empty());
        std::fs::remove_file(&link).unwrap();
        std::os::unix::fs::symlink(&b, &link).unwrap();
        assert!(cache.synchronize_at(&link, ORIGINAL, now).unwrap());
        assert_eq!(std::fs::read(&a).unwrap(), ORIGINAL.as_bytes());
        assert_eq!(std::fs::read(&b).unwrap(), ORIGINAL.as_bytes());
        assert_eq!(cache.reads, 3);
        assert!(cache.files.is_empty());
    }
}
