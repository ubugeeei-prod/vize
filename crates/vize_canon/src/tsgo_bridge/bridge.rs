//! Core tsgo bridge implementation.
//!
//! Contains the `TsgoBridge` struct and its methods for spawning,
//! communicating with, and shutting down the tsgo process. Also includes
//! the `BatchTypeChecker` for efficient multi-document checking.

#[allow(clippy::disallowed_types)]
use std::sync::Arc;
use std::{
    path::PathBuf,
    process::Stdio,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};

use dashmap::DashMap;
use serde_json::{json, Value};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    process::{Child as TokioChild, Command as TokioCommand},
    sync::{oneshot, Mutex},
};

use vize_carton::profiler::{CacheStats, Profiler};

use super::{
    protocol::{JsonRpcNotification, JsonRpcRequest},
    reader::{self, DiagnosticsCache, OpenDocuments, PendingMap, SharedStdin},
    types::*,
};
use vize_carton::cstr;
use vize_carton::String;
use vize_carton::ToCompactString;

/// Bridge to tsgo for type checking via LSP.
pub struct TsgoBridge {
    /// Configuration
    config: TsgoBridgeConfig,
    /// tsgo process handle
    process: Mutex<Option<TokioChild>>,
    /// Stdin writer (shared with reader task for responding to server requests)
    stdin: SharedStdin,
    /// Request ID counter
    request_id: AtomicU64,
    /// Pending requests (wrapped in Arc for sharing with reader task)
    pending: PendingMap,
    /// Whether the bridge is initialized
    initialized: AtomicBool,
    /// Profiler for performance tracking
    profiler: Profiler,
    /// Cache statistics
    cache_stats: CacheStats,
    /// Cached diagnostics by URI (wrapped in Arc for sharing with reader task)
    diagnostics_cache: DiagnosticsCache,
    /// Tracks open document URIs and their versions
    open_documents: OpenDocuments,
}

impl TsgoBridge {
    /// Create a new tsgo bridge with default configuration.
    pub fn new() -> Self {
        Self::with_config(TsgoBridgeConfig::default())
    }

    /// Create a new tsgo bridge with custom configuration.
    #[allow(clippy::disallowed_types)]
    pub fn with_config(config: TsgoBridgeConfig) -> Self {
        let profiler = if config.enable_profiling {
            Profiler::enabled()
        } else {
            Profiler::new()
        };

        Self {
            config,
            process: Mutex::new(None),
            stdin: Arc::new(Mutex::new(None)),
            request_id: AtomicU64::new(1),
            pending: Arc::new(DashMap::new()),
            initialized: AtomicBool::new(false),
            profiler,
            cache_stats: CacheStats::new(),
            diagnostics_cache: Arc::new(DashMap::new()),
            open_documents: Arc::new(DashMap::new()),
        }
    }

    /// Spawn and initialize the tsgo process.
    pub async fn spawn(&self) -> Result<(), TsgoBridgeError> {
        let _timer = self.profiler.timer("tsgo_spawn");

        // Find tsgo executable
        tracing::info!("tsgo_bridge: finding tsgo path...");
        let tsgo_path = self.find_tsgo_path()?;
        tracing::info!("tsgo_bridge: found tsgo at {:?}", tsgo_path);

        // Spawn tsgo with LSP mode
        let mut cmd = TokioCommand::new(&tsgo_path);
        cmd.arg("--lsp")
            .arg("--stdio")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()); // Capture stderr for debugging

        if let Some(ref working_dir) = self.config.working_dir {
            tracing::info!("tsgo_bridge: working_dir = {:?}", working_dir);
            cmd.current_dir(working_dir);
        }

