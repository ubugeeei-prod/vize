//! Test-only attestation from the same client that returned Batch diagnostics.

use super::super::diagnostics::test_route::{self, Route};
use super::snapshot_source::{SnapshotSourceOwner, SourceTextOutcome};
use super::{CorsaProjectClient, EditorLspSession, project_identity::ProjectIdentityError};
use corsa::runtime::block_on;
use serde_json::{Value, json};
use std::{fs, path::Path};

impl CorsaProjectClient {
    /// Read the complete leaf declaration on the attachment that diagnosed this batch.
    pub(crate) fn batch_test_leaf_source(&mut self, uri: &str, authority: &Value) -> Value {
        self.batch_test_source(uri, authority, "loadedLeafSources")
    }

    /// Read the original authored TS source in its own actual diagnosing project.
    pub(crate) fn batch_test_authored_source(&mut self, uri: &str, authority: &Value) -> Value {
        self.batch_test_source(uri, authority, "loadedAuthoredSources")
    }

    fn batch_test_source(&mut self, uri: &str, authority: &Value, source_key: &str) -> Value {
        assert_eq!(authority["mode"], "editor-lsp");
        assert_eq!(authority["nativeBinary"], json!(self.executable));
        let editor = self.editor_lsp.as_mut().unwrap();
        let configuration = materialized_configuration_receipt(editor, uri);
        let Some(config) = configuration["selectedConfig"].as_str() else {
            return json!({"qualified":false,"materializedConfigurationObservation":configuration});
        };
        let config = Path::new(config);
        let attachment = editor.configured_api.as_ref().unwrap();
        let mut owner = match SnapshotSourceOwner::create(&attachment.client) {
            Ok(owner) => owner,
            Err(error) => {
                return json!({"qualified":false,"attachment":attachment.session,
                "materializedConfigurationObservation":configuration,
                "wholeSnapshotError":vize_l0::cstr!("{error}")});
            }
        };
        let singleton = match owner.project(config) {
            Ok(Ok(project)) => json!({"status":"admitted","wholeDescriptor":project.descriptor(),
                "wholeSourceNames":project.source_names()}),
            Ok(Err(reason)) => {
                json!({"status":"refused","wholeRefusal":vize_l0::cstr!("{reason:?}")})
            }
            Err(error) => json!({"status":"error","wholeError":vize_l0::cstr!("{error}")}),
        };
        let mut observation = json!({
            "scope":"test-only unique whole diagnosing descriptor on retained attachment; no production singleton credit",
            "attachment":attachment.session,"snapshotHandle":owner.handle(),
            "sourceNamesRequest":{"snapshot":owner.handle(),"project":authority["project"]["id"]},
            "wholeSnapshotProjects":owner.whole_batch_snapshot_projects_for_test(),
            "originalSingletonAdmission":singleton,"materializedConfigurationObservation":configuration,
            "sameDiagnosingConfiguration":json!(config)==authority["selectedConfig"],
            "sameDiagnosingAttachment":json!(attachment.session)==authority["attachment"],
            "descriptorRefusals":owner.batch_project_identity_refusals_for_test(config,&authority["project"]),
            "qualified":false,
        });
        match owner.batch_project_for_test(config, &authority["project"]) {
            Ok(Ok(project)) => {
                let same_project = json!(project.descriptor()) == authority["project"];
                let same_sources =
                    json!(project.source_names()) == authority["sourceNamesResponse"];
                let sources: Vec<_> = project.source_names().iter().filter(|name| {
                    if source_key == "loadedLeafSources" {
                        Path::new(name.as_str()).ends_with("node_modules/leaf-types/index.d.ts")
                    } else {
                        crate::file_uri::path_to_file_uri(Path::new(name.as_str())).as_str() == uri
                    }
                }).map(|name| {
                    let source_uri = crate::file_uri::path_to_file_uri(Path::new(name.as_str()));
                    match project.read(source_uri.as_str()) {
                        Ok(SourceTextOutcome::Complete(text)) => {
                            let physical = fs::canonicalize(text.file_name()).unwrap();
                            let bytes = fs::read(&physical).unwrap();
                            json!({"nativeSourceName":name,"nativeFileName":text.file_name(),
                                "nativePath":text.path(),"nativeUri":text.uri(),"physical":physical,
                                "completeNativeText":text.text().as_bytes(),
                                "wholeEncodedPayload":text.encoded_bytes(),"completePhysicalBytes":bytes,
                                "nativeMatchesPhysical":bytes == text.text().as_bytes()})
                        }
                        Ok(SourceTextOutcome::Refused { reason, encoded }) => json!({
                            "nativeSourceName":name,"nativeMatchesPhysical":false,
                            "nativeReadRefusal":vize_l0::cstr!("{reason:?}"),
                            "wholeEncodedRefusal":encoded.as_ref().map(|value|value.as_bytes()),
                        }),
                        Err(error) => json!({"nativeSourceName":name,"nativeMatchesPhysical":false,
                            "wholeNativeReadError":vize_l0::cstr!("{error}")}),
                    }
                }).collect::<Vec<_>>();
                observation["project"] = json!(project.descriptor());
                observation["wholeSourceNames"] = json!(project.source_names());
                observation["sameDiagnosingProject"] = json!(same_project);
                observation["sameDiagnosingSourceNames"] = json!(same_sources);
                observation["qualified"] = json!(
                    same_project
                        && same_sources
                        && observation["sameDiagnosingConfiguration"] == true
                        && observation["sameDiagnosingAttachment"] == true
                        && sources.len() == 1
                        && sources
                            .iter()
                            .all(|source| source["nativeMatchesPhysical"] == true)
                );
                observation[source_key] = json!(sources);
            }
            Ok(Err(reason)) => {
                observation["wholeProjectRefusal"] = json!(vize_l0::cstr!("{reason:?}"))
            }
            Err(error) => observation["wholeProjectError"] = json!(vize_l0::cstr!("{error}")),
        }
        if let Err(error) = owner.release() {
            observation["wholeReleaseError"] = json!(vize_l0::cstr!("{error}"));
            observation["qualified"] = json!(false);
        }
        observation
    }
}

