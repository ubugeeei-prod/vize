//! Checker-owned inlay hints from the reusable editor session.

use super::CorsaBridge;
use crate::corsa_bridge::types::CorsaBridgeError;
use serde_json::Value;

impl CorsaBridge {
    /// Request one range of native hints without interpreting type labels.
    pub async fn inlay_hints(
        &self,
        uri: &str,
        start: (u32, u32),
        end: (u32, u32),
    ) -> Result<Option<Value>, CorsaBridgeError> {
        let uri = uri.to_owned();
        self.with_client(move |client| {
            client
                .inlay_hints_raw(&uri, start, end)
                .map_err(CorsaBridgeError::CommunicationError)
        })
        .await
    }
}
