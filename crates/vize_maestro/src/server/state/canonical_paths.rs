//! Session-scoped physical path identities for canonical editor requests.

use dashmap::DashMap;
use parking_lot::{RwLock, RwLockReadGuard};
use std::path::{Path, PathBuf};

use super::ServerState;

#[derive(Default)]
pub(super) struct Cache {
    paths: DashMap<PathBuf, PathBuf>,
    epoch: RwLock<u64>,
}

/// A synchronous physical-path observation. Never retain this across an await.
pub(crate) struct Read<'a> {
    cache: &'a Cache,
    // Structural-only builds still retain the guard until lookup completes.
    _epoch: RwLockReadGuard<'a, u64>,
}

impl Cache {
    fn read(&self) -> Read<'_> {
        Read {
            cache: self,
            _epoch: self.epoch.read(),
        }
    }

    fn clear(&self) {
        let mut epoch = self.epoch.write();
        self.paths.clear();
        *epoch = epoch.wrapping_add(1);
    }
}

impl Read<'_> {
    #[cfg(any(feature = "native", test))]
    pub(crate) fn epoch(&self) -> u64 {
        *self._epoch
    }

    pub(crate) fn resolve(&self, path: &Path) -> PathBuf {
        if let Some(cached) = self.cache.paths.get(path) {
            return cached.clone();
        }
        let physical = vize_carton::path::physical_path(path);
        self.cache
            .paths
            .entry(path.to_path_buf())
            .or_insert_with(|| physical.clone());
        physical
    }

    /// A competing first lookup may have committed a different observation.
    /// Keep the legacy lookup result, but never index that uncommitted value.
    #[cfg(any(feature = "native", test))]
    pub(crate) fn matches_observation(&self, path: &Path, physical: &Path) -> bool {
        self.cache
            .paths
            .get(path)
            .is_some_and(|cached| cached.value().as_path() == physical)
    }
}

impl ServerState {
    /// Resolve one path at most once between filesystem notifications.
    pub(crate) fn canonical_editor_path(&self, path: &Path) -> PathBuf {
        self.canonical_path_read().resolve(path)
    }

    /// Protect one synchronous decision from concurrent watched-path clearing.
    pub(crate) fn canonical_path_read(&self) -> Read<'_> {
        self.canonical_paths.read()
    }

    pub(crate) fn invalidate_canonical_editor_paths(&self) {
        self.canonical_paths.clear();
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::ServerState;

    #[test]
    fn short_read_lease_excludes_another_thread_clear_without_nested_reading() {
        use std::{fs, sync::mpsc, thread};

        let directory = tempfile::tempdir().expect("directory");
        let first = directory.path().join("first.ts");
        let second = directory.path().join("second.ts");
        let alias = directory.path().join("alias.ts");
        fs::write(&first, "first").expect("first source");
        fs::write(&second, "second").expect("second source");
        let first_physical = first.canonicalize().expect("first physical path");
        let second_physical = second.canonicalize().expect("second physical path");
        std::os::unix::fs::symlink(&first, &alias).expect("first alias");
        let state = ServerState::new();
        thread::scope(|scope| {
            let observation = state.canonical_path_read();
            let epoch = observation.epoch();
            assert_eq!(observation.resolve(&alias), first_physical);
            let (attempted, waiting) = mpsc::sync_channel(0);
            let worker_state = &state;
            let worker_alias = alias.clone();
            let worker_second = second.clone();
            let worker = scope.spawn(move || {
                fs::remove_file(&worker_alias).expect("remove first alias");
                std::os::unix::fs::symlink(worker_second, worker_alias).expect("second alias");
                assert!(worker_state.canonical_paths.epoch.try_write().is_none());
                attempted.send(()).expect("clear attempt observed");
                worker_state.invalidate_canonical_editor_paths();
            });
            waiting.recv().expect("worker reached invalidation");
            // Observe a queued writer before exercising the non-nested lookup.
            // The deadline is a deadlock watchdog, not a performance assertion.
            let watchdog = std::time::Instant::now() + std::time::Duration::from_secs(5);
            while state.canonical_paths.epoch.try_read().is_some() {
                assert!(
                    std::time::Instant::now() < watchdog,
                    "clear writer did not queue"
                );
                thread::yield_now();
            }
            assert_eq!(observation.epoch(), epoch);
            // This must use the lease's inner lookup. Acquiring another read lock
            // behind the waiting writer could deadlock an otherwise warm query.
            assert_eq!(observation.resolve(&alias), first_physical);
            drop(observation);
            worker.join().expect("clear worker");
            let current = state.canonical_path_read();
            assert_eq!(current.epoch(), epoch.wrapping_add(1));
            assert_eq!(current.resolve(&alias), second_physical);
            assert!(current.matches_observation(&alias, &second_physical));
            assert!(!current.matches_observation(&alias, &first_physical));
        });
    }

    #[test]
    fn watched_invalidation_refreshes_a_retargeted_symlink() {
        let directory = tempfile::tempdir().unwrap();
        let first = directory.path().join("first");
        let second = directory.path().join("second");
        let alias = directory.path().join("alias");
        std::fs::create_dir(&first).unwrap();
        std::fs::create_dir(&second).unwrap();
        std::fs::write(first.join("source.ts"), "first").unwrap();
        std::fs::write(second.join("source.ts"), "second").unwrap();
        let first_source = first.canonicalize().unwrap().join("source.ts");
        let second_source = second.canonicalize().unwrap().join("source.ts");
        std::os::unix::fs::symlink(&first, &alias).unwrap();
        let state = ServerState::new();
        let authored = alias.join("source.ts");

        assert_eq!(state.canonical_editor_path(&authored), first_source);
        std::fs::remove_file(&alias).unwrap();
        std::os::unix::fs::symlink(&second, &alias).unwrap();
        assert_eq!(state.canonical_editor_path(&authored), first_source);

        state.invalidate_canonical_editor_paths();
        assert_eq!(state.canonical_editor_path(&authored), second_source);
    }
}
