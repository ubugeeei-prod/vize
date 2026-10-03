//! The process returning diagnostics attests its own configured project.

use super::EditorLspSession;
use crate::corsa_bridge::{CorsaBridgeError, NativeVueError};
use corsa::runtime::block_on;
use serde_json::Value;
use std::path::PathBuf;
use vize_l0::cstr;

struct ProjectInfoRequest;
impl lsp_types::request::Request for ProjectInfoRequest {
    type Params = Value;
    type Result = Value;
    const METHOD: &'static str = "custom/projectInfo";
}

impl EditorLspSession {
    pub(super) fn native_vue_configuration(
        &mut self,
        document_uri: &str,
    ) -> Result<PathBuf, NativeVueError> {
        let backend = |error| NativeVueError::Backend(CorsaBridgeError::CommunicationError(error));
        let uri = self.ready_document_uri(document_uri).map_err(backend)?;
        let result = block_on(
            self.client
                .request::<ProjectInfoRequest>(serde_json::json!({
                    "textDocument": {"uri": uri},
                })),
        )
        .map_err(|error| {
            backend(cstr!(
                "cannot resolve native SFC diagnosing project: {error}"
            ))
        })?;
        let path = result
            .get("configFilePath")
            .and_then(Value::as_str)
            .filter(|path| !path.is_empty())
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .ok_or(NativeVueError::UnconfiguredProjection)?;
        Ok(path)
    }
}
