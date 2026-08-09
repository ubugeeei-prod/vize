//! Core Corsa bridge implementation backed by `corsa-bind`.

use serde_json::Value;
#[allow(clippy::disallowed_types)]
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use vize_carton::profiler::{CacheStats, Profiler};
use vize_carton::{String, cstr};

use super::session::build_client;
use super::types::{
    CorsaBridgeConfig, CorsaBridgeError, LspDiagnostic, TypeCheckResult, VIRTUAL_URI_SCHEME,
};
use super::worker::{BoundedWorker, WorkerError};
use crate::corsa_client::CorsaProjectClient;

mod language_features;

/// Bridge to Corsa for type checking and editor queries via project sessions.
#[allow(clippy::disallowed_types)]
pub struct CorsaBridge {
    /// Configuration
    config: CorsaBridgeConfig,
    /// Worker thread owning the synchronous Corsa project session.
    worker: BoundedWorker<Option<CorsaProjectClient>>,
    /// Whether the bridge is initialized
    initialized: AtomicBool,
    /// Profiler for performance tracking
    profiler: Profiler,
    /// Cache statistics
    cache_stats: CacheStats,
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
        let profiler = if config.enable_profiling {
            Profiler::enabled()
        } else {
            Profiler::new()
        };

        Self {
            worker: BoundedWorker::new("vize-corsa-bridge", None),
            config,
            initialized: AtomicBool::new(false),
            profiler,
            cache_stats: CacheStats::new(),
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

    /// Open a virtual document for type checking.
    pub async fn open_virtual_document(
        &self,
        name: &str,
        content: &str,
    ) -> Result<String, CorsaBridgeError> {
        let _timer = self.profiler.timer("open_virtual_document");
        let uri = normalize_document_uri(name);
        let content = content.to_owned();
        let result_uri = uri.clone();

        let cache_len = self
            .with_client(move |client| {
                client
                    .did_open_fast(uri.as_str(), content.as_str())
                    .map_err(CorsaBridgeError::CommunicationError)?;
                Ok(client.diagnostics_cache_len())
            })
            .await?;
        self.cache_stats.set_entries(cache_len as u64);

        if let Some(timer) = _timer {
            timer.record(&self.profiler);
        }

        Ok(result_uri)
    }

    /// Open or update a virtual document.
    pub async fn open_or_update_virtual_document(
        &self,
        name: &str,
        content: &str,
    ) -> Result<String, CorsaBridgeError> {
        self.open_virtual_document(name, content).await
    }

    /// Update a virtual document.
    pub async fn update_virtual_document(
        &self,
        uri: &str,
        content: &str,
        _version: i32,
    ) -> Result<(), CorsaBridgeError> {
        let _timer = self.profiler.timer("update_virtual_document");
        let uri = uri.to_owned();
        let content = content.to_owned();

        let cache_len = self
            .with_client(move |client| {
                client
                    .did_change(uri.as_str(), content.as_str())
                    .map_err(CorsaBridgeError::CommunicationError)?;
                Ok(client.diagnostics_cache_len())
            })
            .await?;
        self.cache_stats.set_entries(cache_len as u64);

        if let Some(timer) = _timer {
            timer.record(&self.profiler);
        }

        Ok(())
    }

    /// Close a virtual document.
    pub async fn close_virtual_document(&self, uri: &str) -> Result<(), CorsaBridgeError> {
        let uri = uri.to_owned();
        let cache_len = self
            .with_client(move |client| {
                client
                    .did_close(uri.as_str())
                    .map_err(CorsaBridgeError::CommunicationError)?;
                Ok(client.diagnostics_cache_len())
            })
            .await?;
        self.cache_stats.set_entries(cache_len as u64);
        Ok(())
    }

    /// Get diagnostics for a document.
    pub async fn get_diagnostics(&self, uri: &str) -> Result<Vec<LspDiagnostic>, CorsaBridgeError> {
        let uri = uri.to_owned();
        let (used_cache, cache_len, diagnostics) = self
            .with_client(move |client| {
                let fetch = client
                    .request_diagnostics_full(uri.as_str())
                    .map_err(CorsaBridgeError::CommunicationError)?;
                let diagnostics = convert_bridge_diagnostics(&fetch.diagnostics)?;
                Ok((
                    fetch.used_cache,
                    client.diagnostics_cache_len(),
                    diagnostics,
                ))
            })
            .await?;

        self.cache_stats.set_entries(cache_len as u64);
        if used_cache {
            self.cache_stats.hit();
        } else {
            self.cache_stats.miss();
        }

        Ok(diagnostics)
    }

    /// Type check a virtual TypeScript document.
    pub async fn type_check(
        &self,
        name: &str,
        content: &str,
    ) -> Result<TypeCheckResult, CorsaBridgeError> {
        let _timer = self.profiler.timer("type_check");

        let uri = self.open_virtual_document(name, content).await?;
        let diagnostics = self.get_diagnostics(&uri).await?;

        if let Some(timer) = _timer {
            timer.record(&self.profiler);
        }

        Ok(TypeCheckResult {
            diagnostics,
            source_map: None,
        })
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

        self.initialized.store(false, Ordering::SeqCst);
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

/// Batch type checker for checking multiple documents efficiently.
#[allow(clippy::disallowed_types)]
pub struct BatchTypeChecker {
    /// Bridge instance
    bridge: Arc<CorsaBridge>,
    /// Batch size
    batch_size: usize,
}

#[allow(clippy::disallowed_types)]
impl BatchTypeChecker {
    /// Create a new batch type checker.
    pub fn new(bridge: Arc<CorsaBridge>) -> Self {
        Self {
            bridge,
            batch_size: 10,
        }
    }

    /// Set batch size.
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Check multiple documents in batch.
    pub async fn check_batch(
        &self,
        documents: &[(String, String)],
    ) -> Vec<Result<TypeCheckResult, CorsaBridgeError>> {
        let _timer = self.bridge.profiler().timer("batch_type_check");
        let mut results = Vec::with_capacity(documents.len());

        for chunk in documents.chunks(self.batch_size) {
            let mut uris = Vec::with_capacity(chunk.len());
            for (name, content) in chunk {
                match self.bridge.open_virtual_document(name, content).await {
                    Ok(uri) => uris.push(Some(uri)),
                    Err(error) => {
                        results.push(Err(error));
                        uris.push(None);
                    }
                }
            }

            for uri in uris.into_iter().flatten() {
                match self.bridge.get_diagnostics(&uri).await {
                    Ok(diagnostics) => results.push(Ok(TypeCheckResult {
                        diagnostics,
                        source_map: None,
                    })),
                    Err(error) => results.push(Err(error)),
                }
            }
        }

        if let Some(timer) = _timer {
            timer.record(self.bridge.profiler());
        }

        results
    }
}

pub(super) fn normalize_document_uri(name: &str) -> String {
    if name.starts_with("file://") {
        name.into()
    } else if name.starts_with('/') {
        crate::file_uri::path_to_file_uri(std::path::Path::new(name))
    } else {
        cstr!("{VIRTUAL_URI_SCHEME}://{name}")
    }
}

fn convert_bridge_diagnostics(
    diagnostics: &[lsp_types::Diagnostic],
) -> Result<Vec<LspDiagnostic>, CorsaBridgeError> {
    diagnostics
        .iter()
        .map(|diagnostic| {
            let value = serde_json::to_value(diagnostic).map_err(|e| {
                CorsaBridgeError::CommunicationError(cstr!("Failed to encode diagnostic: {e}"))
            })?;
            serde_json::from_value(value).map_err(|e| {
                CorsaBridgeError::CommunicationError(cstr!("Failed to parse diagnostic: {e}"))
            })
        })
        .collect()
}

pub(super) fn parse_json_value<T>(value: Value) -> Result<T, CorsaBridgeError>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_value(value).map_err(|e| {
        CorsaBridgeError::CommunicationError(cstr!("Failed to parse Corsa result: {e}"))
    })
}
