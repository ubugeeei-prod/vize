//! One live Corsa session per project root and tsconfig.
//!
//! The map lock covers claim and publish only. Spawn runs after it is
//! released, and one key has one in-flight spawn. Each session has its own
//! mutex so one package cannot stall another. The session lock span is only
//! the wait for that mutex.

#![expect(
    clippy::disallowed_types,
    reason = "worker threads share one session mutex per tsconfig after the pool lock drops"
)]

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::corsa_session::CorsaTypeAwareSession;
use super::corsa_session_flight::{PoolInner, get_or_spawn};
use vize_l0::{String, ToCompactString, profile};

#[derive(Clone, Eq, PartialEq, Hash)]
pub(in crate::linter) struct CorsaSessionKey {
    pub(in crate::linter) project_root: PathBuf,
    pub(in crate::linter) tsconfig: Option<PathBuf>,
}

type SessionSlot = Arc<Mutex<CorsaTypeAwareSession>>;

pub(crate) struct CorsaSessionPool {
    sessions: Mutex<PoolInner<SessionSlot>>,
}

impl Default for CorsaSessionPool {
    fn default() -> Self {
        Self {
            sessions: Mutex::new(PoolInner::default()),
        }
    }
}

impl CorsaSessionPool {
    pub(crate) fn close_all(&self) {
        let taken = {
            let Ok(mut guard) = self.sessions.lock() else {
                return;
            };
            std::mem::take(&mut guard.ready)
        };
        for (_, slot) in taken {
            if let Ok(mut session) = slot.lock() {
                session.close();
            }
        }
    }

    /// Run `work` on the session for `key`, spawning it only when missing.
    ///
    /// The map lock is not held across `spawn` or `work`. A failed `work`
    /// drops only this key's session.
    pub(in crate::linter) fn with_session<T>(
        &self,
        key: CorsaSessionKey,
        spawn: impl FnOnce() -> Result<CorsaTypeAwareSession, String>,
        work: impl FnOnce(&mut CorsaTypeAwareSession) -> Result<T, String>,
    ) -> Result<T, String> {
        let slot = get_or_spawn(&self.sessions, &key, || {
            spawn().map(|session| Arc::new(Mutex::new(session)))
        })?;
        let mut session = profile!("patina.corsa_session.session_lock", slot.lock())
            .map_err(|_| lock_failure())?;
        let result = work(&mut session);
        if result.is_err() {
            session.close();
            drop(session);
            self.remove_slot(&key, &slot);
        }
        result
    }

    fn remove_slot(&self, key: &CorsaSessionKey, slot: &SessionSlot) {
        let Ok(mut guard) = self.sessions.lock() else {
            return;
        };
        let still_current = guard
            .ready
            .get(key)
            .is_some_and(|current| Arc::ptr_eq(current, slot));
        if still_current {
            guard.ready.remove(key);
        }
    }

    #[cfg(test)]
    pub(in crate::linter) fn len(&self) -> usize {
        self.sessions
            .lock()
            .map(|guard| guard.ready.len())
            .unwrap_or(0)
    }
}

pub(super) fn lock_failure() -> String {
    "Failed to lock corsa type session".to_compact_string()
}
