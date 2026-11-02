//! Core Corsa bridge implementation backed by `corsa-bind`.

use serde_json::Value;
#[expect(clippy::disallowed_types, reason = "shared across threads")]
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;
use vize_carton::cstr;
use vize_carton::profiler::{CacheStats, Profiler};

use super::types::{CorsaBridgeConfig, CorsaBridgeError, LspDefinitionResponse, LspLocation};
use super::worker::{BoundedWorker, WorkerError};
use crate::corsa_client::CorsaProjectClient;

mod call_hierarchy;
mod code_actions;
mod completion;
mod component_types;
mod declaration;
mod documents;
mod implementation;
mod initial_diagnostics;
pub use initial_diagnostics::{
    InitialApiDiagnostics, InitialApiDiagnosticsOutcome, PendingInitialApiDiagnostics,
};
mod inlay_hint;
mod language_features;
pub(in crate::corsa_bridge) mod startup;
pub(super) use documents::normalize_document_uri;

/// Bridge to Corsa for type checking and editor queries via project sessions.
#[expect(clippy::disallowed_types, reason = "dependency API uses std String")]
pub struct CorsaBridge {
    /// Configuration
    pub(super) config: CorsaBridgeConfig,
    /// Worker thread owning the synchronous Corsa project session.
    worker: BoundedWorker<Option<CorsaProjectClient>>,
    /// Read-only initial diagnostics never occupy the mutable foreground lane.
    initial_diagnostics_worker: BoundedWorker<()>,
    /// Whether the bridge is initialized
    pub(super) initialized: Arc<AtomicBool>,
    /// Conservative mutable-worker operation identity, including reads/errors.
    pub(super) client_operation_epoch: Arc<AtomicU64>,
    /// Startup admission and retained unpublished owner fence.
    pub(super) startup: Arc<startup::StartupState>,
    /// Whether the reusable editor LSP session may have read stale disk state.
    disk_project_state_dirty: AtomicBool,
    /// Profiler for performance tracking
    profiler: Profiler,
    /// Cache statistics
    cache_stats: CacheStats,
    /// Shared importer-scoped package topology used by every editor surface
    /// attached to this bridge.
    pub(super) package_route_resolver: crate::PackageRouteResolver,
    /// Private materialized mirrors and cache for this native session.
    pub(super) editor_session: Arc<super::EditorMirrorSession>,
    /// Editor owners can attach their project API to their own LSP process.
    pub(super) shared_editor_transport: bool,
}

#[expect(clippy::disallowed_types, reason = "dependency API uses std String")]
impl CorsaBridge {
    /// Trust captured route inputs between workspace watcher notifications.
    pub fn trust_watched_disk_inputs(&self) {
        self.editor_session.trust_watched_disk_inputs();
    }

    /// Create a new Corsa bridge with default configuration.
    pub fn new() -> Self {
        Self::with_config(CorsaBridgeConfig::default())
    }

    /// Create a new Corsa bridge with custom configuration.
    pub fn with_config(config: CorsaBridgeConfig) -> Self {
        let mut bridge =
            Self::with_config_and_package_routes(config, crate::PackageRouteResolver::default());
        bridge.shared_editor_transport = false;
        bridge
    }

    /// Create a bridge that shares package-route cache ownership with its LSP
    /// session (import indexing, mirror generation, and definition mapping).
    pub fn with_config_and_package_routes(
        config: CorsaBridgeConfig,
        package_route_resolver: crate::PackageRouteResolver,
    ) -> Self {
        Self::with_compiler_configuration(config, package_route_resolver, None)
    }

    /// Bind an admitted editor owner's immutable compiler authority. Direct
    /// callers passing `None` keep strict, live TypeScript config loading.
    pub fn with_compiler_configuration(
        config: CorsaBridgeConfig,
        package_route_resolver: crate::PackageRouteResolver,
        compiler_configuration: Option<Arc<crate::CompilerConfigurationSnapshot>>,
    ) -> Self {
        let profiler = if config.enable_profiling {
            Profiler::enabled()
        } else {
            Profiler::new()
        };

        let editor_session = Arc::new(super::EditorMirrorSession::with_compiler_configuration(
            compiler_configuration,
        ));
        Self {
            worker: BoundedWorker::new_with_keepalive(
                "vize-corsa-bridge",
                None,
                Arc::clone(&editor_session),
            ),
            initial_diagnostics_worker: BoundedWorker::new_with_keepalive(
                "vize-initial-api-read",
                (),
                Arc::clone(&editor_session),
            ),
            config,
            initialized: Arc::new(AtomicBool::new(false)),
            client_operation_epoch: Arc::new(AtomicU64::new(0)),
            startup: Arc::new(startup::StartupState::default()),
            disk_project_state_dirty: AtomicBool::new(false),
            profiler,
            cache_stats: CacheStats::new(),
            package_route_resolver,
            editor_session,
            shared_editor_transport: true,
        }
    }

