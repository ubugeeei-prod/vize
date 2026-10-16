//! Final shutdown joins current owners and nonblocking configuration retirement.
use super::ServerState;
use std::sync::Arc;
use vize_canon::CorsaBridge;
use vize_l0::{String, cstr};

mod retired;
pub(super) use retired::RetiredOwners;

#[derive(Default)]
pub(super) struct NativeShutdown {
    pending: Vec<Arc<CorsaBridge>>,
    first_failure: Option<String>,
}

impl ServerState {
    pub(super) fn freeze_corsa_owner_for_exit(&self) {
        // Serialize the final-exit marker with every cache publication/removal.
        let _slot = self.corsa_bridge.write();
        self.corsa_finalizing
            .store(true, std::sync::atomic::Ordering::Release);
    }

    pub(crate) async fn shutdown_corsa_project_contexts(&self) -> Result<(), String> {
        // Serialize protocol and transport finalizers while retaining the exact
        // failed owner set. A retry must not mistake empty cache slots for a
        // previously acknowledged process close.
        let mut finalization = self.corsa_shutdown.lock().await;
        let states = self.freeze_project_contexts_for_exit();
        for state in std::iter::once(self).chain(states.iter().map(Arc::as_ref)) {
            if let Some(bridge) = state.corsa_bridge.write().take()
                && !finalization
                    .pending
                    .iter()
                    .any(|existing| Arc::ptr_eq(existing, &bridge))
            {
                finalization.pending.push(bridge);
            }
        }
        self.retire_project_contexts();
        let bridges = finalization.pending.clone();
        let (outcomes, retired) = futures::join!(
            futures::future::join_all(bridges.iter().map(|bridge| bridge.shutdown_for_exit())),
            self.corsa_retired.close_for_exit(),
        );
        finalization.pending.clear();
        let mut errors = Vec::new();
        for (bridge, outcome) in bridges.into_iter().zip(outcomes) {
            if let Err(error) = outcome {
                finalization.pending.push(bridge);
                errors.push(error);
            }
        }
        if let Err(error) = retired {
            errors.push(vize_canon::CorsaBridgeError::CommunicationError(error));
        }
        if !errors.is_empty() && finalization.first_failure.is_none() {
            finalization.first_failure = Some(cstr!(
                "Failed to close native project owners before LSP shutdown: {errors:?}"
            ));
        }
        match &finalization.first_failure {
            Some(error) => Err(error.clone()),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests;
