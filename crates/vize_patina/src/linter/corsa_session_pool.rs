//! One live Corsa session per project root and tsconfig.
//!
//! The map lock covers lookup and insert only. Spawn, close, and probes run
//! after it is released, and each session has its own mutex so one package
//! cannot stall another.

#![expect(
    clippy::disallowed_types,
    reason = "worker threads share one session mutex per tsconfig after the pool lock drops"
)]

use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

use super::corsa_session::CorsaTypeAwareSession;
use vize_l0::{FxHashMap, String, ToCompactString, profile};

#[derive(Clone, Eq, PartialEq, Hash)]
pub(in crate::linter) struct CorsaSessionKey {
    pub(in crate::linter) project_root: PathBuf,
    pub(in crate::linter) tsconfig: Option<PathBuf>,
}

type SessionSlot = Arc<Mutex<CorsaTypeAwareSession>>;

pub(crate) struct CorsaSessionPool {
    sessions: Mutex<FxHashMap<CorsaSessionKey, SessionSlot>>,
}

impl Default for CorsaSessionPool {
    fn default() -> Self {
        Self {
            sessions: Mutex::new(FxHashMap::default()),
        }
    }
}

impl CorsaSessionPool {
    pub(crate) fn close_all(&self) {
        let taken = {
            let Ok(mut guard) = self.sessions.lock() else {
                return;
            };
            std::mem::take(&mut *guard)
        };
        for (_, slot) in taken {
            if let Ok(mut session) = slot.lock() {
                session.close();
            }
        }
    }

    /// Run `work` on the session for `key`, spawning it only when missing.
    ///
    /// The map lock is not held across `spawn`, `close`, or `work`. A failed
    /// `work` drops only this key's session.
    pub(in crate::linter) fn with_session<T>(
        &self,
        key: CorsaSessionKey,
        spawn: impl FnOnce() -> Result<CorsaTypeAwareSession, String>,
        work: impl FnOnce(&mut CorsaTypeAwareSession) -> Result<T, String>,
    ) -> Result<T, String> {
        let slot = self.slot(&key, spawn)?;
        let mut session = slot.lock().map_err(|_| lock_failure())?;
        let result = work(&mut session);
        if result.is_err() {
            session.close();
            drop(session);
            self.remove_slot(&key, &slot);
        }
        result
    }

    fn slot(
        &self,
        key: &CorsaSessionKey,
        spawn: impl FnOnce() -> Result<CorsaTypeAwareSession, String>,
    ) -> Result<SessionSlot, String> {
        if let Some(existing) = self.map_lock()?.get(key).cloned() {
            return Ok(existing);
        }

        let created = spawn()?;
        let mut guard = self.map_lock()?;
        if let Some(existing) = guard.get(key).cloned() {
            drop(guard);
            drop(created);
            return Ok(existing);
        }

        let slot = Arc::new(Mutex::new(created));
        guard.insert(key.clone(), Arc::clone(&slot));
        Ok(slot)
    }

    fn remove_slot(&self, key: &CorsaSessionKey, slot: &SessionSlot) {
        let Ok(mut guard) = self.sessions.lock() else {
            return;
        };
        let still_current = guard
            .get(key)
            .is_some_and(|current| Arc::ptr_eq(current, slot));
        if still_current {
            guard.remove(key);
        }
    }

    fn map_lock(&self) -> Result<MutexGuard<'_, FxHashMap<CorsaSessionKey, SessionSlot>>, String> {
        profile!("patina.type_aware.corsa.lock", self.sessions.lock()).map_err(|_| lock_failure())
    }

    #[cfg(test)]
    pub(in crate::linter) fn len(&self) -> usize {
        self.sessions.lock().map(|guard| guard.len()).unwrap_or(0)
    }
}

fn lock_failure() -> String {
    "Failed to lock corsa type session".to_compact_string()
}