impl CorsaProjectClient {
    pub(crate) fn batch_test_process_observer(
        &self,
    ) -> Option<corsa_lsp::process::ProcessObserver> {
        self.editor_lsp
            .as_ref()
            .map(|editor| editor.process_observer.clone())
    }

    pub(crate) fn batch_test_receipt(&mut self, uri: &str) -> Value {
        self.batch_test_receipts(&[uri]).pop().unwrap()
    }

    pub(crate) fn batch_test_receipts(&mut self, uris: &[&str]) -> Vec<Value> {
        let route = test_route::take();
        uris.iter()
            .map(|uri| self.batch_receipt_for_route(uri, route))
            .collect()
    }

    fn batch_receipt_for_route(&mut self, uri: &str, route: Option<Route>) -> Value {
        let mut receipt = json!({
            "diagnosedUri":uri, "nativeBinary": self.executable, "cwd": self.cwd,
            "workspaceRoot": self.project_root,
            "startupCpu": null, "nativeProgramCount": null, "diagnosticGroups": null,
        });
        if route == Some(Route::Editor) {
            let Some(editor) = self.editor_lsp.as_mut() else {
                return receipt;
            };
            receipt["mode"] = json!("editor-lsp");
            receipt["nativeInitializeResponse"] = editor.native_initialize_response.clone();
            receipt["nativeShutdownMode"] = json!(if editor.shutdown_to_eof {
                "authenticated-native-7.0.2-shutdown-eof"
            } else {
                "standard-sdk-shutdown-exit"
            });
            let processes = editor.process_observer.receipts();
            receipt["processPid"] = json!(processes.first().map(|r| r.pid));
            // Preserve the original overlay-only gate, including its refusal.
            // A Batch request can have already diagnosed this materialized URI
            // in an earlier chunk, outside the last live overlay map.
            receipt["originalOverlayConfigurationObservation"] =
                match editor.diagnosing_configuration(uri) {
                    Ok(path) => json!({"status":"ok","selectedConfig":path}),
                    Err(error) => json!({"status":"refused","wholeError":match error {
                        ProjectIdentityError::Communication(error) => error,
                        ProjectIdentityError::Unconfigured =>
                            "native document is inferred or unconfigured".into(),
                    }}),
                };
            let observed = materialized_configuration_receipt(editor, uri);
            receipt["selectedConfig"] = observed["selectedConfig"].clone();
            if observed["status"] != "ok" {
                receipt["configurationError"] = observed["wholeError"].clone();
            }
            receipt["materializedConfigurationObservation"] = observed;
            match editor.configured_project_receipt(uri) {
                Ok(observed) => {
                    for key in [
                        "project",
                        "snapshotProjects",
                        "normalizedRequestedOptions",
                        "normalizedRequestedFiles",
                        "attachment",
                        "sourceNamesRequest",
                        "sourceNamesResponse",
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

/// Query the existing diagnosing LSP, without opening or replacing an overlay.
pub(super) fn materialized_configuration_receipt(
    editor: &mut EditorLspSession,
    uri: &str,
) -> Value {
    struct ProjectInfoRequest;
    impl lsp_types::request::Request for ProjectInfoRequest {
        type Params = Value;
        type Result = Value;
        const METHOD: &'static str = "custom/projectInfo";
    }

    let request = json!({"textDocument":{"uri":uri}});
    let mut receipt = json!({
        "method":"custom/projectInfo","wholeRequest":request,
        "scope":"same already-diagnosing LSP, acknowledged materialized Batch URI; no overlay or owner replacement",
        "status":"refused","selectedConfig":null,
    });
    if let Err(error) = editor.ready_workspace_request() {
        receipt["wholeError"] = json!(error);
        return receipt;
    }
    receipt["acknowledgedDocumentGeneration"] = json!(editor.document_generation);
    match block_on(editor.client.request::<ProjectInfoRequest>(request)) {
        Ok(response) => {
            let selected = response
                .get("configFilePath")
                .and_then(Value::as_str)
                .filter(|path| !path.is_empty() && Path::new(path).is_absolute())
                .map(std::path::PathBuf::from);
            receipt["wholeResponse"] = response;
            if let Some(selected) = selected {
                receipt["status"] = json!("ok");
                receipt["selectedConfig"] = json!(selected);
            } else {
                receipt["wholeError"] =
                    json!("native materialized document is inferred or unconfigured");
            }
        }
        Err(error) => receipt["wholeError"] = json!(vize_l0::cstr!("{error}")),
    }
    receipt
}
