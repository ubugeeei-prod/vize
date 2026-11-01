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
        let mut bridges: Vec<_> = self.corsa_bridge_for_shutdown().into_iter().collect();
        for context in contexts {
            // Retired owners discard startups that complete after this point.
            context.mark_retired();
            let Some(bridge) = context
                .published_state()
                .and_then(|state| state.corsa_bridge_for_shutdown())
            else {
                continue;
            };
            if !bridges.iter().any(|known| Arc::ptr_eq(known, &bridge)) {
                bridges.push(bridge);
            }
        }
        for bridge in self.take_detached_corsa_bridges() {
            if bridge.is_initialized() {
                if !bridges.iter().any(|known| Arc::ptr_eq(known, &bridge)) {
                    bridges.push(bridge);
                }
            } else {
                // Pending startup never delays the outer reply (#10760).
                bridge.stop_pending_startup();
            }
        }
        self.retire_project_contexts();
        join_all(bridges.iter().map(|bridge| bridge.shutdown()))
            .await
            .into_iter()
            .collect()
    }

    fn corsa_bridge_for_shutdown(&self) -> Option<Arc<CorsaBridge>> {
        let bridge = self.corsa_bridge.read().clone()?;
        if bridge.is_initialized() {
            Some(bridge)
        } else {
            // Fence queued work and irrevocably queue cleanup behind entered
            // startup. Only already-initialized owners join the outer wait.
            bridge.stop_pending_startup();
            None
        }
    }
}
