//! Declaration-location editor query forwarded through the Corsa session.

use super::{CorsaBridge, parse_json_value};
use crate::corsa_bridge::types::{CorsaBridgeError, LspDefinitionResponse, LspLocation};

#[allow(clippy::disallowed_types)]
impl CorsaBridge {
    /// Get declaration locations for a symbol at a position.
    pub async fn declaration(
        &self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Vec<LspLocation>, CorsaBridgeError> {
        let _timer = self.profiler.timer("corsa_declaration");
        let uri = uri.to_owned();
        let result = self
            .with_client(move |client| {
                client
                    .declaration_raw(uri.as_str(), line, character)
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
}
