//! `CorsaProjectClient` entry points that route editor requests through the
//! lazily spawned editor LSP session.

use corsa::runtime::block_on;
use lsp_types::DocumentDiagnosticReportResult;
use serde_json::Value;
use std::path::PathBuf;
use vize_l0::{FxHashMap, String, cstr};

use super::{CorsaProjectClient, EditorLspSession, retry::retry_transient_editor_request};

impl CorsaProjectClient {
    /// A shared API snapshot also reads the editor's open buffers. Materialized
    /// refreshes reach it before the editor's pending didChange, so refresh it
    /// once the editor acknowledges the current buffers (#10730).
    pub(in crate::lsp_client) fn refresh_acknowledged_editor_snapshot(
        &mut self,
        document_uri: &str,
    ) -> Result<(), String> {
        if !self.project_session_attached_to_editor {
            return Ok(());
        }
        // Borrow the existing owner only; never spawn or replace a transport here.
        if self.editor_lsp.is_none() || self.session.is_none() {
            return Err(cstr!("Attached editor snapshot owner is unavailable"));
        }
        self.editor_lsp_session()?
            .ready_document_uri(document_uri)?;
        let session = self
            .session
            .as_mut()
            .ok_or_else(|| cstr!("Attached editor snapshot owner is unavailable"))?;
        block_on(session.refresh(None))
            .map_err(|error| cstr!("Failed to refresh the acknowledged editor snapshot: {error}"))
    }

