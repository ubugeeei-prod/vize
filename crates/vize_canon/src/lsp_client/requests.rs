//! Request/response and document management for the LSP client.
//!
//! Contains methods for sending JSON-RPC requests and notifications,
//! low-level message I/O, and document lifecycle operations
//! (open, close, diagnostics).

use serde_json::Value;
use std::{
    io::{BufRead, ErrorKind, Read, Write},
    sync::atomic::Ordering,
    thread,
    time::Duration,
};

use super::{LspDiagnostic, TsgoLspClient};
use crate::LspHover;
use vize_carton::cstr;
use vize_carton::FxHashMap;
use vize_carton::String;

impl TsgoLspClient {
    /// Open a virtual document (waits for diagnostics - slower but convenient for single files)
    pub fn did_open(&mut self, uri: &str, content: &str) -> Result<(), String> {
        self.did_open_fast(uri, content)?;
        // Read any diagnostics that might be published
        self.read_notifications()?;
        Ok(())
    }

    /// Open a virtual document without waiting for diagnostics (faster for batch operations)
    /// Call wait_for_diagnostics() after opening all files to collect diagnostics
    pub fn did_open_fast(&mut self, uri: &str, content: &str) -> Result<(), String> {
        let params = serde_json::json!({
            "textDocument": {
                "uri": uri,
                "languageId": "typescript",
                "version": 1,
                "text": content
            }
        });

        self.send_notification("textDocument/didOpen", params)?;

        // Drain any pending messages to prevent pipe buffer from filling up
        self.drain_pending_messages();

        Ok(())
    }

    /// Close a virtual document
    pub fn did_close(&mut self, uri: &str) -> Result<(), String> {
        let params = serde_json::json!({
            "textDocument": {
                "uri": uri
            }
        });

        self.send_notification("textDocument/didClose", params)?;

        // Remove cached diagnostics
        self.diagnostics.remove(uri);

        Ok(())
    }

    /// Get diagnostics for a URI
    pub fn get_diagnostics(&self, uri: &str) -> Vec<LspDiagnostic> {
        self.diagnostics.get(uri).cloned().unwrap_or_default()
    }

    /// Request diagnostics using textDocument/diagnostic (LSP 3.17+)
    pub fn request_diagnostics(&mut self, uri: &str) -> Result<Vec<LspDiagnostic>, String> {
        let params = serde_json::json!({
            "textDocument": {
                "uri": uri
            }
        });

        match self.send_request("textDocument/diagnostic", params) {
            Ok(result) => {
                // Parse the diagnostic response
                if let Some(items) = result.get("items").and_then(|i| i.as_array()) {
                    let diags: Vec<LspDiagnostic> = items
                        .iter()
                        .filter_map(|d| serde_json::from_value(d.clone()).ok())
                        .collect();
                    return Ok(diags);
                }
                Ok(vec![])
            }
            Err(_) => {
                // Fallback to cached diagnostics from publishDiagnostics
                Ok(self.diagnostics.get(uri).cloned().unwrap_or_default())
            }
        }
    }

    /// Get hover information at a position.
    pub fn hover(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<LspHover>, String> {
        let params = serde_json::json!({
            "textDocument": {
                "uri": uri
            },
            "position": {
                "line": line,
                "character": character
            }
        });

        let result = self.send_request("textDocument/hover", params)?;
        if result.is_null() {
            return Ok(None);
        }

        serde_json::from_value(result)
            .map(Some)
            .map_err(|err| cstr!("Failed to parse hover response: {err}"))
    }

    /// Request diagnostics for multiple URIs in batch (pipelined)
    /// Sends all requests first, then collects all responses
    pub fn request_diagnostics_batch(
        &mut self,
        uris: &[String],
    ) -> Vec<(String, Vec<LspDiagnostic>)> {
        // Phase 1: Send all requests
        let mut request_ids: FxHashMap<i64, String> = FxHashMap::default();
        for uri in uris {
            let id = self.request_id.fetch_add(1, Ordering::SeqCst);
            let request = serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "method": "textDocument/diagnostic",
                "params": {
                    "textDocument": {
                        "uri": uri
                    }
                }
            });

            if self.send_message(&request).is_ok() {
                request_ids.insert(id, uri.clone());
            }
        }

        // Phase 2: Collect all responses
        let mut results: Vec<(String, Vec<LspDiagnostic>)> = Vec::new();
        let max_wait = Duration::from_secs(30);
        let start = std::time::Instant::now();

