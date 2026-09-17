//! Lossless completion transport for deferred editor documentation.

use serde_json::Value;

use super::CorsaBridge;
use crate::corsa_bridge::types::CorsaBridgeError;

impl CorsaBridge {
    /// Preserve the backend's opaque item data for a later resolve request.
    pub async fn completion_raw(
        &self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, CorsaBridgeError> {
        let timer = self.profiler.timer("corsa_completion");
        let uri = uri.to_owned();
        let result = self
            .with_client(move |client| {
                client
                    .completion_raw(&uri, line, character)
                    .map_err(CorsaBridgeError::CommunicationError)
            })
            .await;
        if let Some(timer) = timer {
            timer.record(&self.profiler);
        }
        result
    }

    /// Resolve an unchanged backend completion item on its editor session.
    pub async fn completion_resolve(
        &self,
        uri: &str,
        item: Value,
    ) -> Result<Option<Value>, CorsaBridgeError> {
        let uri = uri.to_owned();
        self.with_client(move |client| {
            client
                .completion_resolve_via_editor_lsp(&uri, item)
                .map_err(CorsaBridgeError::CommunicationError)
        })
        .await
    }
}
