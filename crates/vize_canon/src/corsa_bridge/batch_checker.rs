//! Batched convenience API over one shared Corsa bridge.

#![allow(clippy::disallowed_types)] // The bridge is intentionally shared across batch requests.

use std::sync::Arc;

use vize_carton::String;

use super::{CorsaBridge, CorsaBridgeError, TypeCheckResult};

#[allow(clippy::disallowed_types)]
pub struct BatchTypeChecker {
    bridge: Arc<CorsaBridge>,
    batch_size: usize,
}

#[allow(clippy::disallowed_types)]
impl BatchTypeChecker {
    pub fn new(bridge: Arc<CorsaBridge>) -> Self {
        Self {
            bridge,
            batch_size: 10,
        }
    }

    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    pub async fn check_batch(
        &self,
        documents: &[(String, String)],
    ) -> Vec<Result<TypeCheckResult, CorsaBridgeError>> {
        let timer = self.bridge.profiler().timer("batch_type_check");
        let mut results = Vec::with_capacity(documents.len());
        for chunk in documents.chunks(self.batch_size) {
            let mut uris = Vec::with_capacity(chunk.len());
            for (name, content) in chunk {
                match self.bridge.open_virtual_document(name, content).await {
                    Ok(uri) => uris.push(Some(uri)),
                    Err(error) => {
                        results.push(Err(error));
                        uris.push(None);
                    }
                }
            }
            for uri in uris.into_iter().flatten() {
                match self.bridge.get_diagnostics(&uri).await {
                    Ok(diagnostics) => results.push(Ok(TypeCheckResult {
                        diagnostics,
                        source_map: None,
                    })),
                    Err(error) => results.push(Err(error)),
                }
            }
        }
        if let Some(timer) = timer {
            timer.record(self.bridge.profiler());
        }
        results
    }
}
