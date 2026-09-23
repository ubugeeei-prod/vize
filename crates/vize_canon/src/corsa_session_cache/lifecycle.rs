//! Session map for one check-server process (issue #699, Davinci P5-8).
//!
//! A hit reuses the `ProjectSession`. A miss is the only path that spawns one.
//! A session that has not been used for [`DEFAULT_SESSION_IDLE`] is dropped.

use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use vize_carton::FxHashMap;

use super::CorsaSessionKey;

/// How long an unused project session stays alive unless the server overrides it.
pub const DEFAULT_SESSION_IDLE: Duration = Duration::from_secs(60);

thread_local! {
    static TYPESCRIPT_PROJECT_INITS: Cell<u64> = const { Cell::new(0) };
}

/// Count one TypeScript project initialization (`ProjectSession` or editor LSP spawn).
pub(crate) fn note_typescript_project_init() {
    TYPESCRIPT_PROJECT_INITS.with(|inits| inits.set(inits.get().saturating_add(1)));
}

/// Initializations observed on this thread. The check server samples the delta
/// around one check so a later check can prove it added none.
pub(crate) fn typescript_project_inits() -> u64 {
    TYPESCRIPT_PROJECT_INITS.with(|inits| inits.get())
}

struct Slot<T> {
    value: T,
    last_used: Instant,
}

/// Live sessions keyed by the `corsa.session` fingerprint.
pub struct SessionMap<T> {
    slots: FxHashMap<[u8; 16], Slot<T>>,
    idle: Duration,
    #[cfg(test)]
    spawns: u64,
}

impl<T> SessionMap<T> {
    pub(crate) fn new(idle: Duration) -> Self {
        Self {
            slots: FxHashMap::default(),
            idle,
            #[cfg(test)]
            spawns: 0,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.slots.len()
    }

    #[cfg(test)]
    pub(crate) fn spawns(&self) -> u64 {
        self.spawns
    }

    /// Drop sessions whose last use is at least `idle` before `now`.
    /// A zero idle timeout disables teardown.
    pub(crate) fn reap(&mut self, now: Instant) {
        if self.idle.is_zero() {
            return;
        }
        let idle = self.idle;
        self.slots
            .retain(|_, slot| now.saturating_duration_since(slot.last_used) < idle);
    }

    pub(crate) fn get_mut(&mut self, key: &CorsaSessionKey, now: Instant) -> Option<&mut T> {
        let slot = self.slots.get_mut(&key.fingerprint())?;
        slot.last_used = now;
        Some(&mut slot.value)
    }

    /// Record a session that was just spawned. This is not the TypeScript
    /// init counter; that counter moves only inside the spawn itself.
    pub(crate) fn insert_spawned(
        &mut self,
        key: CorsaSessionKey,
        value: T,
        now: Instant,
    ) -> &mut T {
        #[cfg(test)]
        {
            self.spawns = self.spawns.saturating_add(1);
        }
        let slot = self
            .slots
            .entry(key.fingerprint())
            .insert_entry(Slot {
                value,
                last_used: now,
            })
            .into_mut();
        &mut slot.value
    }
}

/// Tsconfig whose identity keys the session. The nearest `tsconfig.json` at or
/// under `project_root` wins; otherwise the shell path, even when it is absent.
pub(crate) fn project_tsconfig(source: &Path, project_root: &Path) -> PathBuf {
    let mut dir = if source.is_dir() {
        Some(source)
    } else {
        source.parent()
    };
    while let Some(current) = dir {
        if current != project_root && !current.starts_with(project_root) {
            break;
        }
        let candidate = current.join("tsconfig.json");
        if candidate.is_file() {
            return candidate;
        }
        if current == project_root {
            break;
        }
        dir = current.parent();
    }
    project_root.join("tsconfig.json")
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use vize_carton::String;

    use super::{SessionMap, project_tsconfig};
    use crate::corsa_session_cache::{CorsaSessionKey, SessionInputs};

    fn inputs() -> SessionInputs {
        SessionInputs {
            project: "/work/app/tsconfig.json".into(),
            tsconfig: String::from("0123456789abcdef0123456789abcdef"),
            toolchain: String::from("0.425.1"),
            corsa: String::from("tsgo"),
            flags: String::from(""),
            platform: String::from("aarch64-macos-unix"),
        }
    }

    fn key_with_flags(flags: &str) -> CorsaSessionKey {
        let mut inputs = inputs();
        inputs.flags = String::from(flags);
        CorsaSessionKey::from_inputs(&inputs)
    }

    #[test]
    fn a_hit_does_not_spawn_and_an_idle_session_is_dropped() {
        let start = Instant::now();
        let idle = Duration::from_secs(1);
        let mut map = SessionMap::new(idle);
        let key = key_with_flags("");
        map.insert_spawned(key.clone(), 1_u8, start);
        assert_eq!(map.spawns(), 1);
        assert_eq!(map.len(), 1);

        let reused = map.get_mut(&key, start + Duration::from_millis(500));
        assert_eq!(reused.copied(), Some(1));
        assert_eq!(map.spawns(), 1, "a hit must not spawn");

        map.reap(start + Duration::from_millis(1_499));
        assert_eq!(map.len(), 1, "still inside the idle window");
        map.reap(start + Duration::from_millis(1_500));
        assert_eq!(map.len(), 0, "idle teardown");
        assert!(
            map.get_mut(&key, start + Duration::from_millis(1_500))
                .is_none()
        );
    }

    #[test]
    fn a_different_key_is_a_different_session() {
        let now = Instant::now();
        let mut map = SessionMap::new(Duration::from_secs(60));
        let first = key_with_flags("");
        let second = key_with_flags("options-api=0");
        map.insert_spawned(first.clone(), "first", now);
        map.insert_spawned(second.clone(), "second", now);
        assert_eq!(map.spawns(), 2);
        assert_eq!(map.len(), 2);
        assert_eq!(map.get_mut(&first, now).copied(), Some("first"));
        assert_eq!(map.spawns(), 2);
    }

    #[test]
    fn a_zero_idle_timeout_keeps_the_session() {
        let now = Instant::now();
        let mut map = SessionMap::<u8>::new(Duration::ZERO);
        let key = key_with_flags("");
        map.insert_spawned(key, 7, now);
        map.reap(now + Duration::from_secs(3_600));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn the_nearest_tsconfig_at_or_under_the_project_root_keys_the_session() {
        let root = tempfile::tempdir().unwrap();
        let nested = root.path().join("packages/ui");
        std::fs::create_dir_all(nested.join("src")).unwrap();
        std::fs::write(root.path().join("tsconfig.json"), "{}").unwrap();
        std::fs::write(nested.join("tsconfig.json"), "{}").unwrap();
        let source = nested.join("src/App.vue");
        std::fs::write(&source, "").unwrap();

        assert_eq!(
            project_tsconfig(&source, root.path()),
            nested.join("tsconfig.json")
        );
        assert_eq!(
            project_tsconfig(&root.path().join("src/Main.vue"), root.path()),
            root.path().join("tsconfig.json")
        );
        let outside = root.path().join("../other/App.vue");
        assert_eq!(
            project_tsconfig(&outside, root.path()),
            root.path().join("tsconfig.json")
        );
    }
}
