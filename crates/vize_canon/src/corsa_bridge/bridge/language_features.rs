//! Positional editor queries the bridge forwards to the Corsa session.

use serde_json::Value;
use vize_carton::String;

use super::{CorsaBridge, parse_json_value};
use crate::corsa_bridge::types::{
    CorsaBridgeError, LspCompletionItem, LspCompletionResponse, LspDefinitionResponse, LspHover,
    LspLocation, LspSignatureHelp,
};

#[allow(clippy::disallowed_types)]
impl CorsaBridge {
    /// Get hover information at a position.
    pub async fn hover(
        &self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<LspHover>, CorsaBridgeError> {
        let _timer = self.profiler.timer("corsa_hover");
        let uri = uri.to_owned();
        let result = self
            .with_client(move |client| {
                client
                    .hover_raw(uri.as_str(), line, character)
                    .map_err(CorsaBridgeError::CommunicationError)
            })
            .await?;

        if let Some(timer) = _timer {
            timer.record(&self.profiler);
        }

        result.map(parse_json_value::<LspHover>).transpose()
    }

    /// Get definition location for a symbol at a position.
    pub async fn definition(
        &self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Vec<LspLocation>, CorsaBridgeError> {
        let _timer = self.profiler.timer("corsa_definition");
        let uri = uri.to_owned();
        let result = self
            .with_client(move |client| {
                client
                    .definition_raw(uri.as_str(), line, character)
                    .map_err(CorsaBridgeError::CommunicationError)
            })
            .await?;

        if let Some(timer) = _timer {
            timer.record(&self.profiler);
        }

        if let Some(value) = result {
            return Ok(parse_json_value::<LspDefinitionResponse>(value)?.into_locations());
        }

        Ok(Vec::new())
    }

    /// Resolve several definition queries in one worker job.
    ///
    /// Corsa's project client is synchronous, so submitting every query as a
    /// separate bridge job would apply `timeout_ms` once per position. This
    /// batch keeps the complete fan-out under one bridge deadline while
    /// preserving the result for every query when the job completes.
    pub async fn definition_batch(
        &self,
        positions: &[(&str, u32, u32)],
    ) -> Result<Vec<Vec<LspLocation>>, CorsaBridgeError> {
        let timer = self.profiler.timer("corsa_definition_batch");
        let positions = positions
            .iter()
            .map(|(uri, line, character)| (String::from(*uri), *line, *character))
            .collect::<Vec<_>>();
        let results = self
            .with_client(move |client| {
                positions
                    .iter()
                    .map(|(uri, line, character)| {
                        client
                            .definition_raw(uri.as_str(), *line, *character)
                            .map_err(CorsaBridgeError::CommunicationError)
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .await?;
        if let Some(timer) = timer {
            timer.record(&self.profiler);
        }

        results
            .into_iter()
            .map(|result| match result {
                Some(value) => {
                    Ok(parse_json_value::<LspDefinitionResponse>(value)?.into_locations())
                }
                None => Ok(Vec::new()),
            })
            .collect()
    }

    /// Get type definition locations for a symbol at a position.
    pub async fn type_definition(
        &self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Vec<LspLocation>, CorsaBridgeError> {
        let _timer = self.profiler.timer("corsa_type_definition");
        let uri = uri.to_owned();
        let result = self
            .with_client(move |client| {
                client
                    .type_definition_raw(uri.as_str(), line, character)
                    .map_err(CorsaBridgeError::CommunicationError)
            })
            .await?;

        if let Some(timer) = _timer {
            timer.record(&self.profiler);
        }

        if let Some(value) = result {
            return Ok(parse_json_value::<LspDefinitionResponse>(value)?.into_locations());
        }

        Ok(Vec::new())
    }

    /// Get references for a symbol at a position.
    pub async fn references(
        &self,
        uri: &str,
        line: u32,
        character: u32,
        include_declaration: bool,
    ) -> Result<Vec<LspLocation>, CorsaBridgeError> {
        let uri = uri.to_owned();
        let result = self
            .with_client(move |client| {
                client
                    .references_raw(uri.as_str(), line, character, include_declaration)
                    .map_err(CorsaBridgeError::CommunicationError)
            })
            .await?;

        if let Some(value) = result {
            return parse_json_value(value);
        }

        Ok(Vec::new())
    }

    /// Resolve several reference queries in one worker job and one aggregate
    /// bridge deadline.
    pub async fn references_batch(
        &self,
        positions: &[(&str, u32, u32)],
        include_declaration: bool,
    ) -> Result<Vec<Vec<LspLocation>>, CorsaBridgeError> {
        let timer = self.profiler.timer("corsa_references_batch");
        let positions = positions
            .iter()
            .map(|(uri, line, character)| (String::from(*uri), *line, *character))
            .collect::<Vec<_>>();
        let results = self
            .with_client(move |client| {
                positions
                    .iter()
                    .map(|(uri, line, character)| {
                        client
                            .references_raw(uri.as_str(), *line, *character, include_declaration)
                            .map_err(CorsaBridgeError::CommunicationError)
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .await?;
        if let Some(timer) = timer {
            timer.record(&self.profiler);
        }

        results
            .into_iter()
            .map(|result| match result {
                Some(value) => parse_json_value(value),
                None => Ok(Vec::new()),
            })
            .collect()
    }

    /// Check whether rename is valid at a position.
    pub async fn prepare_rename(
        &self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, CorsaBridgeError> {
        let uri = uri.to_owned();
        self.with_client(move |client| {
            client
                .prepare_rename_raw(uri.as_str(), line, character)
                .map_err(CorsaBridgeError::CommunicationError)
        })
        .await
    }

    /// Rename a symbol at a position.
    pub async fn rename(
        &self,
        uri: &str,
        line: u32,
        character: u32,
        new_name: &str,
    ) -> Result<Option<Value>, CorsaBridgeError> {
        let uri = uri.to_owned();
        let new_name = new_name.to_owned();
        self.with_client(move |client| {
            client
                .rename_raw(uri.as_str(), line, character, new_name.as_str())
                .map_err(CorsaBridgeError::CommunicationError)
        })
        .await
    }

    /// Resolve several rename queries in one worker job and one aggregate
    /// bridge deadline.
    pub async fn rename_batch(
        &self,
        positions: &[(&str, u32, u32)],
        new_name: &str,
    ) -> Result<Vec<Option<Value>>, CorsaBridgeError> {
        let timer = self.profiler.timer("corsa_rename_batch");
        let positions = positions
            .iter()
            .map(|(uri, line, character)| (String::from(*uri), *line, *character))
            .collect::<Vec<_>>();
        let new_name = String::from(new_name);
        let results = self
            .with_client(move |client| {
                positions
                    .iter()
                    .map(|(uri, line, character)| {
                        client
                            .rename_raw(uri.as_str(), *line, *character, new_name.as_str())
                            .map_err(CorsaBridgeError::CommunicationError)
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .await?;
        if let Some(timer) = timer {
            timer.record(&self.profiler);
        }
        Ok(results)
    }

    /// Request import path updates before files are renamed.
    pub async fn will_rename_files(
        &self,
        renames: &[(&str, &str)],
    ) -> Result<Option<Value>, CorsaBridgeError> {
        let renames: Vec<(String, String)> = renames
            .iter()
            .map(|(old_uri, new_uri)| ((*old_uri).into(), (*new_uri).into()))
            .collect();

        self.with_client(move |client| {
            let renames_ref: Vec<(&str, &str)> = renames
                .iter()
                .map(|(old_uri, new_uri)| (old_uri.as_str(), new_uri.as_str()))
                .collect();
            client
                .will_rename_files_raw(&renames_ref)
                .map_err(CorsaBridgeError::CommunicationError)
        })
        .await
    }

    /// Get completion items at a position.
    pub async fn completion(
        &self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Vec<LspCompletionItem>, CorsaBridgeError> {
        let _timer = self.profiler.timer("corsa_completion");
        let uri = uri.to_owned();
        let result = self
            .with_client(move |client| {
                client
                    .completion_raw(uri.as_str(), line, character)
                    .map_err(CorsaBridgeError::CommunicationError)
            })
            .await?;

        if let Some(timer) = _timer {
            timer.record(&self.profiler);
        }

        if let Some(value) = result {
            return Ok(parse_json_value::<LspCompletionResponse>(value)?.items());
        }

        Ok(Vec::new())
    }

    /// Get signature help at a position.
    pub async fn signature_help(
        &self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<LspSignatureHelp>, CorsaBridgeError> {
        self.signature_help_with_context(uri, line, character, None)
            .await
    }

    /// Get signature help while preserving the client's LSP request context.
    pub async fn signature_help_with_context(
        &self,
        uri: &str,
        line: u32,
        character: u32,
        context: Option<Value>,
    ) -> Result<Option<LspSignatureHelp>, CorsaBridgeError> {
        let _timer = self.profiler.timer("corsa_signature_help");
        let uri = uri.to_owned();
        let result = self
            .with_client(move |client| {
                client
                    .signature_help_raw(uri.as_str(), line, character, context)
                    .map_err(CorsaBridgeError::CommunicationError)
            })
            .await?;

        if let Some(timer) = _timer {
            timer.record(&self.profiler);
        }

        result.map(parse_json_value::<LspSignatureHelp>).transpose()
    }
}
