//! Graceful shutdown captures every accepted owner before ordinary retirement.

use futures::{Future, FutureExt, future::join_all};
use std::panic::AssertUnwindSafe;
use vize_canon::CorsaBridgeError;

use super::{Arc, Ordering, ServerState, Weak};

impl ServerState {
    fn take_shutdown_states(self: &Arc<Self>) -> Vec<Arc<Self>> {
        // This is global for every package and retained request view. A
        // startup already inside its init gate is retained until capture;
        // callers entering that gate later observe shutdown and refuse.
        self.project_contexts
            .shutting_down
            .store(true, Ordering::Release);
        self.project_contexts.retired.store(true, Ordering::Release);
        let contexts = {
            let mut contexts = self.project_contexts.contexts.lock();
            std::mem::take(&mut *contexts)
        };
        let mut states = vec![self.clone()];
        for context in contexts.into_values() {
            if let Some(state) = context.shutdown_state()
                && !states.iter().any(|previous| Arc::ptr_eq(previous, &state))
            {
                states.push(state);
            }
        }
        self.project_contexts.pending_config_reloads.lock().clear();
        self.project_contexts.routes.clear();
        self.project_contexts.configuration_attempts.lock().clear();
        states
    }

    pub(crate) async fn shutdown_project_contexts(
        self: &Arc<Self>,
    ) -> Result<(), CorsaBridgeError> {
        let root = self
            .project_contexts
            .owner
            .read()
            .as_ref()
            .and_then(Weak::upgrade)
            .unwrap_or_else(|| self.clone());
        let states = root.take_shutdown_states();
        finish_all(states.into_iter().map(|state| {
            // Capture the already published owner before waiting for startup
            // admission. A concurrent ordinary retirement may clear its slot,
            // but cannot release the owner accepted by this shutdown.
            let accepted = state.corsa_bridge.read().clone();
            async move {
                let bridge = {
                    // No registry, route, source, or configuration guard survives
                    // this wait. Accepted startup retains its slot until we own
                    // the same initialization gate.
                    let _initialization = state.corsa_init_lock.lock().await;
                    let bridge = state.corsa_bridge.write().take();
                    state.retire_project_owner();
                    #[cfg(feature = "experimental-source-navigation")]
                    state.retire_module_links(super::super::ModuleLinkRetirement::Shutdown);
                    bridge
                };
                // The initialization gate and every state guard are released
                // before IPC; attempt all contexts even if another owner fails.
                let mut bridges = accepted.into_iter().collect::<Vec<_>>();
                if let Some(bridge) = bridge
                    && !bridges
                        .iter()
                        .any(|previous| Arc::ptr_eq(previous, &bridge))
                {
                    bridges.push(bridge);
                }
                finish_all(
                    bridges
                        .into_iter()
                        .map(|bridge| async move { bridge.shutdown().await }),
                )
                .await
            }
        }))
        .await
    }
}

async fn finish_all(
    work: impl IntoIterator<Item = impl Future<Output = Result<(), CorsaBridgeError>>>,
) -> Result<(), CorsaBridgeError> {
    let mut failure = None;
    let guarded = work
        .into_iter()
        .map(|owner| AssertUnwindSafe(owner).catch_unwind());
    for outcome in join_all(guarded).await {
        let outcome = outcome.unwrap_or_else(|_| {
            Err(CorsaBridgeError::CommunicationError(vize_l0::cstr!(
                "compiler owner shutdown panicked"
            )))
        });
        if let Err(error) = outcome {
            tracing::warn!("compiler owner failed to finish shutdown: {error}");
            failure.get_or_insert(error);
        }
    }
    failure.map_or(Ok(()), Err)
}

#[cfg(test)]
mod tests;
