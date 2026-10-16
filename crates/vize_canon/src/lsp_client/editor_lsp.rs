//! Reusable project editor state over Corsa's `--lsp --stdio` transport.
//!
//! The project-session API exposes diagnostics but rejects `hover` (and the
//! other editor requests) as `CorsaError::Unsupported` on every pinned runtime
//! — see ubugeeei-prod/corsa-bind#409. The very same runtime does advertise
//! `hoverProvider` over its LSP transport, so editor features route through a
//! lazily spawned session that mirrors the virtual documents in. Standard tsgo
//! diagnostics use the same session so semantic requests share one project
//! identity and one overlay generation.
//! The session is lazy on purpose: typecheck-only runs never pay for the extra
//! process, and a session that has answered one hover is reused later.

use super::{
    CorsaProjectClient, diagnostics_lsp::initialize_lsp_client,
    language_id::for_uri as language_id_for_uri,
};
use corsa::runtime::block_on;
use corsa_lsp::{LspClient, LspOverlay, LspSpawnConfig, VirtualDocument, process::ProcessObserver};
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
use vize_l0::{FxHashMap, FxHashSet, String, cstr};
#[cfg(test)]
mod batch_test_receipt;
mod call_hierarchy;
mod client;
mod code_actions;
mod completion_resolve;
mod component_types;
pub(super) mod configured_project;
mod declaration;
mod file_rename;
mod implementation;
mod inlay_hint;
mod materialized;
mod native_shutdown;
mod native_vue;
mod original_program;
mod process_close;
mod process_receipt;
mod project_configuration;
mod project_identity;
mod readiness;
mod requests;
mod responder;
mod retirement;
mod retry;
pub mod snapshot_source;
mod synchronize;
#[cfg(test)]
mod tests;
mod type_definition;
pub(super) use original_program::OriginalDiagnosingSession;
use requests::{
    RawDefinitionRequest, RawHoverRequest, RawPrepareRenameRequest, RawReferencesRequest,
    RawRenameRequest, RawSignatureHelpRequest, RawWillRenameFilesRequest,
    signature_help_request_params, will_rename_files_request_params,
};
use responder::spawn_responder;
/// A reusable `--lsp --stdio` session for standard diagnostics and editor
/// requests.
pub(super) struct EditorLspSession {
    startup_control: Option<super::startup_control::StartupControl>,
    startup_registration: Option<super::startup_control::Registration>,
    client: LspClient,
    process_observer: ProcessObserver,
    overlay: LspOverlay,
    stop: Arc<AtomicBool>,
    responder: Option<std::thread::JoinHandle<()>>,
    configured_api: Option<project_configuration::DiagnosingApi>,
    closed: bool,
    shutdown_to_eof: bool,
    #[cfg(test)]
    native_initialize_response: Value,
    /// Last text mirrored into the server, keyed by session document URI.
    documents: FxHashMap<String, String>,
    issued_completions: completion_resolve::IssuedCompletions,
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
    #[cfg(test)]
    fn spawn(executable: &str, cwd: &Path, project_root: &Path) -> Result<Self, String> {
        Self::spawn_with_control(executable, cwd, project_root, None)
    }

    fn spawn_with_control(
        executable: &str,
        cwd: &Path,
        project_root: &Path,
        control: Option<super::startup_control::StartupControl>,
    ) -> Result<Self, String> {
        let mut session = Self::spawn_initializing(executable, cwd, project_root, control)?;
        session.finish_startup()?;
        Ok(session)
    }

    fn finish_startup(&mut self) -> Result<(), String> {
        if let Some(control) = &self.startup_control {
            control.check_admission()?;
        }
        self.startup_registration.take();
        Ok(())
    }

    fn spawn_initializing(
        executable: &str,
        cwd: &Path,
        project_root: &Path,
        startup_control: Option<super::startup_control::StartupControl>,
    ) -> Result<Self, String> {
        let process_observer = ProcessObserver::default();
        let mut spawn = LspSpawnConfig::new(executable).with_cwd(cwd.to_path_buf());
        spawn.command = spawn
            .command
            .with_process_observer(process_observer.clone());
        let (client, registration) = match &startup_control {
            Some(control) => {
                let (client, registration) =
                    control.spawn_editor(spawn, process_observer.clone())?;
                (client, Some(registration))
            }
            None => (
                block_on(LspClient::spawn(spawn))
                    .map_err(|error| cstr!("Failed to start Corsa editor LSP session: {error}"))?,
                None,
            ),
        };
        let stop = Arc::new(AtomicBool::new(false));
        let responder = spawn_responder(client.clone(), stop.clone());

        let initialized = match initialize_lsp_client(&client, project_root) {
            Ok(response) => response,
            Err(error) => {
                crate::lsp_client::retirement::ownership_error(
                    &client.process_ownership(),
                    cstr!("native editor initialization failed: {error}").as_str(),
                );
                stop.store(true, Ordering::Relaxed);
                let _ = block_on(client.close());
                let _ = responder.join();
                let _ = process_receipt::report(&process_observer, false, false);
                return Err(error);
            }
        };
        let shutdown_to_eof =
            native_shutdown::eof_capability(Path::new(executable), &initialized, &process_observer);
        crate::corsa_session_cache::note_typescript_project_init();

        Ok(Self {
            startup_control,
            startup_registration: registration,
            overlay: client.overlay(),
            client,
            process_observer,
            stop,
            responder: Some(responder),
            configured_api: None,
            closed: false,
            shutdown_to_eof,
            #[cfg(test)]
            native_initialize_response: initialized,
            documents: Default::default(),
            issued_completions: Default::default(),
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
}
