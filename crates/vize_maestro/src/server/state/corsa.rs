//! Corsa bridge lifecycle (native TypeScript language features).
#![cfg(feature = "native")]

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use vize_canon::{CorsaBridge, CorsaBridgeConfig};

use super::ServerState;

impl ServerState {
    /// Try to claim the right to fire the "type checking unavailable"
    /// message. Returns true the first time it is called, false thereafter
    /// for the lifetime of the server. The caller is responsible for
    /// actually sending the message via the LSP client.
    pub fn claim_typecheck_unavailable_notice(&self) -> bool {
        !self
            .typecheck_unavailable_notified
            .swap(true, std::sync::atomic::Ordering::SeqCst)
    }

    /// Get or initialize the Corsa bridge.
    ///
    /// Returns `None` if Corsa is not available or failed to initialize.
    pub async fn get_corsa_bridge(&self) -> Option<Arc<CorsaBridge>> {
        if !self.is_lsp_typecheck_enabled() {
            tracing::info!(
                "Skipping Corsa bridge initialization because LSP typecheck is disabled"
            );
            return None;
        }

        // If already initialized successfully, return it
        if let Some(bridge) = self.corsa_bridge.read().clone() {
            return Some(bridge);
        }

        // If initialization already failed, don't retry
        if self.corsa_init_failed.load(Ordering::SeqCst) {
            return None;
        }

        let _guard = self.corsa_init_lock.lock().await;

        // Another request may have completed initialization while we were waiting.
        if let Some(bridge) = self.corsa_bridge.read().clone() {
            return Some(bridge);
        }

        if self.corsa_init_failed.load(Ordering::SeqCst) {
            return None;
        }

        // Get workspace root for Corsa configuration.
        let workspace_root = self.get_workspace_root();
        let type_checker_config = self.get_type_checker_config();

        let config = CorsaBridgeConfig {
            corsa_path: type_checker_config.runtime_path().map(PathBuf::from),
            working_dir: workspace_root,
            timeout_ms: 30000, // Corsa needs time to build project state on first load.
            ..Default::default()
        };
        let working_dir = config.working_dir.clone();
        let corsa_path = config.corsa_path.clone();
        let bridge = CorsaBridge::with_config(config);

        match crate::runtime::timeout(std::time::Duration::from_secs(5), bridge.spawn()).await {
            Ok(Ok(())) => {
                tracing::info!("corsa bridge initialized successfully");
                let bridge = Arc::new(bridge);
                *self.corsa_bridge.write() = Some(bridge.clone());
                Some(bridge)
            }
            Ok(Err(e)) => {
                let reason = vize_carton::cstr!(
                    "spawn failed: {e} (working_dir={working_dir:?}, corsa_path={corsa_path:?})"
                );
                tracing::warn!("corsa bridge {}", reason);
                self.record_corsa_init_failure(reason.as_str());
                None
            }
            Err(_) => {
                let reason = vize_carton::cstr!(
                    "spawn timed out after 5s (working_dir={working_dir:?}, corsa_path={corsa_path:?})"
                );
                tracing::warn!("corsa bridge {}", reason);
                self.record_corsa_init_failure(reason.as_str());
                None
            }
        }
    }

    /// Read the human-readable reason recorded the last time Corsa bridge
    /// initialization failed. Returns `None` if init has not failed.
    ///
    /// Used by handlers and tests to diagnose why the editor session fell
    /// back to the heuristic completion path (see #751).
    pub fn corsa_init_failure(&self) -> Option<Arc<str>> {
        self.corsa_init_failure_reason.read().clone()
    }

    pub(super) fn record_corsa_init_failure(&self, reason: &str) {
        *self.corsa_init_failure_reason.write() = Some(Arc::from(reason));
        self.corsa_init_failed.store(true, Ordering::SeqCst);
    }

    /// Check if the Corsa bridge is available (without initializing).
    pub fn has_corsa_bridge(&self) -> bool {
        self.is_lsp_typecheck_enabled() && self.corsa_bridge.read().is_some()
    }

    /// Drop a Corsa bridge whose backend process died mid-session so the next
    /// [`Self::get_corsa_bridge`] call spawns a fresh session (#3240).
    ///
    /// Passing the failed bridge (rather than clearing unconditionally) makes
    /// concurrent retirement idempotent: only callers still holding the live
    /// handle clear the slot, so a bridge respawned by another task is never
    /// discarded because of a stale failure report. The one-shot
    /// `corsa_init_failed` latch is left untouched — it marks *spawn*
    /// failures, and a session that ran long enough to die proves spawning
    /// works.
    pub fn retire_corsa_bridge(&self, failed: &Arc<CorsaBridge>) {
        let mut slot = self.corsa_bridge.write();
        if slot
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, failed))
        {
            *slot = None;
            tracing::warn!(
                "corsa bridge retired after a backend failure; a fresh session will be spawned on demand"
            );
        }
    }
}
