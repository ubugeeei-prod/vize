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

use super::{
    CorsaProjectClient, diagnostics_lsp::initialize_lsp_client,
    language_id::for_uri as language_id_for_uri,
};
use corsa::runtime::block_on;
use corsa_lsp::{LspClient, LspOverlay, LspSpawnConfig, VirtualDocument, jsonrpc::InboundEvent};
use lsp_types::Uri;
use serde_json::Value;
use std::{
    path::Path,
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use vize_carton::{FxHashMap, FxHashSet, String, cstr};

mod client;
mod readiness;
mod requests;
#[cfg(test)]
mod tests;

use requests::{
    RawCompletionRequest, RawDefinitionRequest, RawHoverRequest, RawPrepareRenameRequest,
    RawReferencesRequest, RawRenameRequest, RawSignatureHelpRequest,
};

/// A reusable `--lsp --stdio` session used only for editor requests.
pub(super) struct EditorLspSession {
    client: LspClient,
    overlay: LspOverlay,
    stop: Arc<AtomicBool>,
    responder: Option<std::thread::JoinHandle<()>>,
    closed: bool,
    /// Last text mirrored into the server, keyed by session document URI.
    documents: FxHashMap<String, String>,
    /// Latest notification generation written to this transport.
    document_generation: u64,
    /// Generation acknowledged by a response-backed semantic request.
    ready_generation: Option<u64>,
    /// Documents opened or changed since the acknowledged generation.
    dirty_documents: FxHashSet<String>,
    /// A close cannot be diagnosed directly, so the next query must act as
    /// the response-backed barrier for that topology change.
    query_barrier_required: bool,
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
            closed: false,
            documents: Default::default(),
            document_generation: 0,
            ready_generation: None,
            dirty_documents: Default::default(),
            query_barrier_required: false,
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
                self.overlay
                    .open(VirtualDocument::new(
                        uri.clone(),
                        language_id_for_uri(document_uri),
                        text,
                    ))
                    .map_err(|error| {
                        cstr!("Failed to open editor LSP overlay for {document_uri}: {error}")
                    })?;
            }
        }
        self.documents.insert(document_uri.into(), text.into());
        self.dirty_documents.insert(document_uri.into());
        self.advance_document_generation();
        Ok(uri)
    }

    /// Bring the reusable editor transport to the same virtual-project view as
    /// the project-session transport, including dependency removals.
    fn synchronize(&mut self, documents: &FxHashMap<String, String>) -> Result<(), String> {
        let removed = self
            .documents
            .keys()
            .filter(|uri| !documents.contains_key(uri.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        for document_uri in removed {
            let uri = Uri::from_str(document_uri.as_str())
                .map_err(|error| cstr!("Invalid LSP document URI {document_uri}: {error}"))?;
            self.overlay.close(&uri).map_err(|error| {
                cstr!("Failed to close editor LSP overlay for {document_uri}: {error}")
            })?;
            self.documents.remove(document_uri.as_str());
            self.dirty_documents.remove(document_uri.as_str());
            self.query_barrier_required = true;
            self.advance_document_generation();
        }
        for (document_uri, text) in documents {
            self.mirror(document_uri, text)?;
        }
        Ok(())
    }

    fn hover(
        &mut self,
        document_uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        let uri = self.ready_document_uri(document_uri)?;
        block_on(self.client.request::<RawHoverRequest>(serde_json::json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": character },
        })))
        .map_err(|error| cstr!("Failed to request editor LSP hover: {error}"))
    }

    fn completion(
        &mut self,
        document_uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        let uri = self.ready_document_uri(document_uri)?;
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

    fn definition(
        &mut self,
        document_uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        let uri = self.ready_document_uri(document_uri)?;
        block_on(
            self.client
                .request::<RawDefinitionRequest>(serde_json::json!({
                    "textDocument": { "uri": uri },
                    "position": { "line": line, "character": character },
                })),
        )
        .map_err(|error| cstr!("Failed to request editor LSP definition: {error}"))
    }

    fn references(
        &mut self,
        document_uri: &str,
        line: u32,
        character: u32,
        include_declaration: bool,
    ) -> Result<Option<Value>, String> {
        let uri = self.ready_document_uri(document_uri)?;
        block_on(
            self.client
                .request::<RawReferencesRequest>(serde_json::json!({
                    "textDocument": { "uri": uri },
                    "position": { "line": line, "character": character },
                    "context": { "includeDeclaration": include_declaration },
                })),
        )
        .map_err(|error| cstr!("Failed to request editor LSP references: {error}"))
    }

    fn prepare_rename(
        &mut self,
        document_uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        let uri = self.ready_document_uri(document_uri)?;
        block_on(
            self.client
                .request::<RawPrepareRenameRequest>(serde_json::json!({
                    "textDocument": { "uri": uri },
                    "position": { "line": line, "character": character },
                })),
        )
        .map_err(|error| cstr!("Failed to request editor LSP prepare rename: {error}"))
    }

    fn rename(
        &mut self,
        document_uri: &str,
        line: u32,
        character: u32,
        new_name: &str,
    ) -> Result<Option<Value>, String> {
        let uri = self.ready_document_uri(document_uri)?;
        block_on(self.client.request::<RawRenameRequest>(serde_json::json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": character },
            "newName": new_name,
        })))
        .map_err(|error| cstr!("Failed to request editor LSP rename: {error}"))
    }

    fn signature_help(
        &mut self,
        document_uri: &str,
        line: u32,
        character: u32,
        context: Option<Value>,
    ) -> Result<Option<Value>, String> {
        let uri = self.ready_document_uri(document_uri)?;
        block_on(
            self.client
                .request::<RawSignatureHelpRequest>(signature_help_request_params(
                    &uri, line, character, context,
                )),
        )
        .map_err(|error| cstr!("Failed to request editor LSP signature help: {error}"))
    }

    /// Complete the standard LSP lifecycle before closing and reaping the
    /// owned process. The responder remains alive until `shutdown` returns so
    /// server-initiated requests cannot deadlock the final response.
    fn shutdown(&mut self) -> Result<(), String> {
        if self.closed {
            return Ok(());
        }

        let mut first_error = None;
        if let Err(error) = block_on(self.client.graceful_close()) {
            first_error = Some(cstr!(
                "Failed to gracefully close editor LSP process: {error}"
            ));
        }
        self.stop.store(true, Ordering::Relaxed);
        if let Some(responder) = self.responder.take()
            && responder.join().is_err()
            && first_error.is_none()
        {
            first_error = Some(cstr!("Editor LSP responder panicked during shutdown"));
        }
        self.closed = true;

        match first_error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}

impl Drop for EditorLspSession {
    fn drop(&mut self) {
        let _ = self.shutdown();
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

fn signature_help_request_params(
    uri: &Uri,
    line: u32,
    character: u32,
    context: Option<Value>,
) -> Value {
    let context = context.unwrap_or_else(|| {
        serde_json::json!({
            "triggerKind": 1,
            "isRetrigger": false
        })
    });
    serde_json::json!({
        "textDocument": { "uri": uri },
        "position": { "line": line, "character": character },
        "context": context,
    })
}
