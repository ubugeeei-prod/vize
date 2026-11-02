//! Real startup owns its queued job, mirror revision and late native client.
#![expect(clippy::disallowed_types, reason = "worker ownership crosses threads")]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use super::CorsaBridge;
use crate::corsa_bridge::EditorMirrorSession;
use crate::corsa_bridge::session::build_client;
use crate::corsa_bridge::types::CorsaBridgeError;

#[derive(Default)]
pub(in crate::corsa_bridge) struct StartupState(Mutex<State>);

#[derive(Default)]
struct State {
    epoch: u64,
    pending: usize,
    seed: bool,
    retained: bool,
    stopping: bool,
    clear_pending: bool,
}

/// Moves into the queued closure; an unentered cancellation drops it too.
pub(in crate::corsa_bridge) struct StartupLease {
    state: Arc<StartupState>,
    session: Arc<EditorMirrorSession>,
    epoch: u64,
    initial: bool,
    entered: bool,
}

impl StartupState {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub(in crate::corsa_bridge) fn reserve(
        self: &Arc<Self>,
        session: Arc<EditorMirrorSession>,
        initial: bool,
        initialized: bool,
    ) -> Option<StartupLease> {
        let mut state = self.lock();
        if state.stopping
            || state.seed
            || (state.retained && !initialized)
            || (initial && (initialized || state.pending != 0 || state.retained))
        {
            return None;
        }
        state.pending += 1;
        state.seed = initial;
        Some(StartupLease {
            state: Arc::clone(self),
            session,
            epoch: state.epoch,
            initial,
            entered: false,
        })
    }

    fn pending(&self, initialized: bool) -> usize {
        let state = self.lock();
        state
            .pending
            .max(usize::from(state.retained && !initialized))
    }

    fn stop(&self, initialized: &AtomicBool) -> bool {
        let mut state = self.lock();
        state.epoch = state.epoch.wrapping_add(1);
        state.stopping = true;
        let needed = state.pending != 0 || state.retained || initialized.load(Ordering::SeqCst);
        initialized.store(false, Ordering::SeqCst);
        if !needed {
            state.stopping = false;
        }
        needed
    }

    fn stopped(&self, session: &EditorMirrorSession) {
        let mut state = self.lock();
        state.retained = false;
        if state.seed {
            // An owned, unentered seed still leases this exact mirror. Its
            // stale epoch refuses startup and its Drop performs the clear.
            state.clear_pending = true;
        } else {
            session.clear();
            state.stopping = false;
        }
    }

    fn finish_shutdown<T>(
        &self,
        slot: &mut Option<T>,
        session: &EditorMirrorSession,
        shutdown: impl FnOnce(&mut T) -> Result<(), CorsaBridgeError>,
    ) -> Result<(), CorsaBridgeError> {
        if let Some(client) = slot.as_mut() {
            shutdown(client)?;
        } else if self.lock().retained {
            return Err(CorsaBridgeError::CommunicationError(
                "failed native startup returned no owner to join".into(),
            ));
        }
        *slot = None;
        self.stopped(session);
        Ok(())
    }
}

impl StartupLease {
    pub(in crate::corsa_bridge) fn belongs_to(
        &self,
        state: &Arc<StartupState>,
        session: &Arc<EditorMirrorSession>,
    ) -> bool {
        Arc::ptr_eq(&self.state, state) && Arc::ptr_eq(&self.session, session)
    }

    pub(in crate::corsa_bridge) fn is_current(&self) -> bool {
        let state = self.state.lock();
        state.epoch == self.epoch && !state.stopping
    }

    pub(in crate::corsa_bridge) fn enter(&mut self) -> bool {
        if !self.is_current() {
            return false;
        }
        // Once physical startup can run, even an error may own cleanup. Never
        // roll its mirror back from the caller's or cancellation's Drop.
        self.entered = true;
        true
    }

    pub(in crate::corsa_bridge) fn arm_before_entry(&self) -> Result<(), CorsaBridgeError> {
        self.session
            .arm_retention_before_startup()
            .inspect_err(|_| {
                let state = self.state.lock();
                if !self.entered && !state.retained {
                    self.session.clear();
                }
            })
    }

    pub(in crate::corsa_bridge) fn retain(&self) {
        self.state.lock().retained = true;
    }

    pub(in crate::corsa_bridge) fn receipt(&self) -> StartupReceipt {
        StartupReceipt {
            state: Arc::clone(&self.state),
            session: Arc::clone(&self.session),
            epoch: self.epoch,
        }
    }

    pub(in crate::corsa_bridge) fn retain_after_failure(&self) {
        self.state.lock().retained = true;
        self.session.retain_after_failed_shutdown();
    }

    pub(in crate::corsa_bridge) fn publish(
        &self,
        initialized: &AtomicBool,
        current: bool,
    ) -> Result<(), CorsaBridgeError> {
        let state = self.state.lock();
        if !current || state.epoch != self.epoch || state.stopping {
            return Err(CorsaBridgeError::ProcessTerminated);
        }
        initialized.store(true, Ordering::SeqCst);
        Ok(())
    }
}

