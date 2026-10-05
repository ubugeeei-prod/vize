//! Corsa bridge lifecycle (native TypeScript language features).
#![cfg(feature = "native")]

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use vize_canon::{CorsaBridge, CorsaBridgeConfig, CorsaBridgeError};

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
        let existing_bridge = { self.corsa_bridge.read().clone() };
        if let Some(bridge) = existing_bridge {
            if bridge.is_draining() {
                return None;
            }
            if bridge.is_initialized() && self.flush_corsa_disk_state_if_dirty(&bridge).await {
                return Some(bridge);
            }
            if bridge.is_initialized() {
                self.retire_corsa_bridge(&bridge);
            }
        }

        // If initialization already failed, don't retry
        if self.corsa_init_failed.load(Ordering::SeqCst) {
            return None;
        }

        let _guard = self.corsa_init_lock.lock().await;

        // Another request may have completed initialization while we were waiting.
        let existing_bridge = { self.corsa_bridge.read().clone() };
        if let Some(bridge) = existing_bridge {
            if bridge.is_draining() {
                return None;
            }
            if bridge.is_initialized() && self.flush_corsa_disk_state_if_dirty(&bridge).await {
                return Some(bridge);
            }
            if bridge.is_initialized() {
                self.retire_corsa_bridge(&bridge);
            }
        }

        if self.corsa_init_failed.load(Ordering::SeqCst) {
            return None;
        }

        // A config/root/watch notification may run while spawn yields. Do not
        // publish its old session or poison the newer configuration on failure.
        let generation = self.corsa_environment_revision.load(Ordering::Acquire);
        // Get workspace root for Corsa configuration.
        let workspace_root = self.get_workspace_root();
        let (type_checker_config, request_timeout_ms) = self.type_checker_config.read().clone();
        let project = vize_carton::config::ProjectModel::new(
            workspace_root.as_deref(),
            None,
            &type_checker_config,
        );
        let tsconfig_path = project.tsconfig().map(PathBuf::from);

        let config = CorsaBridgeConfig {
            corsa_path: type_checker_config.runtime_path().map(PathBuf::from),
            working_dir: workspace_root,
            tsconfig_path,
            timeout_ms: request_timeout_ms,
            ..Default::default()
        };
        let working_dir = config.working_dir.clone();
        let corsa_path = config.corsa_path.clone();
        let existing = { self.corsa_bridge.read().clone() };
        let bridge = existing.unwrap_or_else(|| {
            Arc::new(CorsaBridge::with_config_and_package_routes(
                config,
                self.package_route_resolver.lock().clone(),
            ))
        });
        // Keep a cancelled startup's worker/session owner. Until it drains,
        // later callers refuse rather than creating a second backend process.
        *self.corsa_bridge.write() = Some(Arc::clone(&bridge));

        // The bridge's worker-owned async reply yields while its synchronous
        // handshake drains, and enforces the configured deadline (#8012).
        let spawned = bridge.spawn().await;
        if self.corsa_environment_changes.load(Ordering::Acquire) != 0
            || generation != self.corsa_environment_revision.load(Ordering::Acquire)
        {
            self.discard_corsa_startup(&bridge);
            tracing::debug!("discarding Corsa startup superseded by a project change");
            return None;
        }
        match spawned {
            Ok(()) => {
                tracing::info!("corsa bridge initialized successfully");
                if self.flush_corsa_disk_state_if_dirty(&bridge).await {
                    Some(bridge)
                } else {
                    self.retire_corsa_bridge(&bridge);
                    None
                }
            }
            Err(CorsaBridgeError::Timeout) => {
                self.discard_corsa_startup(&bridge);
                let reason = vize_l0::cstr!(
                    "spawn timed out after {request_timeout_ms}ms (working_dir={working_dir:?}, corsa_path={corsa_path:?})"
                );
                tracing::warn!("corsa bridge {}", reason);
                self.record_corsa_init_failure(reason.as_str());
                None
            }
            Err(e) => {
                self.discard_corsa_startup(&bridge);
                let reason = vize_l0::cstr!(
                    "spawn failed: {e} (working_dir={working_dir:?}, corsa_path={corsa_path:?})"
                );
                tracing::warn!("corsa bridge {}", reason);
                self.record_corsa_init_failure(reason.as_str());
                None
            }
        }
    }

    /// Mark the reusable editor-side project view dirty without touching the
    /// backend from the file-operation notification itself.
    ///
    /// `workspace/didCreateFiles`, `didRenameFiles`, and watched declaration
    /// changes run on the same foreground LSP queue as ordinary requests. A
    /// direct Corsa call there can park `workspace/symbol` or completion behind
    /// a backend timeout even though those requests do not need the
    /// invalidation to answer. The next Corsa-using request flushes this bit
    /// instead.
    pub(crate) fn mark_corsa_disk_state_dirty(&self) {
        let _change = self.corsa_environment_change();
        let bridge = { self.corsa_bridge.read().clone() };
        if let Some(bridge) = bridge {
            bridge.mark_disk_project_state_dirty();
        }
    }

    async fn flush_corsa_disk_state_if_dirty(&self, bridge: &Arc<CorsaBridge>) -> bool {
        match bridge.flush_disk_project_state_if_dirty().await {
            Ok(()) => true,
            Err(error) => {
                tracing::warn!("failed to invalidate cached Corsa disk project state: {error}");
                false
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
        self.is_lsp_typecheck_enabled()
            && self
                .corsa_bridge
                .read()
                .as_ref()
                .is_some_and(|bridge| bridge.is_initialized())
    }

    /// Drop the cached editor session after a Vue file disappears on disk.
    /// Its private mirror is rebuilt by the next Corsa request. This must not
    /// wait for the old worker while handling the file-operation notification.
    pub(crate) fn retire_corsa_bridge_for_deleted_vue_files(&self) {
        let bridge = { self.corsa_bridge.read().clone() };
        if let Some(bridge) = bridge {
            self.retire_corsa_bridge(&bridge);
        }
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
            let _change = self.corsa_environment_change();
            *slot = None;
            tracing::warn!(
                "corsa bridge retired after a backend failure; a fresh session will be spawned on demand"
            );
        }
    }

    /// Configuration owns the cached process. Retire without waiting for IPC;
    /// worker keepalive lets an abandoned old session finish on its own lane.
    pub(super) fn retire_corsa_configuration(&self) {
        *self.corsa_bridge.write() = None;
        self.corsa_init_failed.store(false, Ordering::SeqCst);
        *self.corsa_init_failure_reason.write() = None;
        self.typecheck_unavailable_notified
            .store(false, Ordering::SeqCst);
    }

    fn discard_corsa_startup(&self, pending: &Arc<CorsaBridge>) {
        let mut slot = self.corsa_bridge.write();
        if slot
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, pending))
        {
            *slot = None;
        }
    }
}

#[cfg(all(test, unix))]
mod tests;
