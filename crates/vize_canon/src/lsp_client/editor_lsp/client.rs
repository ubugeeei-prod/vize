//! `CorsaProjectClient` entry points that route editor requests through the
//! lazily spawned editor LSP session.

use lsp_types::DocumentDiagnosticReportResult;
use serde_json::Value;
use std::path::PathBuf;
use vize_carton::{FxHashMap, String, cstr};

use super::{CorsaProjectClient, EditorLspSession, retry::retry_transient_editor_request};
use crate::lsp_client::session_paths::overlay_root_for_project;

impl CorsaProjectClient {
    /// Answer a hover through the editor LSP transport, spawning the session on
    /// first use.
    pub(in crate::lsp_client) fn hover_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        if !self.document_texts.contains_key(uri) {
            return Ok(None);
        }
        self.request_with_editor_lsp_recovery(|session| session.hover(uri, line, character))
    }

    pub(in crate::lsp_client) fn completion_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        if !self.document_texts.contains_key(uri) {
            return Ok(None);
        }
        self.request_with_editor_lsp_recovery(|session| session.completion(uri, line, character))
    }

    pub(in crate::lsp_client) fn definition_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        if !self.document_texts.contains_key(uri) {
            return Ok(None);
        }
        self.request_with_editor_lsp_recovery(|session| session.definition(uri, line, character))
    }

    pub(in crate::lsp_client) fn type_definition_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        if !self.document_texts.contains_key(uri) {
            return Ok(None);
        }
        self.request_with_editor_lsp_recovery(|session| {
            session.type_definition(uri, line, character)
        })
    }

    pub(in crate::lsp_client) fn references_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
        include_declaration: bool,
    ) -> Result<Option<Value>, String> {
        if !self.document_texts.contains_key(uri) {
            return Ok(None);
        }
        self.request_with_editor_lsp_recovery(|session| {
            session.references(uri, line, character, include_declaration)
        })
    }

    pub(in crate::lsp_client) fn prepare_rename_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        if !self.document_texts.contains_key(uri) {
            return Ok(None);
        }
        self.request_with_editor_lsp_recovery(|session| {
            session.prepare_rename(uri, line, character)
        })
    }

    pub(in crate::lsp_client) fn rename_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
        new_name: &str,
    ) -> Result<Option<Value>, String> {
        if !self.document_texts.contains_key(uri) {
            return Ok(None);
        }
        self.request_with_editor_lsp_recovery(|session| {
            session.rename(uri, line, character, new_name)
        })
    }

    pub(in crate::lsp_client) fn signature_help_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
        context: Option<Value>,
    ) -> Result<Option<Value>, String> {
        if !self.document_texts.contains_key(uri) {
            return Ok(None);
        }
        self.request_with_editor_lsp_recovery(|session| {
            session.signature_help(uri, line, character, context.clone())
        })
    }

    pub(in crate::lsp_client) fn diagnostics_via_editor_lsp(
        &mut self,
        document_uri: &str,
        documents: &FxHashMap<String, String>,
    ) -> Result<DocumentDiagnosticReportResult, String> {
        if !documents.contains_key(document_uri) {
            return Err(cstr!(
                "Editor LSP diagnostic project does not contain {document_uri}"
            ));
        }
        self.request_with_editor_lsp_documents_recovery(documents, |session| {
            session.diagnostics(document_uri)
        })
    }

    /// Execute an idempotent editor query and rebuild the reusable LSP session
    /// once when its transport has become unusable.
    ///
    /// Recreating the session also marks every current virtual document dirty;
    /// [`editor_lsp_session`](Self::editor_lsp_session) synchronizes that full
    /// project before the retry. Semantic and protocol-shape failures are never
    /// retried.
    pub(super) fn request_with_editor_lsp_recovery<T>(
        &mut self,
        mut request: impl FnMut(&mut EditorLspSession) -> Result<T, String>,
    ) -> Result<T, String> {
        let first = self.editor_lsp_session().and_then(&mut request);
        retry_transient_editor_request(
            self,
            first,
            CorsaProjectClient::retire_editor_lsp,
            |client| client.editor_lsp_session().and_then(request),
        )
    }

    fn request_with_editor_lsp_documents_recovery<T>(
        &mut self,
        documents: &FxHashMap<String, String>,
        mut request: impl FnMut(&mut EditorLspSession) -> Result<T, String>,
    ) -> Result<T, String> {
        let first = self
            .editor_lsp_session_for_documents(documents)
            .and_then(&mut request);
        retry_transient_editor_request(
            self,
            first,
            CorsaProjectClient::retire_editor_lsp,
            |client| {
                client
                    .editor_lsp_session_for_documents(documents)
                    .and_then(request)
            },
        )
    }

    fn editor_lsp_session(&mut self) -> Result<&mut EditorLspSession, String> {
        let project_root = self.editor_lsp_project_root();
        if self.editor_lsp.is_none() {
            self.editor_lsp = Some(EditorLspSession::spawn(
                self.executable.as_str(),
                &self.cwd,
                &project_root,
            )?);
            self.editor_lsp_documents_dirty = true;
        }
        let session = self
            .editor_lsp
            .as_mut()
            .ok_or_else(|| cstr!("Corsa editor LSP session did not initialize"))?;
        if self.editor_lsp_documents_dirty {
            session.synchronize(&self.document_texts)?;
            self.editor_lsp_documents_dirty = false;
        }
        Ok(session)
    }

    fn editor_lsp_session_for_documents(
        &mut self,
        documents: &FxHashMap<String, String>,
    ) -> Result<&mut EditorLspSession, String> {
        let project_root = self.editor_lsp_project_root();
        let keep_dirty_after_sync = !document_maps_equal(documents, &self.document_texts);
        if self.editor_lsp.is_none() {
            self.editor_lsp = Some(EditorLspSession::spawn(
                self.executable.as_str(),
                &self.cwd,
                &project_root,
            )?);
            self.editor_lsp_documents_dirty = true;
        }
        let session = self
            .editor_lsp
            .as_mut()
            .ok_or_else(|| cstr!("Corsa editor LSP session did not initialize"))?;
        if self.editor_lsp_documents_dirty || !session.has_documents(documents) {
            session.synchronize(documents)?;
            self.editor_lsp_documents_dirty = keep_dirty_after_sync;
        }
        Ok(session)
    }

    fn editor_lsp_project_root(&self) -> PathBuf {
        if self.materialized_project_session {
            overlay_root_for_project(&self.project_root)
        } else {
            self.project_root.clone()
        }
    }

    /// Drop the editor session so the next request respawns it after a project
    /// session transition.
    pub(in crate::lsp_client) fn retire_editor_lsp(&mut self) -> Result<(), String> {
        let result = match self.editor_lsp.as_mut() {
            Some(session) => session.shutdown(),
            None => Ok(()),
        };
        self.editor_lsp = None;
        self.editor_lsp_documents_dirty = true;
        result
    }

    /// Drop the editor session immediately after an external project-shape
    /// change. Unlike [`Self::retire_editor_lsp`], this does not wait for a
    /// protocol `shutdown` response from the stale process, so file-operation
    /// notifications cannot hold the foreground LSP queue hostage.
    pub(in crate::lsp_client) fn discard_editor_lsp(&mut self) -> Result<(), String> {
        let result = match self.editor_lsp.as_mut() {
            Some(session) => session.discard(),
            None => Ok(()),
        };
        self.editor_lsp = None;
        self.editor_lsp_documents_dirty = true;
        result
    }
}

fn document_maps_equal(lhs: &FxHashMap<String, String>, rhs: &FxHashMap<String, String>) -> bool {
    lhs.len() == rhs.len()
        && lhs
            .iter()
            .all(|(uri, text)| rhs.get(uri.as_str()).is_some_and(|current| current == text))
}
