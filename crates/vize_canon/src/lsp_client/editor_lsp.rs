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
use corsa::{
    jsonrpc::InboundEvent,
    lsp::{LspClient, LspOverlay, LspSpawnConfig, VirtualDocument},
    runtime::block_on,
};
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

struct RawDefinitionRequest;

impl lsp_types::request::Request for RawDefinitionRequest {
    type Params = Value;
    type Result = Option<Value>;
    const METHOD: &'static str = "textDocument/definition";
}

struct RawReferencesRequest;

impl lsp_types::request::Request for RawReferencesRequest {
    type Params = Value;
    type Result = Option<Value>;
    const METHOD: &'static str = "textDocument/references";
}

struct RawPrepareRenameRequest;

impl lsp_types::request::Request for RawPrepareRenameRequest {
    type Params = Value;
    type Result = Option<Value>;
    const METHOD: &'static str = "textDocument/prepareRename";
}

struct RawRenameRequest;

impl lsp_types::request::Request for RawRenameRequest {
    type Params = Value;
    type Result = Option<Value>;
    const METHOD: &'static str = "textDocument/rename";
}

struct RawSignatureHelpRequest;

impl lsp_types::request::Request for RawSignatureHelpRequest {
    type Params = Value;
    type Result = Option<Value>;
    const METHOD: &'static str = "textDocument/signatureHelp";
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
        }
        for (document_uri, text) in documents {
            self.mirror(document_uri, text)?;
        }
        Ok(())
    }

    fn document_uri(&self, document_uri: &str) -> Result<Uri, String> {
        if !self.documents.contains_key(document_uri) {
            return Err(cstr!(
                "Editor LSP virtual project does not contain {document_uri}"
            ));
        }
        Uri::from_str(document_uri)
            .map_err(|error| cstr!("Invalid LSP document URI {document_uri}: {error}"))
    }

    fn hover(
        &mut self,
        document_uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        let uri = self.document_uri(document_uri)?;
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
        let uri = self.document_uri(document_uri)?;
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
        let uri = self.document_uri(document_uri)?;
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
        let uri = self.document_uri(document_uri)?;
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
        let uri = self.document_uri(document_uri)?;
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
        let uri = self.document_uri(document_uri)?;
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
        let uri = self.document_uri(document_uri)?;
        block_on(
            self.client
                .request::<RawSignatureHelpRequest>(signature_help_request_params(
                    &uri, line, character, context,
                )),
        )
        .map_err(|error| cstr!("Failed to request editor LSP signature help: {error}"))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_help_request_defaults_to_manual_invocation() {
        let uri = Uri::from_str("file:///workspace/App.vue.ts").unwrap();
        let params = signature_help_request_params(&uri, 4, 7, None);

        assert_eq!(
            params["context"],
            serde_json::json!({"triggerKind": 1, "isRetrigger": false})
        );
    }

    #[test]
    fn signature_help_request_preserves_client_context_losslessly() {
        let uri = Uri::from_str("file:///workspace/App.vue.ts").unwrap();
        let context = serde_json::json!({
            "triggerKind": 2,
            "triggerCharacter": ",",
            "isRetrigger": true,
            "activeSignatureHelp": {
                "signatures": [{"label": "format(value: string, radix: number): string"}],
                "activeSignature": 0,
                "activeParameter": 1
            }
        });
        let params = signature_help_request_params(&uri, 8, 13, Some(context.clone()));

        assert_eq!(params["context"], context);
    }
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
        if !self.document_texts.contains_key(uri) {
            return Ok(None);
        }
        self.editor_lsp_session()?.hover(uri, line, character)
    }

    pub(super) fn completion_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        if !self.document_texts.contains_key(uri) {
            return Ok(None);
        }
        self.editor_lsp_session()?.completion(uri, line, character)
    }

    pub(super) fn definition_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        if !self.document_texts.contains_key(uri) {
            return Ok(None);
        }
        self.editor_lsp_session()?.definition(uri, line, character)
    }

    pub(super) fn references_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
        include_declaration: bool,
    ) -> Result<Option<Value>, String> {
        if !self.document_texts.contains_key(uri) {
            return Ok(None);
        }
        self.editor_lsp_session()?
            .references(uri, line, character, include_declaration)
    }

    pub(super) fn prepare_rename_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        if !self.document_texts.contains_key(uri) {
            return Ok(None);
        }
        self.editor_lsp_session()?
            .prepare_rename(uri, line, character)
    }

    pub(super) fn rename_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
        new_name: &str,
    ) -> Result<Option<Value>, String> {
        if !self.document_texts.contains_key(uri) {
            return Ok(None);
        }
        self.editor_lsp_session()?
            .rename(uri, line, character, new_name)
    }

    pub(super) fn signature_help_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
        context: Option<Value>,
    ) -> Result<Option<Value>, String> {
        if !self.document_texts.contains_key(uri) {
            return Ok(None);
        }
        self.editor_lsp_session()?
            .signature_help(uri, line, character, context)
    }

    fn editor_lsp_session(&mut self) -> Result<&mut EditorLspSession, String> {
        if self.editor_lsp.is_none() {
            self.editor_lsp = Some(EditorLspSession::spawn(
                self.executable.as_str(),
                &self.cwd,
                &self.project_root,
            )?);
            self.editor_lsp_documents_dirty = true;
        }
        if self.editor_lsp_documents_dirty {
            let session = self
                .editor_lsp
                .as_mut()
                .ok_or_else(|| cstr!("Corsa editor LSP session did not initialize"))?;
            session.synchronize(&self.document_texts)?;
            self.editor_lsp_documents_dirty = false;
        }
        self.editor_lsp
            .as_mut()
            .ok_or_else(|| cstr!("Corsa editor LSP session did not initialize"))
    }

    /// Drop the editor session so the next request respawns it after a project
    /// session transition.
    pub(super) fn retire_editor_lsp(&mut self) {
        self.editor_lsp = None;
        self.editor_lsp_documents_dirty = true;
    }
}
