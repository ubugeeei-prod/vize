use super::{CorsaProjectClient, session::uri_document_identifier, utils::value_to_json};
use corsa::{CorsaError, runtime::block_on};
use lsp_types::CompletionContext;
use serde_json::Value;
use vize_s0::{String, cstr};

impl CorsaProjectClient {
    /// Get hover information at a position.
    pub fn hover(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<crate::LspHover>, String> {
        let value = match self.hover_raw(uri, line, character)? {
            Some(value) => value,
            None => return Ok(None),
        };

        serde_json::from_value(value)
            .map(Some)
            .map_err(|err| cstr!("Failed to parse hover response: {err}"))
    }

    pub(crate) fn hover_raw(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        // The project-session API rejects hover as unsupported on every pinned
        // runtime (corsa-bind#409), either up front through
        // `describeCapabilities` or at request time. Both roads lead to the
        // editor LSP transport, which the same runtime does serve.
        let Some(position) = self.api_position(uri, line, character, self.supports_hover_api())?
        else {
            return self.hover_via_editor_lsp(uri, line, character);
        };

        let document_uri = self.session_document_uri(uri);
        match block_on(
            self.project_session()?
                .get_hover_at_position(uri_document_identifier(document_uri.as_str()), position),
        ) {
            Ok(response) => response.map(value_to_json).transpose(),
            Err(CorsaError::Unsupported(_)) => self.hover_via_editor_lsp(uri, line, character),
            Err(error) => Err(cstr!("Failed to request hover: {error}")),
        }
    }

    pub(crate) fn definition_raw(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        let Some(position) =
            self.api_position(uri, line, character, self.supports_definition_api())?
        else {
            return self.definition_via_editor_lsp(uri, line, character);
        };

        let document_uri = self.session_document_uri(uri);
        match block_on(
            self.project_session()?.get_definition_at_position(
                uri_document_identifier(document_uri.as_str()),
                position,
            ),
        ) {
            Ok(response) => self.serialize_with_remapped_uris(response),
            Err(CorsaError::Unsupported(_)) => self.definition_via_editor_lsp(uri, line, character),
            Err(error) => Err(cstr!("Failed to request definition: {error}")),
        }
    }

    pub(crate) fn type_definition_raw(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        self.type_definition_via_editor_lsp(uri, line, character)
    }

    pub(crate) fn declaration_raw(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        self.declaration_via_editor_lsp(uri, line, character)
    }

    pub(crate) fn implementation_raw(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        self.implementation_via_editor_lsp(uri, line, character)
    }

    pub(crate) fn prepare_call_hierarchy_raw(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        self.prepare_call_hierarchy_via_editor_lsp(uri, line, character)
    }

    pub(crate) fn call_hierarchy_incoming_calls_raw(
        &mut self,
        item: Value,
    ) -> Result<Option<Value>, String> {
        self.call_hierarchy_incoming_calls_via_editor_lsp(item)
    }

    pub(crate) fn call_hierarchy_outgoing_calls_raw(
        &mut self,
        item: Value,
    ) -> Result<Option<Value>, String> {
        self.call_hierarchy_outgoing_calls_via_editor_lsp(item)
    }

    pub(crate) fn references_raw(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
        include_declaration: bool,
    ) -> Result<Option<Value>, String> {
        let Some(position) =
            self.api_position(uri, line, character, self.supports_references_api())?
        else {
            return self.references_via_editor_lsp(uri, line, character, include_declaration);
        };

        let document_uri = self.session_document_uri(uri);
        match block_on(
            self.project_session()?.get_references_at_position(
                uri_document_identifier(document_uri.as_str()),
                position,
            ),
        ) {
            Ok(response) => self.serialize_with_remapped_uris(Some(response)),
            Err(CorsaError::Unsupported(_)) => {
                self.references_via_editor_lsp(uri, line, character, include_declaration)
            }
            Err(error) => Err(cstr!("Failed to request references: {error}")),
        }
    }

    pub(crate) fn prepare_rename_raw(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        self.prepare_rename_via_editor_lsp(uri, line, character)
    }

    pub(crate) fn rename_raw(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
        new_name: &str,
    ) -> Result<Option<Value>, String> {
        let Some(position) = self.api_position(uri, line, character, self.supports_rename_api())?
        else {
            return self.rename_via_editor_lsp(uri, line, character, new_name);
        };

        let document_uri = self.session_document_uri(uri);
        match block_on(self.project_session()?.get_rename_at_position(
            uri_document_identifier(document_uri.as_str()),
            position,
            new_name,
        )) {
            Ok(response) => self.serialize_with_remapped_uris(response),
            Err(CorsaError::Unsupported(_)) => {
                self.rename_via_editor_lsp(uri, line, character, new_name)
            }
            Err(error) => Err(cstr!("Failed to request rename: {error}")),
        }
    }

    pub(crate) fn will_rename_files_raw(
        &mut self,
        renames: &[(&str, &str)],
    ) -> Result<Option<Value>, String> {
        self.will_rename_files_via_editor_lsp(renames)
    }

    pub(crate) fn completion_raw(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        // Like hover, the project-session API rejects completion as
        // unsupported on every pinned runtime (corsa-bind#409); the editor LSP
        // transport on the same runtime does serve it (#3911).
        let Some(position) =
            self.api_position(uri, line, character, self.supports_completion_api())?
        else {
            return self.completion_via_editor_lsp(uri, line, character);
        };

        let context = CompletionContext {
            trigger_kind: lsp_types::CompletionTriggerKind::INVOKED,
            trigger_character: None,
        };
        let document_uri = self.session_document_uri(uri);
        match block_on(self.project_session()?.get_completion_at_position(
            uri_document_identifier(document_uri.as_str()),
            position,
            Some(context),
        )) {
            Ok(response) => self.serialize_with_remapped_uris(response),
            Err(CorsaError::Unsupported(_)) => self.completion_via_editor_lsp(uri, line, character),
            Err(error) => Err(cstr!("Failed to request completion: {error}")),
        }
    }

    pub(crate) fn signature_help_raw(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
        context: Option<Value>,
    ) -> Result<Option<Value>, String> {
        self.signature_help_via_editor_lsp(uri, line, character, context)
    }

    fn api_position(
        &self,
        uri: &str,
        line: u32,
        character: u32,
        supported: bool,
    ) -> Result<Option<u32>, String> {
        if !supported || !self.can_use_api_for_uri(uri) {
            return Ok(None);
        }

        self.utf16_offset_for(uri, line, character)
            .map(Some)
            .ok_or_else(|| cstr!("Failed to resolve UTF-16 offset for `{uri}`"))
    }

    fn serialize_with_remapped_uris<T>(&self, response: Option<T>) -> Result<Option<Value>, String>
    where
        T: serde::Serialize,
    {
        let Some(response) = response else {
            return Ok(None);
        };

        let mut value = value_to_json(response)?;
        self.remap_result_uris(&mut value);
        Ok(Some(value))
    }
}
