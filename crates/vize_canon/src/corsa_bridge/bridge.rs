//! Core Corsa bridge implementation backed by `corsa-bind`.

use serde_json::Value;
#[allow(clippy::disallowed_types)]
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use vize_carton::cstr;
use vize_carton::profiler::{CacheStats, Profiler};

use super::session::build_client;
use super::types::{CorsaBridgeConfig, CorsaBridgeError, LspDefinitionResponse, LspLocation};
use super::worker::{BoundedWorker, WorkerError};
use crate::corsa_client::CorsaProjectClient;

#[path = "bridge/call_hierarchy.rs"]
mod call_hierarchy;
#[path = "bridge/declaration.rs"]
mod declaration;
#[path = "bridge/documents.rs"]
mod documents;
#[path = "bridge/implementation.rs"]
mod implementation;
mod language_features;
pub(super) use documents::normalize_document_uri;

/// Bridge to Corsa for type checking and editor queries via project sessions.
#[allow(clippy::disallowed_types)]
pub struct CorsaBridge {
    /// Configuration
    pub(super) config: CorsaBridgeConfig,
    /// Worker thread owning the synchronous Corsa project session.
    worker: BoundedWorker<Option<CorsaProjectClient>>,
    /// Whether the bridge is initialized
    initialized: AtomicBool,
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
}

#[allow(clippy::disallowed_types)]
impl CorsaBridge {
    /// Create a new Corsa bridge with default configuration.
    pub fn new() -> Self {
        Self::with_config(CorsaBridgeConfig::default())
    }

    /// Create a new Corsa bridge with custom configuration.
    #[allow(clippy::disallowed_types)]
    pub fn with_config(config: CorsaBridgeConfig) -> Self {
        Self::with_config_and_package_routes(config, crate::PackageRouteResolver::default())
    }

    /// Create a bridge that shares package-route cache ownership with its LSP
    /// session (import indexing, mirror generation, and definition mapping).
    pub fn with_config_and_package_routes(
        config: CorsaBridgeConfig,
        package_route_resolver: crate::PackageRouteResolver,
    ) -> Self {
        let profiler = if config.enable_profiling {
            Profiler::enabled()
        } else {
            Profiler::new()
        };

        let editor_session = Arc::new(super::EditorMirrorSession::new());
        Self {
            worker: BoundedWorker::new_with_keepalive(
                "vize-corsa-bridge",
                None,
                Arc::clone(&editor_session),
            ),
            config,
            initialized: AtomicBool::new(false),
            disk_project_state_dirty: AtomicBool::new(false),
            profiler,
            cache_stats: CacheStats::new(),
            package_route_resolver,
            editor_session,
        }
    }

    /// Spawn and initialize the Corsa process, bounded by `timeout_ms`. The
    /// handshake is synchronous IPC: a backend that accepts stdio and never
    /// answers otherwise blocks until `corsa`'s own 30s backstop gives up.
    pub async fn spawn(&self) -> Result<(), CorsaBridgeError> {
        let _timer = self.profiler.timer("corsa_spawn");

        if self.initialized.load(Ordering::SeqCst) {
            return Ok(());
        }

        let config = self.config.clone();
        self.submit(move |slot| {
            if slot.is_some() {
                return Ok(());
            }
            *slot = Some(build_client(&config)?);
            Ok(())
        })?;

        self.initialized.store(true, Ordering::SeqCst);

        if let Some(timer) = _timer {
            timer.record(&self.profiler);
        }

        Ok(())
    }

    /// Shutdown the bridge.
    pub async fn shutdown(&self) -> Result<(), CorsaBridgeError> {
        if !self.initialized.load(Ordering::SeqCst) {
            return Ok(());
        }

        let result = self.submit(|slot| {
            let outcome = match slot.as_mut() {
                Some(client) => client
                    .shutdown()
                    .map_err(CorsaBridgeError::CommunicationError),
                None => Ok(()),
            };
            *slot = None;
            outcome
        });

        if !matches!(result, Err(CorsaBridgeError::Timeout)) {
            self.initialized.store(false, Ordering::SeqCst);
            self.editor_session.clear();
        }
        result
    }

    /// Check if bridge is initialized.
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::SeqCst)
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

        self.submit(move |slot| match slot.as_mut() {
            Some(client) => f(client),
            None => Err(CorsaBridgeError::ProcessTerminated),
        })
    }

    /// Run `f` against the session on the worker thread under the configured
    /// deadline — the one place the bound is real. The wait blocks on purpose:
    /// the job never yields, so an async `timeout` around it could never be
    /// polled, and making it yield would activate #3377's shard-guard hazard.
    /// See [`super::worker`] for the full argument.
    fn submit<R, F>(&self, f: F) -> Result<R, CorsaBridgeError>
    where
        F: FnOnce(&mut Option<CorsaProjectClient>) -> Result<R, CorsaBridgeError> + Send + 'static,
        R: Send + 'static,
    {
        // Clamped so a misconfigured `0` stays loud instead of silently
        // disabling the bridge; no value turns the bound off.
        let deadline = Duration::from_millis(self.config.timeout_ms.max(1));
        match self.worker.submit(deadline, f) {
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
