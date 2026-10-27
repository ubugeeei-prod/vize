//! Outer shutdown waits for every compiler that finished startup (#10760).

use futures::future::join_all;
use vize_canon::{CorsaBridge, CorsaBridgeError};

use super::{Arc, Ordering, ServerState};

impl ServerState {
    /// Retire every owner, then await the Shutdown handshake of each compiler
    /// that had finished startup. A startup still pending is discarded as on
    /// retirement, so a held native startup never delays the outer reply.
    pub(crate) async fn shutdown_project_contexts(&self) -> Result<(), CorsaBridgeError> {
        self.project_contexts
            .shutting_down
            .store(true, Ordering::Release);
        self.project_contexts.retired.store(true, Ordering::Release);
        let contexts: Vec<_> = self
            .project_contexts
            .contexts
            .lock()
            .values()
            .cloned()
            .collect();
        let mut bridges: Vec<_> = self.initialized_corsa_bridge().into_iter().collect();
        for context in contexts {
            // Retired owners discard startups that complete after this point.
            context.mark_retired();
            let Some(bridge) = context
                .published_state()
                .and_then(|state| state.initialized_corsa_bridge())
            else {
                continue;
            };
            if !bridges.iter().any(|known| Arc::ptr_eq(known, &bridge)) {
                bridges.push(bridge);
            }
        }
        self.retire_project_contexts();
        join_all(bridges.iter().map(|bridge| bridge.shutdown()))
            .await
            .into_iter()
            .collect()
    }

    fn initialized_corsa_bridge(&self) -> Option<Arc<CorsaBridge>> {
        self.corsa_bridge
            .read()
            .clone()
            .filter(|bridge| bridge.is_initialized())
    }
}
