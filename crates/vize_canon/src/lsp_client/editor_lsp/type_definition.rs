use corsa::runtime::block_on;
use serde_json::Value;
use vize_s0::{String, cstr};

use super::{EditorLspSession, requests::RawTypeDefinitionRequest};

impl EditorLspSession {
    pub(super) fn type_definition(
        &mut self,
        document_uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<Value>, String> {
        let uri = self.ready_document_uri(document_uri)?;
        block_on(
            self.client
                .request::<RawTypeDefinitionRequest>(serde_json::json!({
                    "textDocument": { "uri": uri },
                    "position": { "line": line, "character": character },
                })),
        )
        .map_err(|error| cstr!("Failed to request editor LSP type definition: {error}"))
    }
}
