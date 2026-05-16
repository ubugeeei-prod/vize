//! Core Corsa bridge implementation backed by `corsa-bind`.

use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};
#[allow(clippy::disallowed_types)]
use std::sync::{Arc, Mutex};
use vize_carton::profiler::{CacheStats, Profiler};
use vize_carton::{cstr, String};

use super::types::*;
use crate::corsa_client::CorsaProjectClient;

/// Bridge to Corsa for type checking and editor queries via project sessions.
#[allow(clippy::disallowed_types)]
pub struct CorsaBridge {
    /// Configuration
    config: CorsaBridgeConfig,
    /// Shared Corsa project-session state.
    client: Arc<Mutex<Option<CorsaProjectClient>>>,
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
            config,
            client: Arc::new(Mutex::new(None)),
            initialized: AtomicBool::new(false),
            profiler,
            cache_stats: CacheStats::new(),
        }
    }

    /// Spawn and initialize the Corsa process.
    pub async fn spawn(&self) -> Result<(), CorsaBridgeError> {
        let _timer = self.profiler.timer("corsa_spawn");

        if self.initialized.load(Ordering::SeqCst) {
            return Ok(());
        }

        let mut guard = lock_client(&self.client)?;
        if guard.is_some() {
            return Ok(());
        }

        let corsa_path = self
            .config
            .corsa_path
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned());
        let working_dir = self
            .config
            .working_dir
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned());

        let client = CorsaProjectClient::new(corsa_path.as_deref(), working_dir.as_deref())
            .map_err(CorsaBridgeError::SpawnFailed)?;
        *guard = Some(client);

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

        let mut guard = lock_client(&self.client)?;
        if let Some(client) = guard.as_mut() {
            client
                .shutdown()
                .map_err(CorsaBridgeError::CommunicationError)?;
        }
        *guard = None;

        self.initialized.store(false, Ordering::SeqCst);
        Ok(())
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
        if let Ok(mut guard) = self.client.lock() {
            if let Some(client) = guard.as_mut() {
                client.clear_diagnostics_cache();
            }
        }
        self.cache_stats.set_entries(0);
        self.cache_stats.reset();
    }

    /// Get hover information at a position.
    pub async fn hover(
        &self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<LspHover>, CorsaBridgeError> {
        let _timer = self.profiler.timer("corsa_hover");
        let uri = uri.to_owned();
        let result = self
            .with_client(move |client| {
                client
                    .hover_raw(uri.as_str(), line, character)
                    .map_err(CorsaBridgeError::CommunicationError)
            })
            .await?;

        if let Some(timer) = _timer {
            timer.record(&self.profiler);
        }

        result.map(parse_json_value::<LspHover>).transpose()
    }

    /// Get definition location for a symbol at a position.
    pub async fn definition(
        &self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Vec<LspLocation>, CorsaBridgeError> {
        let _timer = self.profiler.timer("corsa_definition");
        let uri = uri.to_owned();
        let result = self
            .with_client(move |client| {
                client
                    .definition_raw(uri.as_str(), line, character)
                    .map_err(CorsaBridgeError::CommunicationError)
            })
            .await?;

        if let Some(timer) = _timer {
            timer.record(&self.profiler);
        }

        if let Some(value) = result {
            return Ok(parse_json_value::<LspDefinitionResponse>(value)?.into_locations());
        }

        Ok(Vec::new())
    }

    /// Get references for a symbol at a position.
    pub async fn references(
        &self,
        uri: &str,
        line: u32,
        character: u32,
        include_declaration: bool,
    ) -> Result<Vec<LspLocation>, CorsaBridgeError> {
        let uri = uri.to_owned();
        let result = self
            .with_client(move |client| {
                client
                    .references_raw(uri.as_str(), line, character, include_declaration)
                    .map_err(CorsaBridgeError::CommunicationError)
            })
            .await?;

        if let Some(value) = result {
            return parse_json_value(value);
        }

        Ok(Vec::new())
    }

    /// Check whether rename is valid at a position.
    pub async fn prepare_rename(
        &self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, CorsaBridgeError> {
        let uri = uri.to_owned();
        self.with_client(move |client| {
            client
                .prepare_rename_raw(uri.as_str(), line, character)
                .map_err(CorsaBridgeError::CommunicationError)
        })
        .await
    }

    /// Rename a symbol at a position.
    pub async fn rename(
        &self,
        uri: &str,
        line: u32,
        character: u32,
        new_name: &str,
    ) -> Result<Option<Value>, CorsaBridgeError> {
        let uri = uri.to_owned();
        let new_name = new_name.to_owned();
        self.with_client(move |client| {
            client
                .rename_raw(uri.as_str(), line, character, new_name.as_str())
                .map_err(CorsaBridgeError::CommunicationError)
        })
        .await
    }

    /// Request import path updates before files are renamed.
    pub async fn will_rename_files(
        &self,
        renames: &[(&str, &str)],
    ) -> Result<Option<Value>, CorsaBridgeError> {
        let renames: Vec<(String, String)> = renames
            .iter()
            .map(|(old_uri, new_uri)| ((*old_uri).into(), (*new_uri).into()))
            .collect();

        self.with_client(move |client| {
            let renames_ref: Vec<(&str, &str)> = renames
                .iter()
                .map(|(old_uri, new_uri)| (old_uri.as_str(), new_uri.as_str()))
                .collect();
            client
                .will_rename_files_raw(&renames_ref)
                .map_err(CorsaBridgeError::CommunicationError)
        })
        .await
    }

    /// Get completion items at a position.
    pub async fn completion(
        &self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Vec<LspCompletionItem>, CorsaBridgeError> {
        let _timer = self.profiler.timer("corsa_completion");
        let uri = uri.to_owned();
        let result = self
            .with_client(move |client| {
                client
                    .completion_raw(uri.as_str(), line, character)
                    .map_err(CorsaBridgeError::CommunicationError)
            })
            .await?;

        if let Some(timer) = _timer {
            timer.record(&self.profiler);
        }

        if let Some(value) = result {
            return Ok(parse_json_value::<LspCompletionResponse>(value)?.items());
        }

        Ok(Vec::new())
    }

    async fn with_client<R, F>(&self, f: F) -> Result<R, CorsaBridgeError>
    where
        F: FnOnce(&mut CorsaProjectClient) -> Result<R, CorsaBridgeError>,
    {
        if !self.initialized.load(Ordering::SeqCst) {
            return Err(CorsaBridgeError::NotInitialized);
        }

        let mut guard = lock_client(&self.client)?;
        let client = guard.as_mut().ok_or(CorsaBridgeError::ProcessTerminated)?;
        f(client)
    }
}

impl Default for CorsaBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for CorsaBridge {
    fn drop(&mut self) {
        // Async shutdown is handled by explicit callers.
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

fn normalize_document_uri(name: &str) -> String {
    if name.starts_with("file://") {
        name.into()
    } else if name.starts_with('/') {
        cstr!("file://{name}")
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

fn parse_json_value<T>(value: Value) -> Result<T, CorsaBridgeError>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_value(value).map_err(|e| {
        CorsaBridgeError::CommunicationError(cstr!("Failed to parse Corsa result: {e}"))
    })
}

#[allow(clippy::disallowed_types)]
fn lock_client<'a>(
    client: &'a Arc<Mutex<Option<CorsaProjectClient>>>,
) -> Result<std::sync::MutexGuard<'a, Option<CorsaProjectClient>>, CorsaBridgeError> {
    client
        .lock()
        .map_err(|_| CorsaBridgeError::CommunicationError("Corsa client lock poisoned".into()))
}
