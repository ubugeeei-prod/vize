//! One live Corsa session per project root and tsconfig.
//!
//! The map lock covers claim and publish only. Spawn runs after it is
//! released, and one key has one in-flight spawn. Each session has its own
//! mutex so one package cannot stall another. The session lock span is only
//! the wait for that mutex. A caller that waited on a session another
//! caller's failure closed takes a live session instead of failing (#10393).

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

type Slot<S> = Arc<Mutex<S>>;

pub(crate) struct CorsaSessionPool {
    sessions: Mutex<PoolInner<Slot<CorsaTypeAwareSession>>>,
}

/// A session the pool can hand out and retire.
pub(super) trait PooledSession {
    fn is_closed(&self) -> bool;
    fn close(&mut self);
}

impl PooledSession for CorsaTypeAwareSession {
    fn is_closed(&self) -> bool {
        CorsaTypeAwareSession::is_closed(self)
    }

    fn close(&mut self) {
        CorsaTypeAwareSession::close(self);
    }
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
        spawn: impl FnMut() -> Result<CorsaTypeAwareSession, String>,
        work: impl FnOnce(&mut CorsaTypeAwareSession) -> Result<T, String>,
    ) -> Result<T, String> {
        with_pooled_session(&self.sessions, &key, spawn, work)
    }

    #[cfg(test)]
    pub(in crate::linter) fn len(&self) -> usize {
        self.sessions
            .lock()
            .map(|guard| guard.ready.len())
            .unwrap_or(0)
    }
}

fn with_pooled_session<S: PooledSession, T>(
    sessions: &Mutex<PoolInner<Slot<S>>>,
    key: &CorsaSessionKey,
    mut spawn: impl FnMut() -> Result<S, String>,
    work: impl FnOnce(&mut S) -> Result<T, String>,
) -> Result<T, String> {
    loop {
        let slot = get_or_spawn(sessions, key, || {
            spawn().map(|session| Arc::new(Mutex::new(session)))
        })?;
        let mut session = profile!("patina.corsa_session.session_lock", slot.lock())
            .map_err(|_| lock_failure())?;
        if session.is_closed() {
            // A failed caller closed it while this one waited. Take a live
            // session instead of failing on the closed transport.
            drop(session);
            remove_slot(sessions, key, &slot);
            continue;
        }
        let result = work(&mut session);
        if result.is_err() {
            session.close();
            drop(session);
            remove_slot(sessions, key, &slot);
        }
        return result;
    }
}

fn remove_slot<S>(sessions: &Mutex<PoolInner<Slot<S>>>, key: &CorsaSessionKey, slot: &Slot<S>) {
    let Ok(mut guard) = sessions.lock() else {
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

pub(super) fn lock_failure() -> String {
    "Failed to lock corsa type session".to_compact_string()
}

#[cfg(test)]
mod tests {
    use super::{PoolInner, PooledSession, with_pooled_session};
    use crate::linter::corsa_session_pool::CorsaSessionKey;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Barrier, Mutex};
    use vize_l0::ToCompactString;

    struct Fake {
        id: usize,
        closed: bool,
    }

    impl PooledSession for Fake {
        fn is_closed(&self) -> bool {
            self.closed
        }

        fn close(&mut self) {
            self.closed = true;
        }
    }

    #[test]
    fn waiters_take_a_live_session_after_another_caller_fails() {
        let sessions = Mutex::new(PoolInner::<Arc<Mutex<Fake>>>::default());
        let key = CorsaSessionKey {
            project_root: "/app".into(),
            tsconfig: None,
        };
        let spawns = AtomicUsize::new(0);
        let spawn = || {
            Ok(Fake {
                id: spawns.fetch_add(1, Ordering::SeqCst),
                closed: false,
            })
        };
        let failing = Barrier::new(2);
        let (failed, waited) = std::thread::scope(|scope| {
            let failed = scope.spawn(|| {
                with_pooled_session(&sessions, &key, spawn, |_| -> Result<usize, _> {
                    failing.wait();
                    // Fail only once the waiter holds this slot.
                    while sessions
                        .lock()
                        .ok()
                        .and_then(|guard| guard.ready.get(&key).map(Arc::strong_count))
                        .is_some_and(|count| count < 3)
                    {
                        std::thread::yield_now();
                    }
                    Err("source snapshot differs".to_compact_string())
                })
            });
            failing.wait();
            let waited = scope.spawn(|| {
                with_pooled_session(&sessions, &key, spawn, |session| {
                    if session.closed {
                        Err("process is closed: jsonrpc connection".to_compact_string())
                    } else {
                        Ok(session.id)
                    }
                })
            });
            (failed.join().unwrap(), waited.join().unwrap())
        });
        assert_eq!(failed, Err("source snapshot differs".to_compact_string()));
        assert_eq!(waited, Ok(1));
        assert_eq!(spawns.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn concurrent_type_aware_lint_shares_one_live_session() {
        let Ok(mut probe) =
            super::CorsaTypeAwareSession::new_with_corsa_path("Component.vue", None, None)
        else {
            return;
        };
        probe.close();
        let inputs = [
            (
                "First.vue",
                include_str!(
                    "../../../../tests/_fixtures/differential/linter/corsa-session-waiters-10393/First.vue.txt"
                ),
            ),
            (
                "Second.vue",
                include_str!(
                    "../../../../tests/_fixtures/differential/linter/corsa-session-waiters-10393/Second.vue.txt"
                ),
            ),
        ];
        let linter = crate::Linter::with_preset(crate::LintPreset::Opinionated)
            .with_type_aware_lint(true)
            .with_enabled_rules(Some(vec!["type/no-floating-promises".into()]));
        let sequential: Vec<_> = inputs
            .iter()
            .map(|(name, source)| format!("{:?}", linter.lint_sfc(source, name)))
            .collect();
        let parallel = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..8)
                .map(|index| {
                    let (name, source) = inputs[index % 2];
                    let linter = &linter;
                    scope.spawn(move || format!("{:?}", linter.lint_sfc(source, name)))
                })
                .collect();
            handles
                .into_iter()
                .map(|handle| handle.join().unwrap())
                .collect::<Vec<_>>()
        });
        for (index, output) in parallel.iter().enumerate() {
            assert_eq!(output, &sequential[index % 2]);
        }
        linter.native_corsa.close_all();
    }
}
