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
use std::path::{Path, PathBuf};
#[allow(clippy::disallowed_types)]
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use vize_carton::{FxHashMap, FxHashSet, String, cstr};

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
#[derive(Debug, Clone, Default)]
pub struct ServerConfig {
    /// Path to the Corsa executable (uses PATH if not specified)
    pub corsa_path: Option<String>,
    /// Working directory for module resolution
    pub working_dir: Option<String>,
}

/// Corsa server.
#[allow(clippy::disallowed_types)]
pub struct CorsaServer {
    config: ServerConfig,
    running: Arc<AtomicBool>,
    /// Cache of generated Virtual TypeScript (uri -> content)
    cache: FxHashMap<String, String>,
    /// Project-session client for Corsa (lazy initialized).
    corsa_client: Option<crate::corsa_client::CorsaProjectClient>,
    /// Virtual documents currently synced into the persistent Corsa session.
    open_virtual_documents: FxHashSet<String>,
}

impl CorsaServer {
    /// Create a new server with default configuration.
    pub fn new() -> Self {
        Self::with_config(ServerConfig::default())
    }

    /// Create a new server with custom configuration.
    #[allow(clippy::disallowed_types)]
    pub fn with_config(config: ServerConfig) -> Self {
        Self {
            config,
            running: Arc::new(AtomicBool::new(false)),
            cache: FxHashMap::default(),
            corsa_client: None,
            open_virtual_documents: FxHashSet::default(),
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
            #[allow(clippy::disallowed_methods)]
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
            #[allow(clippy::disallowed_methods)]
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

    /// Handle a single JSON-RPC request.
    fn handle_request(&mut self, input: &str) -> JsonRpcResponse {
        let request: JsonRpcRequest = match serde_json::from_str(input) {
            Ok(r) => r,
            Err(e) => {
                return JsonRpcResponse {
                    jsonrpc: "2.0",
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: cstr!("Parse error: {e}"),
                        data: None,
                    }),
                };
            }
        };

        match request.method.as_str() {
            "check" => self.handle_check(request.id, request.params),
            "shutdown" => {
                self.running.store(false, Ordering::SeqCst);
                JsonRpcResponse {
                    jsonrpc: "2.0",
                    id: request.id,
                    result: Some(serde_json::json!({"status": "shutdown"})),
                    error: None,
                }
            }
            _ => JsonRpcResponse {
                jsonrpc: "2.0",
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: cstr!("Method not found: {}", request.method),
                    data: None,
                }),
            },
        }
    }

    /// Handle the "check" method.
    fn handle_check(&mut self, id: Option<u64>, params: serde_json::Value) -> JsonRpcResponse {
        let params: CheckParams = match serde_json::from_value(params) {
            Ok(p) => p,
            Err(e) => {
                return JsonRpcResponse {
                    jsonrpc: "2.0",
                    id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: cstr!("Invalid params: {e}"),
                        data: None,
                    }),
                };
            }
        };

        match self.check_vue_sfc(&params.uri, &params.content) {
            Ok(result) => JsonRpcResponse {
                jsonrpc: "2.0",
                id,
                result: Some(serde_json::to_value(result).unwrap_or(serde_json::Value::Null)),
                error: None,
            },
            Err(e) => JsonRpcResponse {
                jsonrpc: "2.0",
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32000,
                    message: e,
                    data: None,
                }),
            },
        }
    }

    /// Check a Vue SFC and return diagnostics.
    fn check_vue_sfc(&mut self, uri: &str, content: &str) -> Result<CheckResult, String> {
        use vize_atelier_core::parser::parse;
        use vize_atelier_sfc::{
            SfcParseOptions,
            croquis::{SfcCroquisOptions, analyze_sfc_descriptor_with_context},
            parse_sfc,
        };
        use vize_carton::Bump;
        use vize_croquis::virtual_ts::generate_virtual_ts;

        // Parse SFC
        let parse_opts = SfcParseOptions {
            filename: uri.into(),
            ..Default::default()
        };

        let descriptor = parse_sfc(content, parse_opts)
            .map_err(|e| cstr!("Failed to parse SFC: {}", e.message))?;

        // Create allocator
        let allocator = Bump::new();

        let template_offset = descriptor
            .template
            .as_ref()
            .map(|t| t.loc.start as u32)
            .unwrap_or(0);

        let template_ast = if let Some(ref template) = descriptor.template {
            let (root, _) = parse(&allocator, &template.content);
            Some(root)
        } else {
            None
        };

        let analysis = analyze_sfc_descriptor_with_context(
            &descriptor,
            template_ast.as_ref(),
            SfcCroquisOptions::full().without_script_merge(),
        );

        // Generate Virtual TypeScript
        let output = generate_virtual_ts(
            analysis.script_content_ref(),
            template_ast.as_ref(),
            &analysis.croquis.bindings,
            None,
            Some(Path::new(uri)),
            template_offset,
        );

        // Issue #752: rewrite `.vue` import specifiers to `.vue.ts` so the
        // socket-mode Corsa session resolves siblings via the same virtual
        // mirrors used by the batch path, then overlay each relative
        // sibling's virtual TS into the session. The cached `virtual_ts`
        // intentionally reflects what we ship to Corsa (rewritten form), so
        // consumers that introspect the cache see the same coordinates.
        let pre_rewrite_ts = output.content;
        let rewriter = crate::batch::ImportRewriter::new();
        let virtual_ts: String = rewriter
            .rewrite(pre_rewrite_ts.as_str(), oxc_span::SourceType::ts())
            .code;

        self.cache.insert(uri.into(), virtual_ts.clone());

        // Overlay sibling .vue.ts mirrors discovered from the host's imports.
        let relative_specifiers = rewriter
            .collect_relative_vue_specifiers(pre_rewrite_ts.as_str(), oxc_span::SourceType::ts());
        if !relative_specifiers.is_empty() {
            self.overlay_sibling_vue_mirrors(uri, &relative_specifiers);
        }

        // Run Corsa on the virtual TypeScript through the project-session API.
        let mut diagnostics = self.run_corsa(uri, &virtual_ts)?;

        // Merge in Vue-specific compile errors (e.g. props destructure default type
        // mismatch) so the socket-mode check matches the direct `vize check` runner.
        if let Some(sfc_diagnostic) = collect_sfc_compile_diagnostic(uri, content, &descriptor) {
            diagnostics.push(sfc_diagnostic);
        }

        let error_count = diagnostics.iter().filter(|d| d.severity == "error").count();

        Ok(CheckResult {
            diagnostics,
            virtual_ts,
            error_count,
        })
    }

    /// Run Corsa on TypeScript content and parse diagnostics via project sessions.
    fn run_corsa(&mut self, uri: &str, content: &str) -> Result<Vec<Diagnostic>, String> {
        if self.corsa_client.is_none() {
            let client = crate::corsa_client::CorsaProjectClient::new(
                self.config.corsa_path.as_deref(),
                self.config.working_dir.as_deref(),
            )?;
            self.corsa_client = Some(client);
        }

        let virtual_uri = self.virtual_uri_for(uri);
        let client = self
            .corsa_client
            .as_mut()
            .expect("corsa_client must be initialized above");

        if self.open_virtual_documents.contains(virtual_uri.as_str()) {
            client.did_change(&virtual_uri, content)?;
        } else {
            client.did_open(&virtual_uri, content)?;
            self.open_virtual_documents.insert(virtual_uri.clone());
        }
        let corsa_diagnostics = client.request_diagnostics(&virtual_uri)?;

        // Convert Corsa's editor-style diagnostics to the server payload.
        let diagnostics = corsa_diagnostics
            .into_iter()
            .map(|d| {
                let severity: String = match d.severity {
                    Some(1) => "error".into(),
                    Some(2) => "warning".into(),
                    Some(3) => "info".into(),
                    Some(4) => "hint".into(),
                    _ => "error".into(),
                };
                let code = d.code.map(|c| match c {
                    serde_json::Value::Number(n) => cstr!("TS{n}"),
                    serde_json::Value::String(s) => s.into(),
                    _ => cstr!("{c:?}"),
                });
                Diagnostic {
                    message: d.message,
                    severity,
                    line: d.range.start.line + 1,
                    column: d.range.start.character + 1,
                    code,
                }
            })
            .collect();

        Ok(diagnostics)
    }

    /// Overlay sibling `.vue.ts` mirrors for every relative `.vue` import,
    /// recursively, so socket-mode Corsa can resolve `import App from
    /// './app.vue'` (issue #752). Errors are logged and skipped so a missing
    /// sibling still surfaces as TS2307 from the host check.
    fn overlay_sibling_vue_mirrors(&mut self, host_uri: &str, initial_specifiers: &[String]) {
        use vize_atelier_core::parser::parse;
        use vize_atelier_sfc::{
            SfcParseOptions,
            croquis::{SfcCroquisOptions, analyze_sfc_descriptor_with_context},
            parse_sfc,
        };
        use vize_carton::Bump;
        use vize_croquis::virtual_ts::generate_virtual_ts;

        let Some(host_path) = uri_to_path(host_uri, &self.working_dir()) else {
            tracing::debug!("overlay_sibling_vue_mirrors: cannot resolve host path: {host_uri}");
            return;
        };
        let host_dir = match host_path.parent() {
            Some(dir) => dir.to_path_buf(),
            None => return,
        };

        let mut visited: FxHashSet<PathBuf> = FxHashSet::default();
        visited.insert(host_path.clone());

        let mut queue: Vec<(PathBuf, Vec<String>)> = vec![(
            host_dir,
            initial_specifiers
                .iter()
                .map(|s| s.as_str().into())
                .collect(),
        )];
        let rewriter = crate::batch::ImportRewriter::new();

        while let Some((dir, specifiers)) = queue.pop() {
            for specifier in specifiers {
                let resolved = dir.join(&specifier);
                let canonical = std::fs::canonicalize(&resolved).unwrap_or(resolved);
                if !visited.insert(canonical.clone()) {
                    continue;
                }

                let sibling_content = match std::fs::read_to_string(&canonical) {
                    Ok(text) => text,
                    Err(err) => {
                        tracing::debug!(
                            "socket overlay sibling skipped — read failed for {}: {err}",
                            canonical.display(),
                        );
                        continue;
                    }
                };

                let sibling_uri = crate::file_uri::path_to_file_uri(&canonical);
                let sibling_virtual_uri = self.virtual_uri_for(&sibling_uri);

                let parse_opts = SfcParseOptions {
                    filename: sibling_uri.as_str().into(),
                    ..Default::default()
                };
                let Ok(descriptor) = parse_sfc(&sibling_content, parse_opts) else {
                    continue;
                };

                let allocator = Bump::new();
                let template_offset = descriptor
                    .template
                    .as_ref()
                    .map(|t| t.loc.start as u32)
                    .unwrap_or(0);
                let template_ast = descriptor.template.as_ref().map(|template| {
                    let (root, _) = parse(&allocator, &template.content);
                    root
                });
                let analysis = analyze_sfc_descriptor_with_context(
                    &descriptor,
                    template_ast.as_ref(),
                    SfcCroquisOptions::full().without_script_merge(),
                );
                let sibling_output = generate_virtual_ts(
                    analysis.script_content_ref(),
                    template_ast.as_ref(),
                    &analysis.croquis.bindings,
                    None,
                    Some(canonical.as_path()),
                    template_offset,
                );

                let sibling_rewrite =
                    rewriter.rewrite(sibling_output.content.as_str(), oxc_span::SourceType::ts());
                let sibling_virtual_ts: String = sibling_rewrite.code;

                let client = match self.corsa_client.as_mut() {
                    Some(client) => client,
                    None => return,
                };

                let result = if self
                    .open_virtual_documents
                    .contains(sibling_virtual_uri.as_str())
                {
                    client.did_change(&sibling_virtual_uri, &sibling_virtual_ts)
                } else {
                    let r = client.did_open(&sibling_virtual_uri, &sibling_virtual_ts);
                    if r.is_ok() {
                        self.open_virtual_documents
                            .insert(sibling_virtual_uri.clone());
                    }
                    r
                };
                if let Err(err) = result {
                    tracing::debug!(
                        "socket overlay sibling failed for {}: {err}",
                        canonical.display(),
                    );
                    continue;
                }

                let next_specifiers = rewriter.collect_relative_vue_specifiers(
                    sibling_output.content.as_str(),
                    oxc_span::SourceType::ts(),
                );
                if !next_specifiers.is_empty() {
                    let next_dir = canonical
                        .parent()
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|| canonical.clone());
                    queue.push((next_dir, next_specifiers));
                }
            }
        }
    }

    fn virtual_uri_for(&self, uri: &str) -> String {
        if uri.starts_with("file://") || uri.contains("://") {
            return cstr!("{uri}.ts");
        }

        let virtual_path = cstr!("{uri}.ts");
        let path = Path::new(virtual_path.as_str());
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.working_dir().join(path)
        };
        crate::file_uri::path_to_file_uri(&path)
    }

    fn working_dir(&self) -> PathBuf {
        self.config
            .working_dir
            .as_deref()
            .map(PathBuf::from)
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."))
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

/// Surface Vue-specific script-setup semantic errors (e.g.
/// `DEFINE_PROPS_DESTRUCTURE_DEFAULT_TYPE`). Uses the lightweight validator
/// entry point so the socket-mode check stays as fast as the Virtual TS path.
/// Resolve a URI (file:// or plain path) to an absolute filesystem path.
/// Returns None when the URI is a non-`file` scheme, the path cannot be
/// extracted, or percent-decoding yields invalid UTF-8. Used by socket-mode
/// sibling overlay to read siblings from disk. `file://` URIs go through the
/// shared converter in `crate::file_uri` so percent-escapes are decoded as
/// UTF-8 byte sequences (not per-byte chars, which garbles non-ASCII paths).
fn uri_to_path(uri: &str, working_dir: &Path) -> Option<PathBuf> {
    if uri.starts_with("file://") {
        return crate::file_uri::file_uri_to_path(uri);
    }
    if uri.contains("://") {
        return None;
    }
    let path = Path::new(uri);
    if path.is_absolute() {
        Some(path.to_path_buf())
    } else {
        Some(working_dir.join(path))
    }
}

fn collect_sfc_compile_diagnostic(
    _uri: &str,
    source: &str,
    descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
) -> Option<Diagnostic> {
    let script_setup = descriptor.script_setup.as_ref()?;
    if !script_setup_has_validator_candidates(&script_setup.content) {
        return None;
    }

    let Err(error) = vize_atelier_sfc::validate_script_setup_semantics_located(
        &script_setup.content,
        script_setup.loc.start,
        source,
    ) else {
        return None;
    };

    let (line, column) = if let Some(loc) = error.loc.as_ref() {
        (
            (loc.start_line as u32).saturating_sub(1),
            (loc.start_column as u32).saturating_sub(1),
        )
    } else {
        let offset = sfc_block_fallback_offset(descriptor);
        offset_to_line_column(source, offset)
    };

    let message = match error.code.as_deref() {
        Some(code) => cstr!("[{}] {}", code, error.message),
        None => error.message.clone(),
    };

    Some(Diagnostic {
        message,
        severity: "error".into(),
        line,
        column,
        code: error.code.clone(),
    })
}

/// See the canon batch path for rationale — keep this in sync with
/// `crates/vize_canon/src/batch/virtual_project.rs`.
fn script_setup_has_validator_candidates(content: &str) -> bool {
    content.contains("defineProps<") && content.contains("= defineProps")
}

fn sfc_block_fallback_offset(descriptor: &vize_atelier_sfc::SfcDescriptor<'_>) -> usize {
    if let Some(setup) = descriptor.script_setup.as_ref() {
        return setup.loc.start;
    }
    if let Some(script) = descriptor.script.as_ref() {
        return script.loc.start;
    }
    if let Some(template) = descriptor.template.as_ref() {
        return template.loc.start;
    }
    0
}

fn offset_to_line_column(source: &str, offset: usize) -> (u32, u32) {
    let target = offset.min(source.len());
    let mut line: u32 = 0;
    let mut line_start: usize = 0;
    for (index, ch) in source.char_indices() {
        if index >= target {
            break;
        }
        if ch == '\n' {
            line += 1;
            line_start = index + 1;
        }
    }
    // LSP `Position.character` is in UTF-16 code units. Astral characters
    // (`len_utf16() == 2`) count as two so the column lines up with
    // `vue-tsc` / `@vue/language-tools`. (#965)
    let column: u32 = source[line_start..target]
        .chars()
        .map(|ch| ch.len_utf16() as u32)
        .sum();
    (line, column)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{CorsaServer, JsonRpcRequest, ServerConfig, uri_to_path};
    use vize_carton::String;

    #[test]
    fn test_json_rpc_request_parse() {
        let json = r#"{"jsonrpc":"2.0","id":1,"method":"check","params":{"uri":"test.vue","content":"<template></template>"}}"#;
        let request: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.method, "check");
        assert_eq!(request.id, Some(1));
    }

    #[test]
    fn relative_virtual_uris_are_rooted_in_working_dir() {
        let server = CorsaServer::with_config(ServerConfig {
            corsa_path: None,
            working_dir: Some("/workspace/project".into()),
        });

        assert_eq!(
            server.virtual_uri_for("src/App.vue"),
            String::from("file:///workspace/project/src/App.vue.ts")
        );
    }

    #[test]
    fn absolute_virtual_uris_are_file_uris() {
        let server = CorsaServer::new();

        assert_eq!(
            server.virtual_uri_for("/workspace/pages/[name] #1.vue"),
            String::from("file:///workspace/pages/%5Bname%5D%20%231.vue.ts")
        );
    }

    #[test]
    fn existing_file_uris_keep_their_scheme() {
        let server = CorsaServer::new();

        assert_eq!(
            server.virtual_uri_for("file:///workspace/src/App.vue"),
            String::from("file:///workspace/src/App.vue.ts")
        );
    }

    #[test]
    fn uri_to_path_decodes_multi_byte_utf8_escapes() {
        // %E3%83%86%E3%82%B9%E3%83%88 is "テスト"; per-byte char pushes
        // would turn it into mojibake instead of the original segment.
        assert_eq!(
            uri_to_path(
                "file:///Users/foo/%E3%83%86%E3%82%B9%E3%83%88/App.vue",
                Path::new("/wd")
            ),
            Some(PathBuf::from("/Users/foo/テスト/App.vue"))
        );
    }

    #[test]
    fn uri_to_path_decodes_spaces() {
        assert_eq!(
            uri_to_path("file:///work/my%20app/App.vue", Path::new("/wd")),
            Some(PathBuf::from("/work/my app/App.vue"))
        );
    }

    #[test]
    fn uri_to_path_rejects_invalid_utf8_escapes() {
        assert_eq!(
            uri_to_path("file:///work/%FF%FE/App.vue", Path::new("/wd")),
            None
        );
    }

    #[test]
    fn uri_to_path_resolves_relative_paths_against_working_dir() {
        assert_eq!(
            uri_to_path("src/App.vue", Path::new("/workspace/project")),
            Some(PathBuf::from("/workspace/project/src/App.vue"))
        );
    }

    #[test]
    fn uri_to_path_rejects_non_file_schemes() {
        assert_eq!(uri_to_path("untitled://buffer-1", Path::new("/wd")), None);
    }
}
