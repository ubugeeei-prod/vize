//! Corsa server for Vue SFC type checking.
//!
//! This server provides a JSON-RPC interface over Unix socket or stdin/stdout
//! for type checking Vue Single File Components using Corsa as the backend.
//!
//! ## Protocol
//!
//! Request format:
//! ```json
//! {"jsonrpc": "2.0", "id": 1, "method": "check", "params": {"uri": "file.vue", "content": "..."}}
//! ```
//!
//! Response format:
//! ```json
//! {"jsonrpc": "2.0", "id": 1, "result": {"diagnostics": [...], "virtualTs": "..."}}
//! ```
//!
//! ## Unix Socket Mode
//!
//! Start server: `vize check-server --socket ./node_modules/.vize/vize.sock`
//! Connect: `echo '{"jsonrpc":"2.0","id":1,"method":"check",...}' | nc -U ./node_modules/.vize/vize.sock`

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
#[expect(clippy::disallowed_types, reason = "shared across threads")]
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use vize_carton::{FxHashMap, String};

use crate::corsa_session_cache::{DEFAULT_SESSION_IDLE, SessionMap};

mod diagnostics;
mod request;
mod sessions;
mod sfc_semantics;

/// JSON-RPC Request
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<u64>,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

/// JSON-RPC Response
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC Error
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Check request parameters
#[derive(Debug, Deserialize)]
pub struct CheckParams {
    pub uri: String,
    pub content: String,
    /// Canonical checking-flag spelling. Omitted means no overrides, which is
    /// what `vize check` sends. A different spelling is a different session.
    #[serde(default)]
    pub flags: String,
}

/// Check response
#[derive(Debug, Serialize)]
pub struct CheckResult {
    pub diagnostics: Vec<Diagnostic>,
    #[serde(rename = "virtualTs")]
    pub virtual_ts: String,
    #[serde(rename = "errorCount")]
    pub error_count: usize,
}

/// Diagnostic from type checking
#[derive(Debug, Serialize, Clone)]
pub struct Diagnostic {
    pub message: String,
    pub severity: String,
    pub line: u32,
    pub column: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Path to the Corsa executable (uses PATH if not specified)
    pub corsa_path: Option<String>,
    /// Working directory for module resolution
    pub working_dir: Option<String>,
    /// Drop a project session that has not been used for this long.
    /// Zero keeps every session until the process exits.
    pub idle_timeout: Duration,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            corsa_path: None,
            working_dir: None,
            idle_timeout: DEFAULT_SESSION_IDLE,
        }
    }
}

/// Corsa server.
#[expect(clippy::disallowed_types, reason = "shared across threads")]
pub struct CorsaServer {
    config: ServerConfig,
    running: Arc<AtomicBool>,
    /// Cache of generated Virtual TypeScript (uri -> content)
    cache: FxHashMap<String, String>,
    /// Project sessions keyed by the full P5-1b ambient manifest.
    sessions: SessionMap<crate::corsa_client::CorsaProjectClient>,
    /// TypeScript project initializations observed by this server.
    observed_project_inits: u64,
    /// Shared importer-scoped package topology for the full server lifetime.
    package_route_resolver: crate::PackageRouteResolver,
    /// Private editor mirror/cache for this check-server process.
    editor_session: crate::corsa_bridge::EditorMirrorSession,
}

impl CorsaServer {
    /// Create a new server with default configuration.
    pub fn new() -> Self {
        Self::with_config(ServerConfig::default())
    }

    /// Create a new server with custom configuration.
    #[expect(clippy::disallowed_types, reason = "shared across threads")]
    pub fn with_config(config: ServerConfig) -> Self {
        let idle_timeout = config.idle_timeout;
        Self {
            config,
            running: Arc::new(AtomicBool::new(false)),
            cache: FxHashMap::default(),
            sessions: SessionMap::new(idle_timeout),
            observed_project_inits: 0,
            package_route_resolver: crate::PackageRouteResolver::default(),
            editor_session: crate::corsa_bridge::EditorMirrorSession::new(),
        }
    }

    /// Run the server, reading from stdin and writing to stdout.
    pub fn run(&mut self) -> std::io::Result<()> {
        self.running.store(true, Ordering::SeqCst);

        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();
        let reader = BufReader::new(stdin.lock());

        for line in reader.lines() {
            if !self.running.load(Ordering::SeqCst) {
                break;
            }

            let line = match line {
                Ok(l) => l,
                Err(_) => continue,
            };

            if line.trim().is_empty() {
                continue;
            }

            let response = self.handle_request(&line);
            let response_json = serde_json::to_string(&response).unwrap_or_else(|_| {
                r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal error"}}"#.into()
            });

            writeln!(stdout, "{}", response_json)?;
            stdout.flush()?;
        }

        Ok(())
    }

    /// Run the server on a Unix socket.
    pub fn run_socket(&mut self, socket_path: &str) -> std::io::Result<()> {
        // Remove existing socket file
        let _ = std::fs::remove_file(socket_path);

        let listener = UnixListener::bind(socket_path)?;
        self.running.store(true, Ordering::SeqCst);

        eprintln!("Listening on Unix socket: {}", socket_path);

        // Handle connections
        for stream in listener.incoming() {
            if !self.running.load(Ordering::SeqCst) {
                break;
            }

            match stream {
                Ok(stream) => {
                    self.handle_connection(stream);
                }
                Err(e) => {
                    eprintln!("Connection error: {}", e);
                }
            }
        }

        // Clean up socket file
        let _ = std::fs::remove_file(socket_path);

        Ok(())
    }

    /// Handle a single Unix socket connection.
    fn handle_connection(&mut self, stream: UnixStream) {
        let reader = BufReader::new(&stream);
        let mut writer = &stream;

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };

            if line.trim().is_empty() {
                continue;
            }

            let response = self.handle_request(&line);
            let response_json = serde_json::to_string(&response).unwrap_or_else(|_| {
                r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal error"}}"#.into()
            });

            if writeln!(writer, "{}", response_json).is_err() {
                break;
            }
            if writer.flush().is_err() {
                break;
            }

            // Check if shutdown was requested
            if !self.running.load(Ordering::SeqCst) {
                break;
            }
        }
    }

    /// Stop the server.
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

impl Default for CorsaServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod reuse_tests;

#[cfg(test)]
mod tests {
    use super::JsonRpcRequest;

    #[test]
    fn test_json_rpc_request_parse() {
        let json = r#"{"jsonrpc":"2.0","id":1,"method":"check","params":{"uri":"test.vue","content":"<template></template>"}}"#;
        let request: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.method, "check");
        assert_eq!(request.id, Some(1));
    }
}
