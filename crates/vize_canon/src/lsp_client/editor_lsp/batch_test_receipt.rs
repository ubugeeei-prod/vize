//! Test-only attestation from the same client that returned Batch diagnostics.

use super::super::diagnostics::test_route::{self, Route};
use super::{CorsaProjectClient, project_identity::ProjectIdentityError};
use serde_json::{Value, json};

impl CorsaProjectClient {
    pub(crate) fn batch_test_receipt(&mut self, uri: &str) -> Value {
        let mut receipt = json!({
            "nativeBinary": self.executable, "cwd": self.cwd,
            "workspaceRoot": self.project_root,
            "startupCpu": null, "nativeProgramCount": null, "diagnosticGroups": null,
        });
        let route = test_route::take();
        if route == Some(Route::Editor) {
            let Some(editor) = self.editor_lsp.as_mut() else {
                return receipt;
            };
            receipt["mode"] = json!("editor-lsp");
            let selected = editor.diagnosing_configuration(uri);
            match selected {
                Ok(path) => receipt["selectedConfig"] = json!(path),
                Err(error) => {
                    receipt["selectedConfig"] = Value::Null;
                    receipt["configurationError"] = json!(match error {
                        ProjectIdentityError::Communication(error) => error,
                        ProjectIdentityError::Unconfigured =>
                            "native document is inferred or unconfigured".into(),
                    });
                }
            }
            match editor.configured_project_receipt(uri) {
                Ok(observed) => {
                    for key in [
                        "project",
                        "snapshotProjects",
                        "normalizedRequestedOptions",
                        "normalizedRequestedFiles",
                        "attachment",
                        "sourceFileNamesRequest",
                        "sourceFileNamesResponse",
                    ] {
                        receipt[key] = observed[key].clone();
                    }
                }
                Err(error) => receipt["observationError"] = json!(error),
            }
        } else if route == Some(Route::Api) {
            let Some(session) = &self.session else {
                return receipt;
            };
            receipt["mode"] = json!("project-api");
            receipt["selectedConfig"] = json!(session.project().config_file_name);
            receipt["project"] = json!(session.project());
            receipt["snapshotProjects"] = json!(session.snapshot().projects);
            let request =
                json!({"snapshot":session.snapshot().handle,"project":session.project().id});
            receipt["sourceFileNamesRequest"] = request.clone();
            receipt["sourceFileNamesResponse"] = match corsa::runtime::block_on(
                session
                    .client()
                    .raw_json_request("getSourceFileNames", request),
            ) {
                Ok(value) => json!({"status":"ok","wholeResponse":value}),
                Err(error) => json!({"status":"error","wholeError":vize_l0::cstr!("{error}")}),
            };
            match corsa::runtime::block_on(session.client().parse_config_file(
                corsa::api::DocumentIdentifier::from(session.project().config_file_name.as_str()),
            )) {
                Ok(parsed) => {
                    receipt["normalizedRequestedOptions"] = parsed.options;
                    receipt["normalizedRequestedFiles"] = json!(parsed.file_names);
                }
                Err(error) => receipt["observationError"] = json!(vize_l0::cstr!("{error}")),
            }
        } else {
            receipt["mode"] = json!("unknown");
            receipt["selectedConfig"] = Value::Null;
        }
        receipt
    }
}
