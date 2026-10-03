//! The process returning diagnostics attests its own configured project.

use super::EditorLspSession;
use corsa::runtime::block_on;
use serde_json::Value;
use std::path::PathBuf;
use vize_l0::{String, cstr};

pub(super) enum ProjectIdentityError {
    Communication(String),
    Unconfigured,
}

struct ProjectInfoRequest;
impl lsp_types::request::Request for ProjectInfoRequest {
    type Params = Value;
    type Result = Value;
    const METHOD: &'static str = "custom/projectInfo";
}

impl EditorLspSession {
    pub(super) fn diagnosing_configuration(
        &mut self,
        document_uri: &str,
    ) -> Result<PathBuf, ProjectIdentityError> {
        let uri = self
            .ready_document_uri(document_uri)
            .map_err(ProjectIdentityError::Communication)?;
        let result = block_on(
            self.client
                .request::<ProjectInfoRequest>(serde_json::json!({
                    "textDocument": {"uri": uri},
                })),
        )
        .map_err(|error| {
            ProjectIdentityError::Communication(cstr!("cannot resolve diagnosing project: {error}"))
        })?;
        configured_path(&result)
    }
}

fn configured_path(result: &Value) -> Result<PathBuf, ProjectIdentityError> {
    result
        .get("configFilePath")
        .and_then(Value::as_str)
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .ok_or(ProjectIdentityError::Unconfigured)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_inferred_and_relative_project_receipts_are_refused() {
        for result in [
            Value::Null,
            serde_json::json!({}),
            serde_json::json!({"configFilePath": null}),
            serde_json::json!({"configFilePath": 1}),
            serde_json::json!({"configFilePath": ""}),
            serde_json::json!({"configFilePath": "inferredProject1*"}),
            serde_json::json!({"configFilePath": "tsconfig.json"}),
        ] {
            assert!(matches!(
                configured_path(&result),
                Err(ProjectIdentityError::Unconfigured)
            ));
        }
    }
}
