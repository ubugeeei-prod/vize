//! The process returning diagnostics attests its own configured project.

use super::EditorLspSession;
use corsa::runtime::block_on;
use serde_json::Value;
use std::path::{Path, PathBuf};
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
        let result = self.request_project_info(serde_json::json!(uri))?;
        configured_path(&result)
    }

    pub(super) fn apply_startup_configuration(&self, config: &Path) -> Result<(), String> {
        let api = self
            .configured_api
            .as_ref()
            .ok_or_else(|| cstr!("Startup configuration has no retained API attachment"))?;
        // This native config read does not create or update a snapshot. Select
        // a real member before the first API snapshot can consume pending prefs.
        let parsed = block_on(api.client.parse_config_file(
            super::super::session::uri_document_identifier(&crate::file_uri::path_to_file_uri(
                config,
            )),
        ))
        .map_err(|error| cstr!("Cannot read native startup configuration: {error}"))?;
        let member = startup_member(&parsed.file_names)?;
        let uri = crate::file_uri::path_to_file_uri(member);
        let result = self.request_project_info(serde_json::json!(uri));
        super::inlay_hint::trace_native_inlay("startupConfigurationSnapshot", || {
            serde_json::json!({
                "requestedConfig":config,"memberUri":uri,
                "nativeConfig":parsed,"apiAttachment":api.session,
                "result":match &result {
                    Ok(value)=>serde_json::json!({"ok":true,"response":value}),
                    Err(ProjectIdentityError::Communication(error))=>serde_json::json!({"ok":false,"error":error.as_str()}),
                    Err(ProjectIdentityError::Unconfigured)=>serde_json::json!({"ok":false,"unconfigured":true})
                }
            })
        });
        let observed = result.and_then(|value| configured_path(&value));
        match observed {
            Ok(path) if path == config => Ok(()),
            Ok(_) | Err(ProjectIdentityError::Unconfigured) => Err(cstr!(
                "Startup configuration snapshot belongs to another project"
            )),
            Err(ProjectIdentityError::Communication(error)) => Err(error),
        }
    }

    fn request_project_info(&self, uri: Value) -> Result<Value, ProjectIdentityError> {
        block_on(
            self.client
                .request::<ProjectInfoRequest>(serde_json::json!({
                    "textDocument": {"uri": uri},
                })),
        )
        .map_err(|error| {
            ProjectIdentityError::Communication(cstr!("cannot resolve diagnosing project: {error}"))
        })
    }
}

fn startup_member(files: &[std::string::String]) -> Result<&Path, String> {
    files
        .iter()
        .map(Path::new)
        .find(|path| path.is_absolute() && path.is_file())
        .ok_or_else(|| cstr!("Native startup configuration has no existing absolute member"))
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
    fn startup_member_uses_only_an_existing_native_config_member() {
        let root = tempfile::tempdir().unwrap();
        let member = root.path().join("source.ts");
        std::fs::write(&member, "export {};").unwrap();
        for files in [
            vec![],
            vec!["source.ts".into()],
            vec![root.path().join("absent.ts").to_string_lossy().into_owned()],
        ] {
            assert!(startup_member(&files).is_err());
        }
        let files = vec![
            root.path().join("absent.ts").to_string_lossy().into_owned(),
            member.to_string_lossy().into_owned(),
        ];
        assert_eq!(startup_member(&files).unwrap(), member);
        assert_eq!(std::fs::read(&member).unwrap(), b"export {};");
    }

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