pub(in crate::corsa_bridge) struct StartupReceipt {
    state: Arc<StartupState>,
    session: Arc<EditorMirrorSession>,
    epoch: u64,
}

impl StartupReceipt {
    pub(in crate::corsa_bridge) fn is_current(&self, bridge: &CorsaBridge) -> bool {
        if !Arc::ptr_eq(&self.state, &bridge.startup)
            || !Arc::ptr_eq(&self.session, &bridge.editor_session)
            || !bridge.is_initialized()
        {
            return false;
        }
        self.is_ready(&bridge.initialized)
    }

    pub(in crate::corsa_bridge) fn is_ready(&self, initialized: &AtomicBool) -> bool {
        let state = self.state.lock();
        initialized.load(Ordering::SeqCst) && state.epoch == self.epoch && !state.stopping
    }
}

impl Drop for StartupLease {
    fn drop(&mut self) {
        let mut state = self.state.lock();
        if self.initial {
            // The exclusive seed reservation prevents another startup from
            // owning this unstarted revision. Failed graph/admission work must
            // not consume its initial materialized delta before eager retry.
            if !self.entered && !state.retained {
                self.session.clear();
            }
            state.seed = false;
        }
        state.pending -= 1;
        if state.clear_pending && !state.seed && !state.retained {
            self.session.clear();
            state.clear_pending = false;
            state.stopping = false;
        }
    }
}

impl CorsaBridge {
    /// Prepared, queued and entered startups remain owned until their guard
    /// drops. A late unpublished native owner also requires explicit shutdown.
    pub fn pending_startups(&self) -> usize {
        self.startup.pending(self.is_initialized())
    }

    /// Fence a stored but uninitialized owner without delaying outer shutdown.
    /// Cleanup is irrevocably queued on its existing worker; no caller future
    /// or new timer owns the cleanup's lifetime (#10760).
    pub fn stop_pending_startup(&self) -> bool {
        if self.is_initialized() || !self.startup.stop(&self.initialized) {
            return false;
        }
        let startup = Arc::clone(&self.startup);
        let session = Arc::clone(&self.editor_session);
        let epoch = Arc::clone(&self.client_operation_epoch);
        let queued = self.worker.enqueue_retirement(move |slot| {
            epoch.fetch_add(1, Ordering::SeqCst);
            let result = startup.finish_shutdown(slot, &session, |client| {
                client
                    .shutdown()
                    .map_err(CorsaBridgeError::CommunicationError)
            });
            if let Err(error) = result {
                session.retain_after_failed_shutdown();
                tracing::warn!(%error, "pending Corsa startup cleanup did not join its owner");
            }
        });
        if queued.is_err() {
            // A closed receiver can still be unwinding its owned state. It is
            // not a native join witness, so preserve the exact files.
            self.editor_session.retain_after_failed_shutdown();
        }
        true
    }

    /// Spawn a REAL session under the existing worker's original deadline.
    pub async fn spawn(&self) -> Result<(), CorsaBridgeError> {
        let timer = self.profiler.timer("corsa_spawn");
        if self.is_initialized() {
            return Ok(());
        }
        let mut lease = self
            .startup
            .reserve(Arc::clone(&self.editor_session), false, false)
            .ok_or(CorsaBridgeError::ProcessTerminated)?;
        let config = self.config.clone();
        let shared = self.shared_editor_transport;
        let initialized = Arc::clone(&self.initialized);
        self.submit_async(move |slot| {
            // This remains inside the queued owner: cleanup cannot remove the
            // marker between admission and physical entry on another worker.
            lease.arm_before_entry()?;
            if !lease.enter() {
                return Err(CorsaBridgeError::ProcessTerminated);
            }
            if slot.is_none() {
                // This eager constructor has not received the Canon mirror.
                // Its error keeps the existing retry/error ownership law;
                // only the seeded factory can expose this graph on failure.
                *slot = Some(build_client(&config, shared)?);
            }
            lease.retain();
            lease.publish(&initialized, true)
        })
        .await?;
        if let Some(timer) = timer {
            timer.record(&self.profiler);
        }
        Ok(())
    }

    /// Fence even an uninitialized startup, then queue cleanup behind entered
    /// IPC. No blocking Drop or second deadline is introduced.
    pub async fn shutdown(&self) -> Result<(), CorsaBridgeError> {
        if !self.startup.stop(&self.initialized) {
            return Ok(());
        }
        let startup = Arc::clone(&self.startup);
        let session = Arc::clone(&self.editor_session);
        self.submit_ready_async(move |slot| {
            // Keep an unknown-live owner and its mirror on a shutdown error;
            // dropping a caller is not a successful transport join.
            startup.finish_shutdown(slot, &session, |client| {
                let outcome = client
                    .shutdown()
                    .map_err(CorsaBridgeError::CommunicationError);
                if outcome.is_err() {
                    session.retain_after_failed_shutdown();
                }
                outcome
            })
        })
        .await
    }
}

#[cfg(test)]
mod tests;
