use super::{CorsaProjectClient, session::uri_document_identifier, utils::value_to_json};
use corsa::{CorsaError, runtime::block_on};
use lsp_types::CompletionContext;
use serde_json::Value;
use vize_carton::{String, cstr};

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
            self.session
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
            return Ok(None);
        };

        let document_uri = self.session_document_uri(uri);
        let response =
            block_on(self.session.get_definition_at_position(
                uri_document_identifier(document_uri.as_str()),
                position,
            ))
            .map_err(|error| cstr!("Failed to request definition: {error}"))?;
        self.serialize_with_remapped_uris(response)
    }

    pub(crate) fn references_raw(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
        _include_declaration: bool,
    ) -> Result<Option<Value>, String> {
        let Some(position) =
            self.api_position(uri, line, character, self.supports_references_api())?
        else {
            return Ok(None);
        };

        let document_uri = self.session_document_uri(uri);
        let response =
            block_on(self.session.get_references_at_position(
                uri_document_identifier(document_uri.as_str()),
                position,
            ))
            .map_err(|error| cstr!("Failed to request references: {error}"))?;
        self.serialize_with_remapped_uris(Some(response))
    }

    pub(crate) fn prepare_rename_raw(
        &mut self,
        _uri: &str,
        _line: u32,
        _character: u32,
    ) -> Result<Option<Value>, String> {
        Ok(None)
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
            return Ok(None);
        };

        let document_uri = self.session_document_uri(uri);
        let response = block_on(self.session.get_rename_at_position(
            uri_document_identifier(document_uri.as_str()),
            position,
            new_name,
        ))
        .map_err(|error| cstr!("Failed to request rename: {error}"))?;
        self.serialize_with_remapped_uris(response)
    }

    pub(crate) fn will_rename_files_raw(
        &mut self,
        _renames: &[(&str, &str)],
    ) -> Result<Option<Value>, String> {
        Ok(None)
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
        match block_on(self.session.get_completion_at_position(
            uri_document_identifier(document_uri.as_str()),
            position,
            Some(context),
        )) {
            Ok(response) => self.serialize_with_remapped_uris(response),
            Err(CorsaError::Unsupported(_)) => self.completion_via_editor_lsp(uri, line, character),
            Err(error) => Err(cstr!("Failed to request completion: {error}")),
        }
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