        tracing::info!("tsgo_bridge: spawning process...");
        let mut child = cmd.spawn().map_err(|e| {
            TsgoBridgeError::SpawnFailed(cstr!("Failed to spawn tsgo at {tsgo_path:?}: {e}"))
        })?;
        tracing::info!("tsgo_bridge: process spawned");

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| TsgoBridgeError::SpawnFailed("Failed to get stdin".into()))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| TsgoBridgeError::SpawnFailed("Failed to get stdout".into()))?;

        let stderr = child.stderr.take();

        *self.process.lock().await = Some(child);
        *self.stdin.lock().await = Some(BufWriter::new(stdin));

        // Start stderr reader task (for debugging)
        if let Some(stderr) = stderr {
            tokio::spawn(async move {
                let mut reader = BufReader::new(stderr);
                #[allow(clippy::disallowed_types)]
                let mut line = std::string::String::new();
                loop {
                    line.clear();
                    match reader.read_line(&mut line).await {
                        Ok(0) => break,
                        Ok(_) => tracing::warn!("tsgo stderr: {}", line.trim()),
                        Err(_) => break,
                    }
                }
            });
        }

        // Start response reader task
        tracing::info!("tsgo_bridge: starting reader task...");
        #[allow(clippy::disallowed_types)]
        {
            reader::start_reader_task(
                stdout,
                Arc::clone(&self.pending),
                Arc::clone(&self.diagnostics_cache),
                Arc::clone(&self.stdin),
            );
        }

        // Initialize LSP
        tracing::info!("tsgo_bridge: calling initialize()...");
        self.initialize().await?;
        tracing::info!("tsgo_bridge: initialized");

        self.initialized.store(true, Ordering::SeqCst);

        if let Some(timer) = _timer {
            timer.record(&self.profiler);
        }

        Ok(())
    }

    /// Find tsgo executable path.
    ///
    /// Search order:
    /// 1. Explicit config.tsgo_path
    /// 2. Native binary in node_modules (platform-specific) - walks up parent dirs
    /// 3. Local node_modules/.bin/tsgo (requires node)
    /// 4. Global PATH
    fn find_tsgo_path(&self) -> Result<PathBuf, TsgoBridgeError> {
        // 1. Use explicit path if provided
        if let Some(ref path) = self.config.tsgo_path {
            if path.exists() {
                return Ok(path.clone());
            }
        }

        let base_dir = self
            .config
            .working_dir
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        // Platform-specific paths for @typescript/native-preview
        let platform_suffix = if cfg!(target_os = "macos") {
            if cfg!(target_arch = "aarch64") {
                "darwin-arm64"
            } else {
                "darwin-x64"
            }
        } else if cfg!(target_os = "linux") {
            if cfg!(target_arch = "aarch64") {
                "linux-arm64"
            } else {
                "linux-x64"
            }
        } else if cfg!(target_os = "windows") {
            "win32-x64"
        } else {
            ""
        };

        // Helper to search for tsgo in a directory
        let search_in_dir = |dir: &std::path::Path| -> Option<PathBuf> {
            // Try pnpm structure first
            let pnpm_pattern = dir.join("node_modules/.pnpm");
            if pnpm_pattern.exists() {
                if let Ok(entries) = std::fs::read_dir(&pnpm_pattern) {
                    for entry in entries.flatten() {
                        let name = entry.file_name();
                        let name_str = name.to_string_lossy();
                        if name_str.starts_with("@typescript+native-preview-")
                            && name_str.contains(platform_suffix)
                        {
                            let native_path = entry.path().join(&*cstr!(
                                "node_modules/@typescript/native-preview-{}/lib/tsgo",
                                platform_suffix
                            ));
                            if native_path.exists() {
                                return Some(native_path);
                            }
                        }
                    }
                }
            }

            // Try npm/yarn structure
            let native_candidates = [
                dir.join(&*cstr!(
                    "node_modules/@typescript/native-preview-{}/lib/tsgo",
                    platform_suffix
                )),
                dir.join("node_modules/@typescript/native-preview/lib/tsgo"),
            ];

            for candidate in &native_candidates {
                if candidate.exists() {
                    return Some(candidate.clone());
                }
            }

            // Try .bin/tsgo (requires node in PATH)
            let bin_tsgo = dir.join("node_modules/.bin/tsgo");
            if bin_tsgo.exists() {
                return Some(bin_tsgo);
            }

            None
        };

        // 2. Search in base_dir first, then walk up parent directories
        if let Some(path) = search_in_dir(&base_dir) {
            tracing::info!("tsgo_bridge: found tsgo at {:?}", path);
            return Ok(path);
        }

        let mut current = base_dir.as_path();
        while let Some(parent) = current.parent() {
            if let Some(path) = search_in_dir(parent) {
                tracing::info!("tsgo_bridge: found tsgo at {:?}", path);
                return Ok(path);
            }
            current = parent;
        }

        // 3. Try global PATH
        if let Ok(path) = which::which("tsgo") {
            tracing::info!("tsgo_bridge: found tsgo in PATH at {:?}", path);
            return Ok(path);
        }

        Err(TsgoBridgeError::SpawnFailed(
            "tsgo not found. Install with: npm install -D @typescript/native-preview".into(),
        ))
    }

    /// Send LSP initialize request.
    async fn initialize(&self) -> Result<(), TsgoBridgeError> {
        let _timer = self.profiler.timer("lsp_initialize");

        let root_uri = self
            .config
            .working_dir
            .as_ref()
            .map(|p| cstr!("file://{}", p.display()))
            .unwrap_or_else(|| "file:///".into());

        tracing::info!("tsgo_bridge: LSP rootUri = {}", root_uri);

        let params = json!({
            "processId": std::process::id(),
            "capabilities": {
                "textDocument": {
                    "synchronization": {
                        "didSave": true
                    },
                    "publishDiagnostics": {
                        "relatedInformation": true
                    }
                }
            },
            "rootUri": root_uri,
            "initializationOptions": {}
        });

        tracing::info!("tsgo_bridge: sending initialize request...");
        self.send_request("initialize", Some(params)).await?;
        tracing::info!("tsgo_bridge: initialize response received");

        // Send initialized notification
        tracing::info!("tsgo_bridge: sending initialized notification...");
        self.send_notification("initialized", Some(json!({})))
            .await?;
        tracing::info!("tsgo_bridge: initialized notification sent");

        if let Some(timer) = _timer {
            timer.record(&self.profiler);
        }

        Ok(())
    }

    /// Send a JSON-RPC request and wait for response.
    pub(crate) async fn send_request(
        &self,
        method: &str,
        params: Option<Value>,
    ) -> Result<Value, TsgoBridgeError> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);

        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id,
            method: method.into(),
            params,
        };

        let content = serde_json::to_string(&request)
            .map_err(|e| TsgoBridgeError::CommunicationError(e.to_compact_string()))?;

        let message = cstr!("Content-Length: {}\r\n\r\n{}", content.len(), content);

        // Create response channel
        let (tx, rx) = oneshot::channel();
        self.pending.insert(id, tx);

        // Send request
        {
            let mut stdin_guard = self.stdin.lock().await;
            if let Some(ref mut stdin) = *stdin_guard {
                stdin
                    .write_all(message.as_bytes())
                    .await
                    .map_err(|e| TsgoBridgeError::CommunicationError(e.to_compact_string()))?;
                stdin
                    .flush()
                    .await
                    .map_err(|e| TsgoBridgeError::CommunicationError(e.to_compact_string()))?;
            } else {
                return Err(TsgoBridgeError::NotInitialized);
            }
        }

        // Wait for response with timeout
        match tokio::time::timeout(std::time::Duration::from_millis(self.config.timeout_ms), rx)
            .await
        {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err(TsgoBridgeError::CommunicationError(
                "Response channel closed".into(),
            )),
            Err(_) => {
                self.pending.remove(&id);
                Err(TsgoBridgeError::Timeout)
            }
        }
    }

    /// Send a JSON-RPC notification (no response expected).
    pub(crate) async fn send_notification(
        &self,
        method: &str,
        params: Option<Value>,
    ) -> Result<(), TsgoBridgeError> {
        let notification = JsonRpcNotification {
            jsonrpc: "2.0",
            method: method.into(),
            params,
        };

        let content = serde_json::to_string(&notification)
            .map_err(|e| TsgoBridgeError::CommunicationError(e.to_compact_string()))?;

        let message = cstr!("Content-Length: {}\r\n\r\n{}", content.len(), content);

        let mut stdin_guard = self.stdin.lock().await;
        if let Some(ref mut stdin) = *stdin_guard {
            stdin
                .write_all(message.as_bytes())
                .await
                .map_err(|e| TsgoBridgeError::CommunicationError(e.to_compact_string()))?;
            stdin
                .flush()
                .await
                .map_err(|e| TsgoBridgeError::CommunicationError(e.to_compact_string()))?;
            Ok(())
        } else {
            Err(TsgoBridgeError::NotInitialized)
        }
    }

    /// Open a virtual document for type checking.
    pub async fn open_virtual_document(
        &self,
        name: &str,
        content: &str,
    ) -> Result<String, TsgoBridgeError> {
        if !self.initialized.load(Ordering::SeqCst) {
            return Err(TsgoBridgeError::NotInitialized);
        }

        let _timer = self.profiler.timer("open_virtual_document");

        // Use file:// URI scheme for compatibility with tsgo
        // tsgo only publishes diagnostics for file:// URIs
        let uri = if name.starts_with("file://") || name.starts_with('/') {
            if name.starts_with("file://") {
                String::from(name)
            } else {
                cstr!("file://{name}")
            }
        } else {
            cstr!("{VIRTUAL_URI_SCHEME}://{name}")
        };

        // Clear cached diagnostics for this URI
        self.diagnostics_cache.remove(&uri);

        let params = json!({
            "textDocument": {
                "uri": uri,
                "languageId": "typescript",
                "version": 1,
                "text": content
            }
        });

        self.send_notification("textDocument/didOpen", Some(params))
            .await?;

        // Track this document as open with version 1
        self.open_documents.insert(uri.clone(), 1);

        if let Some(timer) = _timer {
            timer.record(&self.profiler);
        }

        Ok(uri)
    }

    /// Open or update a virtual document. Uses didChange if already open, didOpen otherwise.
    pub async fn open_or_update_virtual_document(
        &self,
        name: &str,
        content: &str,
    ) -> Result<String, TsgoBridgeError> {
        if !self.initialized.load(Ordering::SeqCst) {
            return Err(TsgoBridgeError::NotInitialized);
        }

        // Build URI first to check if document is already open
        let uri = if name.starts_with("file://") || name.starts_with('/') {
            if name.starts_with("file://") {
                String::from(name)
            } else {
                cstr!("file://{name}")
            }
        } else {
            cstr!("{VIRTUAL_URI_SCHEME}://{name}")
        };

        // Check if document is already open
        if let Some(mut version_ref) = self.open_documents.get_mut(&uri) {
            // Document is already open, send didChange with incremented version
            let new_version = *version_ref + 1;
            *version_ref = new_version;
            drop(version_ref); // Release the lock before async call

            self.update_virtual_document(&uri, content, new_version)
                .await?;
            Ok(uri)
        } else {
            // Document not open, send didOpen
            self.open_virtual_document(name, content).await
        }
    }

    /// Update a virtual document.
    pub async fn update_virtual_document(
        &self,
        uri: &str,
        content: &str,
        version: i32,
    ) -> Result<(), TsgoBridgeError> {
        if !self.initialized.load(Ordering::SeqCst) {
            return Err(TsgoBridgeError::NotInitialized);
        }

        let _timer = self.profiler.timer("update_virtual_document");

        // Clear cached diagnostics for this URI
        self.diagnostics_cache.remove(uri);

        let params = json!({
            "textDocument": {
                "uri": uri,
                "version": version
            },
            "contentChanges": [{
                "text": content
            }]
        });

        self.send_notification("textDocument/didChange", Some(params))
            .await?;

        if let Some(timer) = _timer {
            timer.record(&self.profiler);
        }

        Ok(())
    }

    /// Close a virtual document.
    pub async fn close_virtual_document(&self, uri: &str) -> Result<(), TsgoBridgeError> {
        if !self.initialized.load(Ordering::SeqCst) {
            return Err(TsgoBridgeError::NotInitialized);
        }

        // Remove from cache and open documents tracking
        self.diagnostics_cache.remove(uri);
        self.open_documents.remove(uri);

        let params = json!({
            "textDocument": {
                "uri": uri
            }
        });

        self.send_notification("textDocument/didClose", Some(params))
            .await
    }

    /// Get diagnostics for a document.
    /// First tries textDocument/diagnostic request, then falls back to cached publishDiagnostics.
    pub async fn get_diagnostics(&self, uri: &str) -> Result<Vec<LspDiagnostic>, TsgoBridgeError> {
        if !self.initialized.load(Ordering::SeqCst) {
            return Err(TsgoBridgeError::NotInitialized);
        }

        // Check cache first (diagnostics arrive via publishDiagnostics notification)
        if let Some(cached) = self.diagnostics_cache.get(uri) {
            self.cache_stats.hit();
            tracing::info!(
                "tsgo_bridge: cache hit for {}, {} diagnostics",
                uri,
                cached.len()
            );
            return Ok(cached.clone());
        }

        self.cache_stats.miss();

        // Try textDocument/diagnostic request first (LSP 3.17+ pull diagnostics)
        // This is how CLI gets diagnostics
        tracing::info!(
            "tsgo_bridge: requesting diagnostics via textDocument/diagnostic for {}",
            uri
        );

        let params = json!({
            "textDocument": {
                "uri": uri
            }
        });

        match self
            .send_request("textDocument/diagnostic", Some(params))
            .await
        {
            Ok(result) => {
                // Parse diagnostic response
                if let Some(items) = result.get("items").and_then(|i| i.as_array()) {
                    let diags: Vec<LspDiagnostic> = items
                        .iter()
                        .filter_map(|d| serde_json::from_value(d.clone()).ok())
                        .collect();
                    tracing::info!(
                        "tsgo_bridge: received {} diagnostics via request for {}",
                        diags.len(),
                        uri
                    );
                    // Cache for later
                    self.diagnostics_cache
                        .insert(String::from(uri), diags.clone());
                    return Ok(diags);
                }
                tracing::info!(
                    "tsgo_bridge: diagnostic request returned no items for {}",
                    uri
                );
            }
            Err(e) => {
                tracing::warn!("tsgo_bridge: textDocument/diagnostic request failed: {}", e);
            }
        }

        // Fallback: wait briefly for publishDiagnostics notification
        tracing::info!("tsgo_bridge: waiting for publishDiagnostics for {}", uri);
        let max_wait = std::time::Duration::from_millis(500);
        let poll_interval = std::time::Duration::from_millis(50);
        let start = std::time::Instant::now();

        while start.elapsed() < max_wait {
            if let Some(cached) = self.diagnostics_cache.get(uri) {
                tracing::info!(
                    "tsgo_bridge: diagnostics arrived via notification for {}, {} items",
                    uri,
                    cached.len()
                );
                return Ok(cached.clone());
            }
            tokio::time::sleep(poll_interval).await;
        }

        tracing::info!(
            "tsgo_bridge: no diagnostics for {} (file may have no errors)",
            uri
        );

        // Return empty if no diagnostics (file might have no errors)
        Ok(vec![])
    }

    /// Type check a virtual TypeScript document.
    pub async fn type_check(
        &self,
        name: &str,
        content: &str,
    ) -> Result<TypeCheckResult, TsgoBridgeError> {
        let _timer = self.profiler.timer("type_check");

        let uri = self.open_virtual_document(name, content).await?;

        // Wait for diagnostics
        let diagnostics = self.get_diagnostics(&uri).await?;

        // Keep document open for incremental updates
        // self.close_virtual_document(&uri).await?;

        if let Some(timer) = _timer {
            timer.record(&self.profiler);
        }

        Ok(TypeCheckResult {
            diagnostics,
            source_map: None,
        })
    }

    /// Shutdown the bridge.
    pub async fn shutdown(&self) -> Result<(), TsgoBridgeError> {
        if !self.initialized.load(Ordering::SeqCst) {
            return Ok(());
        }

        // Send shutdown request
        let _ = self.send_request("shutdown", None).await;

        // Send exit notification
        let _ = self.send_notification("exit", None).await;

        // Kill process if still running
        let mut process_guard = self.process.lock().await;
        if let Some(mut process) = process_guard.take() {
            let _ = process.kill().await;
        }

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
        self.diagnostics_cache.clear();
        self.cache_stats.reset();
    }

    /// Get hover information at a position.
    ///
    /// Sends a textDocument/hover request to tsgo.
    pub async fn hover(
        &self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<LspHover>, TsgoBridgeError> {
        if !self.initialized.load(Ordering::SeqCst) {
            return Err(TsgoBridgeError::NotInitialized);
        }

        let _timer = self.profiler.timer("tsgo_hover");

        let params = json!({
            "textDocument": {
                "uri": uri
            },
            "position": {
                "line": line,
                "character": character
            }
        });

        let result = self
            .send_request("textDocument/hover", Some(params))
            .await?;

        if let Some(timer) = _timer {
            timer.record(&self.profiler);
        }

        // null response means no hover info
        if result.is_null() {
            return Ok(None);
        }

        let hover: LspHover = serde_json::from_value(result).map_err(|e| {
            TsgoBridgeError::CommunicationError(cstr!("Failed to parse hover: {e}"))
        })?;

        Ok(Some(hover))
    }

    /// Get definition location for a symbol at a position.
    ///
    /// Sends a textDocument/definition request to tsgo.
    pub async fn definition(
        &self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Vec<LspLocation>, TsgoBridgeError> {
        if !self.initialized.load(Ordering::SeqCst) {
            return Err(TsgoBridgeError::NotInitialized);
        }

        let _timer = self.profiler.timer("tsgo_definition");

        let params = json!({
            "textDocument": {
                "uri": uri
            },
            "position": {
                "line": line,
                "character": character
            }
        });

        let result = self
            .send_request("textDocument/definition", Some(params))
            .await?;

        if let Some(timer) = _timer {
            timer.record(&self.profiler);
        }

        // null response means no definition
        if result.is_null() {
            return Ok(Vec::new());
        }

        // Try parsing as definition response (can be location, array, or links)
        let response: LspDefinitionResponse = serde_json::from_value(result).map_err(|e| {
            TsgoBridgeError::CommunicationError(cstr!("Failed to parse definition: {e}"))
        })?;

        Ok(response.into_locations())
    }

    /// Get references for a symbol at a position.
    ///
    /// Sends a textDocument/references request to tsgo.
    pub async fn references(
        &self,
        uri: &str,
        line: u32,
        character: u32,
        include_declaration: bool,
    ) -> Result<Vec<LspLocation>, TsgoBridgeError> {
        if !self.initialized.load(Ordering::SeqCst) {
            return Err(TsgoBridgeError::NotInitialized);
        }

        let params = json!({
            "textDocument": {
                "uri": uri
            },
            "position": {
                "line": line,
                "character": character
            },
            "context": {
                "includeDeclaration": include_declaration
            }
        });

        let result = self
            .send_request("textDocument/references", Some(params))
            .await?;

        if result.is_null() {
            return Ok(Vec::new());
        }

        serde_json::from_value(result).map_err(|e| {
            TsgoBridgeError::CommunicationError(cstr!("Failed to parse references: {e}"))
        })
    }

    /// Check whether rename is valid at a position.
    ///
    /// Sends a textDocument/prepareRename request to tsgo.
    pub async fn prepare_rename(
        &self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, TsgoBridgeError> {
        if !self.initialized.load(Ordering::SeqCst) {
            return Err(TsgoBridgeError::NotInitialized);
        }

        let params = json!({
            "textDocument": {
                "uri": uri
            },
            "position": {
                "line": line,
                "character": character
            }
        });

        let result = self
            .send_request("textDocument/prepareRename", Some(params))
            .await?;

        if result.is_null() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }

    /// Rename a symbol at a position.
    ///
    /// Sends a textDocument/rename request to tsgo.
    pub async fn rename(
        &self,
        uri: &str,
        line: u32,
        character: u32,
        new_name: &str,
    ) -> Result<Option<Value>, TsgoBridgeError> {
        if !self.initialized.load(Ordering::SeqCst) {
            return Err(TsgoBridgeError::NotInitialized);
        }

        let params = json!({
            "textDocument": {
                "uri": uri
            },
            "position": {
                "line": line,
                "character": character
            },
            "newName": new_name
        });

        let result = self
            .send_request("textDocument/rename", Some(params))
            .await?;

        if result.is_null() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }

    /// Request import path updates before files are renamed.
    ///
    /// Sends a workspace/willRenameFiles request to tsgo.
    pub async fn will_rename_files(
        &self,
        renames: &[(&str, &str)],
    ) -> Result<Option<Value>, TsgoBridgeError> {
        if !self.initialized.load(Ordering::SeqCst) {
            return Err(TsgoBridgeError::NotInitialized);
        }

        let files: Vec<Value> = renames
            .iter()
            .map(|(old_uri, new_uri)| {
                json!({
                    "oldUri": old_uri,
                    "newUri": new_uri
                })
            })
            .collect();

        let result = self
            .send_request("workspace/willRenameFiles", Some(json!({ "files": files })))
            .await?;

        if result.is_null() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }

    /// Get completion items at a position.
    ///
    /// Sends a textDocument/completion request to tsgo.
    pub async fn completion(
        &self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Vec<LspCompletionItem>, TsgoBridgeError> {
        if !self.initialized.load(Ordering::SeqCst) {
            return Err(TsgoBridgeError::NotInitialized);
        }

        let _timer = self.profiler.timer("tsgo_completion");

        let params = json!({
            "textDocument": {
                "uri": uri
            },
            "position": {
                "line": line,
                "character": character
            },
            "context": {
                "triggerKind": 1  // Invoked
            }
        });

        let result = self
            .send_request("textDocument/completion", Some(params))
            .await?;

        if let Some(timer) = _timer {
            timer.record(&self.profiler);
        }

        // null response means no completions
        if result.is_null() {
            return Ok(Vec::new());
        }

        // Try parsing as completion response (can be array or list)
        let response: LspCompletionResponse = serde_json::from_value(result).map_err(|e| {
            TsgoBridgeError::CommunicationError(cstr!("Failed to parse completion: {e}"))
        })?;

        Ok(response.items())
    }
}

impl Default for TsgoBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TsgoBridge {
    fn drop(&mut self) {
        // Note: Can't do async cleanup in Drop, caller should call shutdown()
    }
}

/// Batch type checker for checking multiple documents efficiently.
#[allow(clippy::disallowed_types)]
pub struct BatchTypeChecker {
    /// Bridge instance
    bridge: Arc<TsgoBridge>,
    /// Batch size
    batch_size: usize,
}

#[allow(clippy::disallowed_types)]
impl BatchTypeChecker {
    /// Create a new batch type checker.
    pub fn new(bridge: Arc<TsgoBridge>) -> Self {
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
    ) -> Vec<Result<TypeCheckResult, TsgoBridgeError>> {
        let _timer = self.bridge.profiler().timer("batch_type_check");

        let mut results = Vec::with_capacity(documents.len());

        for chunk in documents.chunks(self.batch_size) {
            // Open all documents in the chunk
            let mut uris = Vec::with_capacity(chunk.len());
            for (name, content) in chunk {
                match self.bridge.open_virtual_document(name, content).await {
                    Ok(uri) => uris.push(Some(uri)),
                    Err(e) => {
                        results.push(Err(e));
                        uris.push(None);
                    }
                }
            }

            // Wait for diagnostics to be computed (reduced for faster batch processing)
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;

            // Collect diagnostics
            for uri in uris.into_iter().flatten() {
                match self.bridge.get_diagnostics(&uri).await {
                    Ok(diagnostics) => {
                        results.push(Ok(TypeCheckResult {
                            diagnostics,
                            source_map: None,
                        }));
                    }
                    Err(e) => results.push(Err(e)),
                }
            }
        }

        if let Some(timer) = _timer {
            timer.record(self.bridge.profiler());
        }

        results
    }
}