    /// Check if bridge is initialized.
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::SeqCst)
    }

    /// Whether a cancelled or timed-out call still owns the synchronous lane.
    /// Retaining this bridge until it drains prevents parallel retry processes.
    pub fn is_draining(&self) -> bool {
        self.worker.is_draining() || self.initial_diagnostics_worker.is_draining()
    }

    /// Get profiler reference.
    pub fn profiler(&self) -> &Profiler {
        &self.profiler
    }

    /// Get cache statistics.
    pub fn cache_stats(&self) -> &CacheStats {
        &self.cache_stats
    }

    /// Clear diagnostics cache.
    pub fn clear_cache(&self) {
        let _ = self.submit(|slot| {
            if let Some(client) = slot.as_mut() {
                client.clear_diagnostics_cache();
            }
            Ok(())
        });
        self.cache_stats.set_entries(0);
        self.cache_stats.reset();
    }

    pub(super) async fn with_client<R, F>(&self, f: F) -> Result<R, CorsaBridgeError>
    where
        F: FnOnce(&mut CorsaProjectClient) -> Result<R, CorsaBridgeError> + Send + 'static,
        R: Send + 'static,
    {
        if !self.initialized.load(Ordering::SeqCst) {
            return Err(CorsaBridgeError::NotInitialized);
        }

        self.submit_ready_async(move |slot| match slot.as_mut() {
            Some(client) => f(client),
            None => Err(CorsaBridgeError::ProcessTerminated),
        })
        .await
    }

    /// Run `f` against the session on the worker thread under the configured
    /// deadline for the synchronous compatibility API.
    fn submit<R, F>(&self, f: F) -> Result<R, CorsaBridgeError>
    where
        F: FnOnce(&mut Option<CorsaProjectClient>) -> Result<R, CorsaBridgeError> + Send + 'static,
        R: Send + 'static,
    {
        // Clamped so a misconfigured `0` stays loud instead of silently
        // disabling the bridge; no value turns the bound off.
        let deadline = Duration::from_millis(self.config.timeout_ms.max(1));
        let epoch = Arc::clone(&self.client_operation_epoch);
        match self.worker.submit(deadline, move |slot| {
            epoch.fetch_add(1, Ordering::SeqCst);
            f(slot)
        }) {
            Ok(result) => result,
            Err(WorkerError::TimedOut) => {
                let bound = self.config.timeout_ms;
                tracing::warn!("corsa request outran the {bound}ms bridge bound; abandoned it");
                Err(CorsaBridgeError::Timeout)
            }
            Err(WorkerError::Stopped) => Err(CorsaBridgeError::ProcessTerminated),
        }
    }

    pub(super) async fn submit_async<R, F>(&self, f: F) -> Result<R, CorsaBridgeError>
    where
        F: FnOnce(&mut Option<CorsaProjectClient>) -> Result<R, CorsaBridgeError> + Send + 'static,
        R: Send + 'static,
    {
        let deadline = Duration::from_millis(self.config.timeout_ms.max(1));
        let epoch = Arc::clone(&self.client_operation_epoch);
        match self
            .worker
            .submit_async(deadline, move |slot| {
                epoch.fetch_add(1, Ordering::SeqCst);
                f(slot)
            })
            .await
        {
            Ok(result) => result,
            Err(WorkerError::TimedOut) => {
                let bound = self.config.timeout_ms;
                tracing::warn!("corsa request outran the {bound}ms bridge bound; abandoned it");
                Err(CorsaBridgeError::Timeout)
            }
            Err(WorkerError::Stopped) => Err(CorsaBridgeError::ProcessTerminated),
        }
    }

    pub(super) async fn submit_ready_async<R, F>(&self, f: F) -> Result<R, CorsaBridgeError>
    where
        F: FnOnce(&mut Option<CorsaProjectClient>) -> Result<R, CorsaBridgeError> + Send + 'static,
        R: Send + 'static,
    {
        let deadline = Duration::from_millis(self.config.timeout_ms.max(1));
        let epoch = Arc::clone(&self.client_operation_epoch);
        match self
            .worker
            .submit_ready_async(deadline, move |slot| {
                epoch.fetch_add(1, Ordering::SeqCst);
                f(slot)
            })
            .await
        {
            Ok(result) => result,
            Err(WorkerError::TimedOut) => {
                let bound = self.config.timeout_ms;
                tracing::warn!("corsa request outran the {bound}ms bridge bound; abandoned it");
                Err(CorsaBridgeError::Timeout)
            }
            Err(WorkerError::Stopped) => Err(CorsaBridgeError::ProcessTerminated),
        }
    }
}

impl Default for CorsaBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for CorsaBridge {
    fn drop(&mut self) {
        // Async shutdown is handled by explicit callers; dropping the worker
        // closes its job channel and the session is dropped on that thread.
    }
}

pub(super) fn parse_json_value<T>(value: Value) -> Result<T, CorsaBridgeError>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_value(value).map_err(|e| {
        CorsaBridgeError::CommunicationError(cstr!("Failed to parse Corsa result: {e}"))
    })
}

pub(super) fn parse_lsp_locations(value: Value) -> Result<Vec<LspLocation>, CorsaBridgeError> {
    Ok(parse_json_value::<LspDefinitionResponse>(value)?.into_locations())
}

#[cfg(test)]
mod tests {
    use super::parse_lsp_locations;

    #[test]
    fn declaration_and_implementation_location_links_normalize_to_locations() {
        let locations = parse_lsp_locations(serde_json::json!([{
            "targetUri": "file:///workspace/src/service.ts",
            "targetRange": {
                "start": { "line": 3, "character": 0 },
                "end": { "line": 5, "character": 1 }
            },
            "targetSelectionRange": {
                "start": { "line": 3, "character": 6 },
                "end": { "line": 3, "character": 21 }
            },
            "originSelectionRange": {
                "start": { "line": 0, "character": 10 },
                "end": { "line": 0, "character": 17 }
            }
        }]))
        .expect("location-link response");

        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0].uri, "file:///workspace/src/service.ts");
        assert_eq!(locations[0].range.start.line, 3);
        assert_eq!(locations[0].range.start.character, 6);
        assert_eq!(locations[0].range.end.line, 3);
        assert_eq!(locations[0].range.end.character, 21);
    }
}
