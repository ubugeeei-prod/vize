//! `CorsaProjectClient` entry points that route editor requests through the
//! lazily spawned editor LSP session.

use corsa::runtime::block_on;
use lsp_types::DocumentDiagnosticReportResult;
use serde_json::Value;
use std::path::PathBuf;
use vize_l0::{FxHashMap, String, cstr};

use super::{CorsaProjectClient, EditorLspSession, retry::retry_transient_editor_request};
use crate::lsp_client::session_paths::overlay_root_for_project;

impl CorsaProjectClient {
    /// An attached API snapshot must follow the acknowledged editor overlay.
    /// A disk refresh before didChange can retain the previous open buffer in
    /// its immutable snapshot even after subsequent LSP queries see the edit.
    pub(in crate::lsp_client) fn refresh_acknowledged_editor_snapshot(
        &mut self,
        logical_uri: &str,
    ) -> Result<(), String> {
        if !self.project_session_attached_to_editor {
            return Ok(());
        }
        // This route only borrows the existing shared owner. An inconsistent
        // attachment must not initialize or replace either transport here.
        if self.editor_lsp.is_none() || self.session.is_none() {
            return Err(cstr!("Attached editor snapshot owner is unavailable"));
        }
        let phase =
            crate::corsa_bridge::preparation_trace::Phase::start("initial_api_snapshot_refresh", 1);
        self.editor_lsp_session()?.ready_document_uri(logical_uri)?;
        let session = self
            .session
            .as_mut()
            .ok_or_else(|| cstr!("Acknowledged editor snapshot owner is unavailable"))?;
        block_on(session.refresh(None))
            .map_err(|error| cstr!("Failed to refresh acknowledged editor snapshot: {error}"))?;
        phase.finish();
        Ok(())
    }

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
        let phase = crate::corsa_bridge::preparation_trace::Phase::start("editor_hover", 1);
        let result =
            self.request_with_editor_lsp_recovery(|session| session.hover(uri, line, character));
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