    /// Answer a hover through the editor LSP transport, spawning the session on
    /// first use.
    pub(in crate::lsp_client) fn hover_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        let phase = crate::corsa_bridge::preparation_trace::Phase::start("editor_hover", 1);
        let result = self.request_with_editor_lsp_query_recovery(uri, |session| {
            session.hover(uri, line, character)
        });
        if result.is_ok() {
            phase.finish();
        }
        result
    }

    pub(in crate::lsp_client) fn completion_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        self.request_with_editor_lsp_query_recovery(uri, |session| {
            session.completion(uri, line, character)
        })
    }

    pub(in crate::lsp_client) fn definition_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        self.request_with_editor_lsp_query_recovery(uri, |session| {
            session.definition(uri, line, character)
        })
    }

    pub(in crate::lsp_client) fn type_definition_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        self.request_with_editor_lsp_query_recovery(uri, |session| {
            session.type_definition(uri, line, character)
        })
    }

    pub(in crate::lsp_client) fn declaration_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        self.request_with_editor_lsp_query_recovery(uri, |session| {
            session.declaration(uri, line, character)
        })
    }

    pub(in crate::lsp_client) fn implementation_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        self.request_with_editor_lsp_query_recovery(uri, |session| {
            session.implementation(uri, line, character)
        })
    }

    pub(in crate::lsp_client) fn prepare_call_hierarchy_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        self.request_with_editor_lsp_query_recovery(uri, |session| {
            session.prepare_call_hierarchy(uri, line, character)
        })
    }

    pub(in crate::lsp_client) fn call_hierarchy_incoming_calls_via_editor_lsp(
        &mut self,
        item: Value,
    ) -> Result<Option<Value>, String> {
        self.request_with_editor_lsp_recovery(|session| {
            session.call_hierarchy_incoming_calls(item.clone())
        })
    }

    pub(in crate::lsp_client) fn call_hierarchy_outgoing_calls_via_editor_lsp(
        &mut self,
        item: Value,
    ) -> Result<Option<Value>, String> {
        self.request_with_editor_lsp_recovery(|session| {
            session.call_hierarchy_outgoing_calls(item.clone())
        })
    }

    pub(in crate::lsp_client) fn references_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
        include_declaration: bool,
    ) -> Result<Option<Value>, String> {
        self.request_with_editor_lsp_query_recovery(uri, |session| {
            session.references(uri, line, character, include_declaration)
        })
    }

    pub(in crate::lsp_client) fn prepare_rename_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        self.request_with_editor_lsp_query_recovery(uri, |session| {
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
        self.request_with_editor_lsp_query_recovery(uri, |session| {
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
        self.request_with_editor_lsp_query_recovery(uri, |session| {
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

    fn request_with_editor_lsp_query_recovery(
        &mut self,
        uri: &str,
        request: impl FnMut(&mut EditorLspSession) -> Result<Option<Value>, String>,
    ) -> Result<Option<Value>, String> {
        if self.document_texts.contains_key(uri) {
            return self.request_with_editor_lsp_recovery(request);
        }
        // An unchanged, closed importer may depend on this live unsaved graph.
        // Include its real bytes in a request-local view, without retaining a
        // new caller overlay or borrowing a stale independent API snapshot.
        if self.logical_workspace_overlays && !self.document_texts.is_empty() {
            let path = crate::file_uri::file_uri_to_path(uri)
                .ok_or_else(|| cstr!("Closed logical Corsa query requires a file URI: {uri}"))?;
            let text = match std::fs::read_to_string(path) {
                Ok(text) => text,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(error) => {
                    return Err(cstr!(
                        "Failed to read closed logical Corsa query {uri}: {error}"
                    ));
                }
            };
            let mut documents = self.document_texts.clone();
            documents.insert(uri.into(), text.into());
            return self.request_with_editor_lsp_documents_recovery(&documents, request);
        }
        Ok(None)
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
        crate::corsa_bridge::native_operation::checkpoint()?;
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
        crate::corsa_bridge::native_operation::checkpoint()?;
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

    pub(super) fn editor_lsp_session(&mut self) -> Result<&mut EditorLspSession, String> {
        self.validate_logical_workspace_documents(&self.document_texts)?;
        self.ensure_editor_transport()?;
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
        self.validate_logical_workspace_documents(documents)?;
        let keep_dirty_after_sync = !document_maps_equal(documents, &self.document_texts);
        self.ensure_editor_transport()?;
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

    #[cfg(test)]
    pub(crate) fn logical_configuration_for_test(&mut self, uri: &str) -> Result<PathBuf, String> {
        self.editor_lsp_session()?
            .diagnosing_configuration(uri)
            .map_err(|error| match error {
                super::project_identity::ProjectIdentityError::Communication(error) => error,
                super::project_identity::ProjectIdentityError::Unconfigured => {
                    cstr!("The actual diagnosing LSP has no configured project for {uri}")
                }
            })
    }

    pub(in crate::lsp_client) fn editor_lsp_project_root(&self) -> PathBuf {
        if self.materialized_project_session {
            self.overlay_storage.root().to_path_buf()
        } else {
            self.project_root.clone()
        }
    }

    /// Drop the editor session so the next request respawns it after a project
    /// session transition.
    pub(in crate::lsp_client) fn retire_editor_lsp(&mut self) -> Result<(), String> {
        if self.project_session_attached_to_editor {
            return self.retire_shared_editor_transport(false);
        }
        let result = match self.editor_lsp.as_mut() {
            Some(session) => session.shutdown(),
            None => Ok(()),
        };
        self.editor_lsp = None;
        self.editor_lsp_documents_dirty = true;
        self.overlay_storage.observe_retirement(&result);
        result
    }

    /// Drop the editor session immediately after an external project-shape
    /// change. Unlike [`Self::retire_editor_lsp`], this does not wait for a
    /// protocol `shutdown` response from the stale process, so file-operation
    /// notifications cannot hold the foreground LSP queue hostage.
    pub(in crate::lsp_client) fn discard_editor_lsp(&mut self) -> Result<(), String> {
        if self.project_session_attached_to_editor {
            return self.retire_shared_editor_transport(true);
        }
        let result = match self.editor_lsp.as_mut() {
            Some(session) => session.discard(),
            None => Ok(()),
        };
        self.editor_lsp = None;
        self.editor_lsp_documents_dirty = true;
        self.overlay_storage.observe_retirement(&result);
        result
    }
}

fn document_maps_equal(lhs: &FxHashMap<String, String>, rhs: &FxHashMap<String, String>) -> bool {
    lhs.len() == rhs.len()
        && lhs
            .iter()
            .all(|(uri, text)| rhs.get(uri.as_str()).is_some_and(|current| current == text))
}
