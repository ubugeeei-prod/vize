//! Checker-owned component relations on the existing editor document.

use super::CorsaBridge;
use crate::corsa_bridge::types::CorsaBridgeError;

impl CorsaBridge {
    /// Compare recorded bindings to the actual Vue module's Component type.
    pub async fn component_types(
        &self,
        uri: &str,
        source: &str,
        vue_module: u32,
        positions: &[u32],
    ) -> Result<Option<Vec<Option<bool>>>, CorsaBridgeError> {
        let uri = uri.to_owned();
        let source = source.to_owned();
        let positions = positions.to_vec();
        self.with_client(move |client| {
            client
                .component_types_raw(&uri, &source, vue_module, &positions)
                .map_err(CorsaBridgeError::CommunicationError)
        })
        .await
    }
}