    pub(in crate::lsp_client) fn declaration_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        if !self.document_texts.contains_key(uri) {
            return Ok(None);
        }
        self.request_with_editor_lsp_recovery(|session| session.declaration(uri, line, character))
    }

    pub(in crate::lsp_client) fn implementation_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        if !self.document_texts.contains_key(uri) {
            return Ok(None);
        }
        self.request_with_editor_lsp_recovery(|session| {
            session.implementation(uri, line, character)
        })
    }

    pub(in crate::lsp_client) fn prepare_call_hierarchy_via_editor_lsp(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        if !self.document_texts.contains_key(uri) {
            return Ok(None);
        }
        self.request_with_editor_lsp_recovery(|session| {
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

    pub(in crate::lsp_client) fn editor_lsp_project_root(&self) -> PathBuf {
        if self.materialized_project_session {
            overlay_root_for_project(&self.project_root)
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
        result
    }

    /// Queued bridge work retires its initialized owner through normal shutdown.
    pub(crate) fn retire_queued_disk_project_state(&mut self) -> Result<(), String> {
        let phase =
            crate::corsa_bridge::preparation_trace::Phase::start("queued_editor_retirement", 1);
        self.clear_diagnostics_cache();
        let result = self.retire_editor_lsp();
        phase.finish();
        result
    }

    /// Record the exact discard caller only under the existing opt-in gate.
    pub(super) fn trace_editor_discard(&self, caller: impl FnOnce() -> String) {
        super::inlay_hint::trace_native_inlay("editorDiscard", || {
            let caller = caller();
            serde_json::json!({
                "branch": if self.project_session_attached_to_editor { "attached" } else { "independent" },
                "projectRoot": self.project_root,
                "executable": self.executable.as_str(),
                "outerPid": std::process::id(),
                "nativePidExposedBySdk": false,
                "editorPresent": self.editor_lsp.is_some(),
                "sessionPresent": self.session.is_some(),
                "documentCount": self.document_texts.len(),
                "caller": caller.as_str()
            })
        });
    }

    /// Drop the editor session immediately after an external project-shape
    /// change. Unlike [`Self::retire_editor_lsp`], this does not wait for a
    /// protocol `shutdown` response from the stale process, so file-operation
    /// notifications cannot hold the foreground LSP queue hostage.
    pub(in crate::lsp_client) fn discard_editor_lsp(&mut self) -> Result<(), String> {
        self.trace_editor_discard(|| cstr!("{:?}", std::backtrace::Backtrace::force_capture()));
        if self.project_session_attached_to_editor {
            return self.retire_shared_editor_transport(true);
        }
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

#[cfg(all(test, unix))]
mod initial_snapshot_tests {
    use super::CorsaProjectClient;
    use std::os::unix::fs::PermissionsExt;

    fn guarded_client() -> (tempfile::TempDir, CorsaProjectClient, std::path::PathBuf) {
        let root = tempfile::tempdir().unwrap();
        let executable = root.path().join("must-not-start");
        std::fs::write(&executable, "#!/bin/sh\n: > \"$0.invoked\"\nexit 1\n").unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut client = CorsaProjectClient::empty_for_test(root.path().to_path_buf());
        client.executable = executable.to_str().unwrap().into();
        let marker = root.path().join("must-not-start.invoked");
        (root, client, marker)
    }

    #[test]
    fn independent_initial_snapshot_does_not_initialize_an_editor() {
        let (_root, mut client, marker) = guarded_client();
        client
            .refresh_acknowledged_editor_snapshot("file:///unopened.ts")
            .unwrap();
        assert!(client.editor_lsp.is_none());
        assert!(client.session.is_none());
        assert!(
            !marker.exists(),
            "independent route must not execute any provider"
        );
    }

    #[test]
    fn inconsistent_initial_attachment_refuses_without_spawning_an_owner() {
        let (_root, mut client, marker) = guarded_client();
        client.project_session_attached_to_editor = true;
        assert!(
            client
                .refresh_acknowledged_editor_snapshot("file:///unopened.ts")
                .is_err()
        );
        assert!(client.editor_lsp.is_none());
        assert!(client.session.is_none());
        assert!(
            !marker.exists(),
            "inconsistent attachment must not execute any provider"
        );
    }

    #[test]
    fn queued_retirement_keeps_empty_and_inconsistent_owners_dormant() {
        for attached in [false, true] {
            let (_root, mut client, marker) = guarded_client();
            client.project_session_attached_to_editor = attached;
            client
                .diagnostics
                .insert("file:///cached.ts".into(), Vec::new());
            client.retire_queued_disk_project_state().unwrap();
            assert_eq!(client.diagnostics_cache_len(), 0);
            assert!(client.editor_lsp.is_none() && client.session.is_none());
            assert!(!client.project_session_attached_to_editor);
            assert!(!marker.exists(), "retirement must not execute a provider");
        }
    }

    #[test]
    fn queued_retirement_preserves_shutdown_failure_and_clears_stale_cache() {
        let (root, mut client, _marker) = guarded_client();
        let peer = root.path().join("controlled-peer.ts");
        let log = root.path().join("whole-packets.jsonl");
        let source = include_str!("native_shutdown/peer.ts.txt")
            .replace("LOG_PATH", &serde_json::to_string(&log).unwrap())
            .replace("REPLY_MODE", "\"error\"");
        std::fs::write(&peer, source).unwrap();
        std::fs::set_permissions(&peer, std::fs::Permissions::from_mode(0o700)).unwrap();
        client.editor_lsp = Some(
            super::EditorLspSession::spawn(peer.to_str().unwrap(), root.path(), root.path())
                .unwrap(),
        );
        client
            .diagnostics
            .insert("file:///cached.ts".into(), Vec::new());
        let actual = client.retire_queued_disk_project_state().unwrap_err();
        let data = serde_json::json!({"code":-32603,"message":"controlled shutdown refusal","data":{"phase":"shutdown"}});
        let error = corsa_lsp::CorsaError::Rpc(serde_json::from_value(data.clone()).unwrap());
        assert_eq!(
            actual,
            vize_l0::cstr!("Failed to gracefully close editor LSP process: {error}")
        );
        assert_eq!(client.diagnostics_cache_len(), 0);
        assert!(client.editor_lsp.is_none() && client.session.is_none());
        client.retire_queued_disk_project_state().unwrap();
        let packets: Vec<serde_json::Value> = std::fs::read_to_string(log)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        let begin = packets
            .iter()
            .position(|row| {
                row["direction"] == "request" && row["wholePacket"]["method"] == "shutdown"
            })
            .unwrap();
        assert_eq!(
            &packets[begin..],
            &[
                serde_json::json!({"direction":"request","wholePacket":{"jsonrpc":"2.0","id":2,"method":"shutdown"}}),
                serde_json::json!({"direction":"response","wholePacket":{"jsonrpc":"2.0","id":2,"error":data}}),
                serde_json::json!({"event":"stdin-eof"})
            ]
        );
    }
}
