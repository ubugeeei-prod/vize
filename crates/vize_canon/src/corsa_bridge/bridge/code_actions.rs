//! Checker quick fixes on the shared canonical editor project.

use serde_json::Value;

use super::CorsaBridge;
use crate::corsa_bridge::types::{CorsaBridgeError, LspRange};

impl CorsaBridge {
    /// Request quick fixes using diagnostics from the current backend snapshot.
    pub async fn code_actions(
        &self,
        uri: &str,
        range: LspRange,
    ) -> Result<Option<Value>, CorsaBridgeError> {
        let uri = uri.to_owned();
        self.with_client(move |client| {
            client
                .code_actions_via_editor_lsp(&uri, &range)
                .map_err(CorsaBridgeError::CommunicationError)
        })
        .await
    }
}
