//! Implementation-location editor query forwarded through the Corsa session.

use super::{CorsaBridge, parse_lsp_locations};
use crate::corsa_bridge::types::{CorsaBridgeError, LspLocation};

#[allow(clippy::disallowed_types)]
impl CorsaBridge {
    /// Get implementation locations for a symbol at a position.
    pub async fn implementation(
        &self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Vec<LspLocation>, CorsaBridgeError> {
        let _timer = self.profiler.timer("corsa_implementation");
        let uri = uri.to_owned();
        let result = self
            .with_client(move |client| {
                client
                    .implementation_raw(uri.as_str(), line, character)
                    .map_err(CorsaBridgeError::CommunicationError)
            })
            .await?;

        if let Some(timer) = _timer {
            timer.record(&self.profiler);
        }

        if let Some(value) = result {
            return parse_lsp_locations(value);
        }

        Ok(Vec::new())
    }
}
