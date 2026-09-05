use serde_json::Value;

use super::CorsaBridge;
use crate::corsa_bridge::types::CorsaBridgeError;

impl CorsaBridge {
    /// Prepare call-hierarchy items for a symbol at a position.
    ///
    /// The editor LSP transport owns this protocol shape; callers keep the raw
    /// payload so they can map the item URI and both ranges with their local
    /// authored-source model.
    pub async fn prepare_call_hierarchy(
        &self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, CorsaBridgeError> {
        let uri = uri.to_owned();
        self.with_client(move |client| {
            client
                .prepare_call_hierarchy_raw(uri.as_str(), line, character)
                .map_err(CorsaBridgeError::CommunicationError)
        })
        .await
    }
}
