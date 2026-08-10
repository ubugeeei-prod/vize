//! Bounded session cache and strong disk fingerprints for alias contexts.
#![allow(clippy::disallowed_types)]

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use vize_carton::FxHashMap;

use super::AliasContext;

const CONTEXT_CACHE_CAPACITY: usize = 8;

#[derive(Default)]
pub(super) struct SessionCache {
    slots: FxHashMap<PathBuf, CachedContext>,
    clock: u64,
}

struct CachedContext {
    fingerprint: ContextFingerprint,
    context: Arc<AliasContext>,
    last_used: u64,
}

impl SessionCache {
    pub(super) fn get(
        &mut self,
        source_path: &Path,
        fingerprint: &ContextFingerprint,
    ) -> Option<Arc<AliasContext>> {
        let valid = self.slots.get(source_path).is_some_and(|cached| {
            cached.fingerprint == *fingerprint && cached.fingerprint.stamps_still_valid()
        });
        if !valid {
            self.slots.remove(source_path);
            return None;
        }
        self.clock = self.clock.wrapping_add(1);
        let cached = self.slots.get_mut(source_path)?;
        cached.last_used = self.clock;
        Some(Arc::clone(&cached.context))
    }

    pub(super) fn insert(
        &mut self,
        source_path: PathBuf,
        fingerprint: ContextFingerprint,
        context: Arc<AliasContext>,
    ) {
        if self.slots.len() >= CONTEXT_CACHE_CAPACITY && !self.slots.contains_key(&source_path) {
            let lru = self
                .slots
                .iter()
                .min_by_key(|(_, cached)| cached.last_used)
                .map(|(path, _)| path.clone());
            if let Some(lru) = lru {
                self.slots.remove(&lru);
            }
        }
        self.clock = self.clock.wrapping_add(1);
        self.slots.insert(
            source_path,
            CachedContext {
                fingerprint,
                context,
                last_used: self.clock,
            },
        );
    }
}

pub(super) fn lock_session_cache() -> MutexGuard<'static, SessionCache> {
    static CACHE: OnceLock<Mutex<SessionCache>> = OnceLock::new();
    recover_lock(CACHE.get_or_init(|| Mutex::new(SessionCache::default())))
}

fn recover_lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            mutex.clear_poison();
            poisoned.into_inner()
        }
    }
}

/// The import closure and disk inputs a cached editor route depends on.
#[derive(PartialEq)]
#[allow(clippy::disallowed_types)]
pub(super) struct ContextFingerprint {
    host_content: u64,
    overlays: u64,
    stamps: Vec<DiskInputStamp>,
}

impl ContextFingerprint {
    #[allow(clippy::disallowed_methods)]
    pub(super) fn capture(
        source_path: &Path,
        content: &str,
        overlays: &FxHashMap<PathBuf, &str>,
    ) -> Self {
        use std::hash::{Hash, Hasher};
        let mut host = std::hash::DefaultHasher::new();
        source_path.hash(&mut host);
        content.hash(&mut host);
        let mut overlay_entries: Vec<_> = overlays.iter().collect();
        overlay_entries.sort_by(|left, right| left.0.cmp(right.0));
        let mut overlay_hash = std::hash::DefaultHasher::new();
        for (path, text) in overlay_entries {
            path.hash(&mut overlay_hash);
            text.hash(&mut overlay_hash);
        }
        Self {
            host_content: host.finish(),
            overlays: overlay_hash.finish(),
            stamps: Vec::new(),
        }
    }

    pub(super) fn stamp(&mut self, context: &AliasContext) {
        let mut paths = vec![context.project_root.join("tsconfig.json")];
        if let Some(mirror) = context.mirror.as_ref() {
            paths.extend(mirror.governing_config_paths());
            // The materialized closure intentionally keeps content digests:
            // same-mtime, same-length edits must invalidate an editor session.
            paths.extend(mirror.registered_original_paths_sorted());
        }
        paths.extend(context.route_inputs.iter().cloned());
        paths.sort();
        paths.dedup();
        self.stamps = paths.into_iter().map(DiskInputStamp::capture).collect();
    }

    fn stamps_still_valid(&self) -> bool {
        self.stamps
            .iter()
            .all(|stamp| *stamp == DiskInputStamp::capture(stamp.path.clone()))
    }
}

/// Disk identity strong enough for same-mtime edits and workspace-link retargets.
#[derive(PartialEq)]
struct DiskInputStamp {
    path: PathBuf,
    modified: Option<std::time::SystemTime>,
    len: Option<u64>,
    kind: Option<DiskInputKind>,
    content_digest: Option<u64>,
    symlink_target: Option<PathBuf>,
}

#[derive(Clone, Copy, PartialEq)]
enum DiskInputKind {
    File,
    Directory,
    Symlink,
    Other,
}

impl DiskInputStamp {
    fn capture(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let metadata = std::fs::symlink_metadata(&path).ok();
        let modified = metadata
            .as_ref()
            .and_then(|metadata| metadata.modified().ok());
        let len = metadata.as_ref().map(std::fs::Metadata::len);
        let kind = metadata.as_ref().map(|metadata| {
            let file_type = metadata.file_type();
            if file_type.is_file() {
                DiskInputKind::File
            } else if file_type.is_dir() {
                DiskInputKind::Directory
            } else if file_type.is_symlink() {
                DiskInputKind::Symlink
            } else {
                DiskInputKind::Other
            }
        });
        let content_digest = matches!(kind, Some(DiskInputKind::File))
            .then(|| std::fs::read(&path).ok().map(|content| digest(&content)))
            .flatten();
        let symlink_target = matches!(kind, Some(DiskInputKind::Symlink))
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
}

fn digest(content: &[u8]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::hash::DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::{ContextFingerprint, SessionCache, recover_lock};
    use crate::corsa_bridge::vue_dependencies_alias::AliasContext;
    use vize_carton::{FxHashMap, cstr};

    #[test]
    fn cache_evicts_only_the_least_recently_used_context() {
        let mut cache = SessionCache::default();
        let overlays = FxHashMap::default();
        for index in 0..9 {
            let path = std::path::PathBuf::from(cstr!("/workspace/{index}/App.vue").as_str());
            let context = Arc::new(AliasContext::for_host(&path, "", &overlays));
            let mut fingerprint = ContextFingerprint::capture(&path, "", &overlays);
            fingerprint.stamp(&context);
            cache.insert(path, fingerprint, context);
        }
        assert_eq!(cache.slots.len(), 8);
        assert!(
            !cache
                .slots
                .contains_key(std::path::Path::new("/workspace/0/App.vue"))
        );
        assert!(
            cache
                .slots
                .contains_key(std::path::Path::new("/workspace/8/App.vue"))
        );
    }

    #[test]
    fn poisoned_mutex_is_recovered() {
        let mutex = Arc::new(Mutex::new(0));
        let poisoned = Arc::clone(&mutex);
        let _ = std::thread::spawn(move || {
            let _guard = poisoned.lock().unwrap();
            panic!("poison for test");
        })
        .join();
        *recover_lock(&mutex) = 1;
        assert_eq!(*recover_lock(&mutex), 1);
    }
}
