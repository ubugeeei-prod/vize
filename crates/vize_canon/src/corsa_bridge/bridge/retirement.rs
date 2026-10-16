//! A past owner may fail protocol shutdown while still proving exact reaping.
#![expect(
    clippy::disallowed_types,
    reason = "retained SDK cleanup capabilities share process ownership"
)]
use super::{Arc, CorsaBridge, CorsaBridgeError, Ordering};
use crate::lsp_client::{
    retirement::{CloseOwners, observation_json},
    startup_control::StartupControl,
};
use corsa::process::ProcessTerminalObserver;
use serde_json::{Value, json};
use std::time::{Duration, Instant};
use vize_l0::{String, cstr};

#[derive(Clone, Debug)]
pub struct NativeRetirementOutcome {
    pub custody_complete: bool,
    pub raw_errors: Vec<String>,
    pub receipt: Value,
}

#[derive(Default)]
pub(super) struct RetirementState {
    owners: CloseOwners,
    errors: Vec<String>,
}

pub(super) struct CloseBatch {
    pub(super) outcome: NativeRetirementOutcome,
    pub(super) active_errors: Vec<String>,
    pub(super) completed: Vec<ProcessTerminalObserver>,
}

impl CorsaBridge {
    /// Retire a past owner without calling an unsuccessful protocol graceful.
    /// Missing acknowledgments retain exact SDK handles on this bridge for retry.
    pub async fn shutdown_for_retirement(
        &self,
    ) -> Result<NativeRetirementOutcome, CorsaBridgeError> {
        let batch = self.close_retirement_batch().await?;
        // Hand the owned raw rows to the typed caller in this same poll, then
        // prune their non-owning registry entries. A canceled queue waiter has
        // not reached this handoff and cannot erase its evidence.
        self.startup_control
            .acknowledge_publication(&batch.completed);
        Ok(batch.outcome)
    }

    pub(super) async fn close_retirement_batch(&self) -> Result<CloseBatch, CorsaBridgeError> {
        self.finalizing.store(true, Ordering::Release);
        let deadline = Instant::now() + Duration::from_millis(self.config.timeout_ms.max(1));
        let aborted = self.startup_control.abort_pending(deadline).await;
        if let Err(error) = &aborted {
            self.retirement
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .errors
                .push(error.clone());
        }
        let retained = Arc::clone(&self.retirement);
        let control = self.startup_control.clone();
        let initialized = Arc::clone(&self.initialized);
        let mirrors = Arc::clone(&self.editor_session);
        let remaining = deadline.saturating_duration_since(Instant::now());
        let result = self
            .submit_ready_with_bound(remaining.max(Duration::from_nanos(1)), move |slot| {
                if let Some(client) = slot.as_mut() {
                    // Publish capabilities before cleanup consumes any session.
                    let owners = client.retirement_owners();
                    retained
                        .lock()
                        .unwrap_or_else(|error| error.into_inner())
                        .owners
                        .append(owners);
                    let errors = client.shutdown_errors();
                    retained
                        .lock()
                        .unwrap_or_else(|error| error.into_inner())
                        .errors
                        .extend(errors);
                } else {
                    let handles = retained
                        .lock()
                        .unwrap_or_else(|error| error.into_inner())
                        .owners
                        .handles
                        .clone();
                    let errors = handles
                        .iter()
                        .filter(|handle| !handle.acknowledged())
                        .filter_map(|handle| handle.retry().err())
                        .collect::<Vec<_>>();
                    retained
                        .lock()
                        .unwrap_or_else(|error| error.into_inner())
                        .errors
                        .extend(errors);
                }
                *slot = None;
                mirrors.clear();
                initialized.store(false, Ordering::SeqCst);
                let pending = {
                    let mut state = retained.lock().unwrap_or_else(|error| error.into_inner());
                    state.owners.handles.retain(|handle| !handle.acknowledged());
                    state
                        .owners
                        .handles
                        .iter()
                        .filter_map(|handle| handle.observer())
                        .collect::<Vec<_>>()
                };
                // Failed constructors may leave only non-owning evidence while the
                // SDK Drop lane finishes. Wait on metadata only, never a Child or
                // handshake lock, within the same absolute exit deadline.
                for observer in control.unpublished_observers() {
                    if !observer.snapshot().custody_complete()
                        && !pending.iter().any(|owner| owner.same_owner(&observer))
                    {
                        observer.wait_for_terminal(deadline);
                    }
                }
                let state = retained.lock().unwrap_or_else(|error| error.into_inner());
                Ok(close_batch(&control, &state))
            })
            .await;
        match result {
            Ok(_) if remaining.is_zero() => Err(CorsaBridgeError::Timeout),
            Ok(batch) => Ok(batch),
            Err(error) => match aborted {
                Err(aborted) => Err(CorsaBridgeError::CommunicationError(cstr!(
                    "{aborted}; final worker cleanup: {error}"
                ))),
                Ok(()) => Err(error),
            },
        }
    }
}

fn close_batch(control: &StartupControl, state: &RetirementState) -> CloseBatch {
    let lifetime = control.lifetime_snapshot();
    let startup_pending = control.startup_custody_pending();
    let custody_complete = !startup_pending
        && !state.owners.missing_binding
        && state.owners.handles.is_empty()
        && lifetime.observed == lifetime.published + lifetime.pending.len() as u64
        && lifetime
            .pending
            .iter()
            .all(|owner| owner.custody_complete());
    let mut raw_errors = state.errors.clone();
    for owner in &lifetime.pending {
        raw_errors.extend(owner.raw_errors.iter().map(|error| cstr!("{error}")));
    }
    let receipt = json!({"custody_complete":custody_complete,"raw_errors":raw_errors,"expectedOwnedPids":lifetime.pending.iter().map(|owner| owner.expected_pid).collect::<Vec<_>>(),"children":lifetime.pending.iter().map(observation_json).collect::<Vec<_>>(),"attachedBindings":state.owners.attached_bindings,"missingAttachmentBinding":state.owners.missing_binding,"observedCount":lifetime.observed,"publishedCount":lifetime.published,"pendingCleanupHandles":state.owners.handles.len(),"pendingStartupCustody":startup_pending});
    CloseBatch {
        outcome: NativeRetirementOutcome {
            custody_complete,
            raw_errors,
            receipt,
        },
        active_errors: state.errors.clone(),
        completed: lifetime.completed,
    }
}