        while !request_ids.is_empty() && start.elapsed() < max_wait {
            match self.try_read_message_nonblocking() {
                Some(Ok(msg)) => {
                    // Check if this is a response
                    if let Some(msg_id) = msg.get("id").and_then(|i| i.as_i64()) {
                        if let Some(uri) = request_ids.remove(&msg_id) {
                            // Parse diagnostics from result
                            let diags = msg
                                .get("result")
                                .and_then(|r| r.get("items"))
                                .and_then(|i| i.as_array())
                                .map(|items| {
                                    items
                                        .iter()
                                        .filter_map(|d| serde_json::from_value(d.clone()).ok())
                                        .collect()
                                })
                                .unwrap_or_default();
                            results.push((uri, diags));
                        }
                    } else {
                        // Handle notification
                        self.handle_notification(&msg);
                    }
                }
                Some(Err(_)) => break,
                None => {
                    thread::sleep(Duration::from_millis(1));
                }
            }
        }

        results
    }

    /// Send a JSON-RPC request and wait for response
    pub(crate) fn send_request(&mut self, method: &str, params: Value) -> Result<Value, String> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });

        self.send_message(&request)?;

        // Read response (and any notifications)
        loop {
            let msg = self.read_message()?;

            // Check if this is our response
            if let Some(msg_id) = msg.get("id") {
                if msg_id.as_i64() == Some(id) {
                    if let Some(error) = msg.get("error") {
                        return Err(cstr!("LSP error: {error:?}"));
                    }
                    return Ok(msg.get("result").cloned().unwrap_or(Value::Null));
                }
            }

            // Handle notification
            self.handle_notification(&msg);
        }
    }

    /// Send a JSON-RPC notification (no response expected)
    pub(crate) fn send_notification(&mut self, method: &str, params: Value) -> Result<(), String> {
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });

        self.send_message(&notification)
    }

    /// Send a message with Content-Length header
    pub(crate) fn send_message(&mut self, msg: &Value) -> Result<(), String> {
        #[allow(clippy::disallowed_methods)]
        let content = serde_json::to_string(msg).map_err(|e| cstr!("JSON error: {e}"))?;
        let header = cstr!("Content-Length: {}\r\n\r\n", content.len());

        self.stdin
            .write_all(header.as_bytes())
            .map_err(|e| cstr!("Write error: {e}"))?;
        self.stdin
            .write_all(content.as_bytes())
            .map_err(|e| cstr!("Write error: {e}"))?;
        self.stdin.flush().map_err(|e| cstr!("Flush error: {e}"))?;

        Ok(())
    }

    /// Read a single LSP message
    #[allow(clippy::disallowed_types)]
    pub(crate) fn read_message(&mut self) -> Result<Value, String> {
        // Read headers (with retry on WouldBlock for non-blocking mode)
        let mut content_length: usize = 0;
        let mut headers_read: Vec<std::string::String> = Vec::new();

        loop {
            let mut line = std::string::String::new();
            let bytes_read = loop {
                match self.stdout.read_line(&mut line) {
                    Ok(n) => break n,
                    Err(e) if e.kind() == ErrorKind::WouldBlock => {
                        // Non-blocking mode: wait a bit and retry
                        thread::sleep(Duration::from_millis(1));
                        continue;
                    }
                    Err(e) => return Err(cstr!("Read error: {e}")),
                }
            };

            if bytes_read == 0 {
                // EOF - process may have exited
                return Err(cstr!(
                    "EOF while reading headers. Headers read so far: {headers_read:?}"
                ));
            }

            headers_read.push(line.clone());
            let line = line.trim();

            if line.is_empty() {
                break;
            }

            if let Some(len_str) = line.strip_prefix("Content-Length: ") {
                content_length = len_str
                    .parse()
                    .map_err(|e| cstr!("Invalid Content-Length: {e}"))?;
            }
        }

        if content_length == 0 {
            return Err(cstr!("No Content-Length header. Headers: {headers_read:?}"));
        }

        // Read content (with retry on WouldBlock)
        let mut content = vec![0u8; content_length];
        let mut bytes_read = 0;
        while bytes_read < content_length {
            match self.stdout.read(&mut content[bytes_read..]) {
                Ok(0) => return Err("EOF while reading content".into()),
                Ok(n) => bytes_read += n,
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(1));
                    continue;
                }
                Err(e) => return Err(cstr!("Read error: {e}")),
            }
        }

        let msg: Value =
            serde_json::from_slice(&content).map_err(|e| cstr!("JSON parse error: {e}"))?;

        Ok(msg)
    }
}
