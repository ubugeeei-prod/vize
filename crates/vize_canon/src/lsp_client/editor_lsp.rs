//! Reusable project editor state over Corsa's `--lsp --stdio` transport.
//!
//! The project-session API exposes diagnostics but rejects `hover` (and the
//! other editor requests) as `CorsaError::Unsupported` on every pinned runtime
//! — see ubugeeei-prod/corsa-bind#409. The very same runtime does advertise
//! `hoverProvider` over its LSP transport, so editor features route through a
//! lazily spawned session that mirrors the virtual documents in. Standard tsgo
//! diagnostics use the same session so semantic requests share one project
//! identity and one overlay generation.
//!
//! The session is lazy on purpose: typecheck-only runs never pay for the extra
//! process, and a session that has answered one hover is reused later.

use super::{
    CorsaProjectClient, diagnostics_lsp::initialize_lsp_client,
    language_id::for_uri as language_id_for_uri,
};
use corsa::runtime::block_on;
use corsa_lsp::{LspClient, LspOverlay, LspSpawnConfig, VirtualDocument};
use lsp_types::{DocumentDiagnosticReportResult, Uri};
use serde_json::Value;
use std::{
    path::Path,
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use vize_carton::{FxHashMap, FxHashSet, String, cstr};

mod client;
mod file_rename;
mod readiness;
mod requests;
mod responder;
mod retry;
mod synchronize;
#[cfg(test)]
mod tests;
mod type_definition;

use requests::{
    RawCompletionRequest, RawDefinitionRequest, RawHoverRequest, RawPrepareRenameRequest,
    RawReferencesRequest, RawRenameRequest, RawSignatureHelpRequest, RawWillRenameFilesRequest,
    signature_help_request_params, will_rename_files_request_params,
};
use responder::spawn_responder;

/// A reusable `--lsp --stdio` session for standard diagnostics and editor
/// requests.
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
    /// Overlay notifications written since the last response-backed barrier.
    /// This spans synchronization calls because workspace discovery can add
    /// one document per call faster than the bounded transport drains them.
    unacknowledged_notifications: usize,
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
            unacknowledged_notifications: 0,
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

    fn diagnostics(
        &mut self,
        document_uri: &str,
    ) -> Result<DocumentDiagnosticReportResult, String> {
        let uri = self.ready_document_uri(document_uri)?;
        super::diagnostics_lsp::request_lsp_document_diagnostics(&self.client, &uri).map_err(
            |error| cstr!("Failed to request editor LSP diagnostics for {document_uri}: {error}"),
        )
    }

    fn will_rename_files(&mut self, renames: &[(&str, &str)]) -> Result<Option<Value>, String> {
        self.ready_workspace_request()?;
        block_on(
            self.client
                .request::<RawWillRenameFilesRequest>(will_rename_files_request_params(renames)),
        )
        .map_err(|error| cstr!("Failed to request editor LSP workspace/willRenameFiles: {error}"))
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
        self.finish_close(first_error)
    }

    /// Tear down the editor process without waiting for a protocol shutdown
    /// response. Workspace file events use this path when the reusable editor
    /// project view has to be discarded before the next semantic request; a
    /// stuck old server must not block the foreground LSP notification queue.
    fn discard(&mut self) -> Result<(), String> {
        if self.closed {
            return Ok(());
        }

        let mut first_error = None;
        if let Err(error) = block_on(self.client.close()) {
            first_error = Some(cstr!("Failed to close editor LSP process: {error}"));
        }
        self.finish_close(first_error)
    }

    fn finish_close(&mut self, mut first_error: Option<String>) -> Result<(), String> {
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
