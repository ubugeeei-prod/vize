//! `CorsaProjectClient` entry points that route editor requests through the
//! lazily spawned editor LSP session.

use serde_json::Value;
use vize_carton::{String, cstr};

use super::{CorsaProjectClient, EditorLspSession};

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
        self.editor_lsp_session()?.hover(uri, line, character)
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
        self.editor_lsp_session()?.completion(uri, line, character)
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
        self.editor_lsp_session()?.definition(uri, line, character)
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
        self.editor_lsp_session()?
            .references(uri, line, character, include_declaration)
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
        self.editor_lsp_session()?
            .prepare_rename(uri, line, character)
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
        self.editor_lsp_session()?
            .rename(uri, line, character, new_name)
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
    pub(in crate::lsp_client) fn retire_editor_lsp(&mut self) {
        self.editor_lsp = None;
        self.editor_lsp_documents_dirty = true;
    }
}
