//! Editor-feature fallback over Corsa's `--lsp --stdio` transport.
//!
//! The project-session API exposes diagnostics but rejects `hover` (and the
//! other editor requests) as `CorsaError::Unsupported` on every pinned runtime
//! — see ubugeeei-prod/corsa-bind#409. The very same runtime does advertise
//! `hoverProvider` over its LSP transport, so editor features route through a
//! second, lazily spawned session that mirrors the virtual documents in.
//!
//! The session is lazy on purpose: typecheck-only runs (the common case) never
//! pay for the extra process, and a session that has answered one hover is
//! reused for every later request.

use super::{CorsaProjectClient, diagnostics_lsp::initialize_lsp_client};
use corsa::{
    jsonrpc::InboundEvent,
    lsp::{LspClient, LspOverlay, LspSpawnConfig, VirtualDocument},
    runtime::block_on,
};
use lsp_types::Uri;
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use vize_carton::{FxHashMap, String, cstr};

struct RawHoverRequest;

impl lsp_types::request::Request for RawHoverRequest {
    type Params = Value;
    type Result = Option<Value>;
    const METHOD: &'static str = "textDocument/hover";
}

struct RawCompletionRequest;

impl lsp_types::request::Request for RawCompletionRequest {
    type Params = Value;
    type Result = Option<Value>;
    const METHOD: &'static str = "textDocument/completion";
}

/// A reusable `--lsp --stdio` session used only for editor requests.
pub(super) struct EditorLspSession {
    client: LspClient,
    overlay: LspOverlay,
    stop: Arc<AtomicBool>,
    responder: Option<std::thread::JoinHandle<()>>,
    /// Last text mirrored into the server, keyed by session document URI.
    documents: FxHashMap<String, String>,
}

impl EditorLspSession {
    fn spawn(executable: &str, cwd: &Path, project_root: &Path) -> Result<Self, String> {
        let client = block_on(LspClient::spawn(
            LspSpawnConfig::new(executable).with_cwd(cwd.to_path_buf()),
        ))
        .map_err(|error| cstr!("Failed to start Corsa editor LSP session: {error}"))?;
        let stop = Arc::new(AtomicBool::new(false));
        let responder = spawn_responder(client.clone(), stop.clone());

        if let Err(error) = initialize_lsp_client(&client, project_root) {
            stop.store(true, Ordering::Relaxed);
            let _ = block_on(client.close());
            let _ = responder.join();
            return Err(error);
        }

        Ok(Self {
            overlay: client.overlay(),
            client,
            stop,
            responder: Some(responder),
            documents: Default::default(),
        })
    }

    /// Mirror `text` into the server, opening the document on first sight.
    fn mirror(&mut self, document_uri: &str, text: &str) -> Result<Uri, String> {
        let uri = Uri::from_str(document_uri)
            .map_err(|error| cstr!("Invalid LSP document URI {document_uri}: {error}"))?;
        match self.documents.get(document_uri) {
            Some(previous) if previous.as_str() == text => return Ok(uri),
            Some(_) => {
                self.overlay.replace(&uri, text).map_err(|error| {
                    cstr!("Failed to update editor LSP overlay for {document_uri}: {error}")
                })?;
            }
            None => {
                let language_id =
                    if document_uri.ends_with(".tsx") || document_uri.ends_with(".jsx") {
                        "typescriptreact"
                    } else {
                        "typescript"
                    };
                self.overlay
                    .open(VirtualDocument::new(uri.clone(), language_id, text))
                    .map_err(|error| {
                        cstr!("Failed to open editor LSP overlay for {document_uri}: {error}")
                    })?;
            }
        }
        self.documents.insert(document_uri.into(), text.into());
        Ok(uri)
    }

    fn hover(
        &mut self,
        document_uri: &str,
        text: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        let uri = self.mirror(document_uri, text)?;
        block_on(self.client.request::<RawHoverRequest>(serde_json::json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": character },
        })))
        .map_err(|error| cstr!("Failed to request editor LSP hover: {error}"))
    }

    fn completion(
        &mut self,
        document_uri: &str,
        text: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        let uri = self.mirror(document_uri, text)?;
        block_on(
            self.client
                .request::<RawCompletionRequest>(serde_json::json!({
                    "textDocument": { "uri": uri },
                    "position": { "line": line, "character": character },
                    "context": { "triggerKind": 1 },
                })),
        )
        .map_err(|error| cstr!("Failed to request editor LSP completion: {error}"))
    }
}

impl Drop for EditorLspSession {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = block_on(self.client.close());
        if let Some(responder) = self.responder.take() {
            let _ = responder.join();
        }
    }
}

/// Answers the server-initiated requests tsgo makes during startup; without a
/// reply the server blocks before it ever serves an editor request.
fn spawn_responder(client: LspClient, stop: Arc<AtomicBool>) -> std::thread::JoinHandle<()> {
    let events = client.subscribe();
    std::thread::spawn(move || {
        while !stop.load(Ordering::Relaxed) {
            if let Ok(InboundEvent::Request { id, method, params }) =
                events.recv_timeout(Duration::from_millis(50))
            {
                let response = match method.as_ref() {
                    "workspace/configuration" => configuration_response(&params),
                    _ => Value::Null,
                };
                let _ = client.respond(id, response);
            }
        }
    })
}

/// `workspace/configuration` results are positional: the array must hold one
/// entry per requested item, in request order, with `null` for settings the
/// client cannot supply. We supply none, so every slot is `null`. A bare `[]`
/// would misalign servers that read `result[i]` for `items[i]`.
fn configuration_response(params: &Value) -> Value {
    let requested = params
        .get("items")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    Value::Array(vec![Value::Null; requested])
}

impl CorsaProjectClient {
    /// Answer a hover through the editor LSP transport, spawning the session on
    /// first use.
    pub(super) fn hover_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        let document_uri = self.session_document_uri(uri);
        let Some(text) = self.document_texts.get(uri).cloned() else {
            return Ok(None);
        };

        if self.editor_lsp.is_none() {
            let root = self.editor_lsp_root();
            let executable = self.executable.clone();
            let cwd = self.cwd.clone();
            self.editor_lsp = Some(EditorLspSession::spawn(executable.as_str(), &cwd, &root)?);
        }
        let Some(session) = self.editor_lsp.as_mut() else {
            return Ok(None);
        };
        session.hover(document_uri.as_str(), text.as_str(), line, character)
    }

    pub(super) fn completion_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        let document_uri = self.session_document_uri(uri);
        let Some(text) = self.document_texts.get(uri).cloned() else {
            return Ok(None);
        };

        if self.editor_lsp.is_none() {
            let root = self.editor_lsp_root();
            let executable = self.executable.clone();
            let cwd = self.cwd.clone();
            self.editor_lsp = Some(EditorLspSession::spawn(executable.as_str(), &cwd, &root)?);
        }
        let Some(session) = self.editor_lsp.as_mut() else {
            return Ok(None);
        };
        session.completion(document_uri.as_str(), text.as_str(), line, character)
    }

    /// Drop the editor session so the next request respawns it. Used when the
    /// overlay root moves under a materialized project session.
    pub(super) fn retire_editor_lsp(&mut self) {
        self.editor_lsp = None;
    }

    fn editor_lsp_root(&self) -> PathBuf {
        if self.materialized_project_session {
            super::session_paths::overlay_root_for_project(&self.project_root)
        } else {
            self.project_root.clone()
        }
    }
}
