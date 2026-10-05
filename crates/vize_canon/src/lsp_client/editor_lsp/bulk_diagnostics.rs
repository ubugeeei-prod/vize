//! Project-wide categories on the already retained diagnosing attachment.

use super::{
    EditorLspSession,
    snapshot_source::{SnapshotSourceOwner, SourceTextOutcome},
};
use corsa::{CorsaError, runtime::block_on};
use lsp_types::Diagnostic;
use serde_json::json;
use std::path::Path;
use vize_l0::String;

mod conversion;
mod cost;
mod positions;
#[cfg(test)]
mod receipt;

#[cfg(test)]
pub(in crate::lsp_client) use receipt::fallback as fallback_receipt;
#[cfg(test)]
pub(in crate::lsp_client) use receipt::reset as reset_receipt;
#[cfg(test)]
pub(super) use receipt::take as take_receipt;

use conversion::{NativeDiagnostic, project_diagnostics_observed};

#[derive(Debug)]
pub(in crate::lsp_client) enum BulkDiagnostics {
    Complete(Vec<Vec<Diagnostic>>),
    Refused,
    RetireOwner {
        request_error: Option<CorsaError>,
        release_error: Option<CorsaError>,
    },
}

impl EditorLspSession {
    pub(in crate::lsp_client) fn bulk_diagnostics(
        &mut self,
        config: &Path,
        uris: &[String],
    ) -> Result<BulkDiagnostics, String> {
        #[cfg(test)]
        receipt::reset();
        let mut cost = cost::Cost::new();
        if cost.enabled() {
            cost.count("requested-uris", uris.len());
            cost.count("overlay-documents", self.documents.len());
            cost.count(
                "overlay-text-bytes",
                self.documents.values().map(|text| text.len()).sum(),
            );
        }
        // Keep the original response-backed overlay barrier. An API socket
        // request alone does not acknowledge LSP notification installation.
        let clock = cost.tick();
        let ready = self.ready_workspace_request();
        cost.duration("readiness", clock);
        ready?;
        let Some(api) = &self.configured_api else {
            return Ok(BulkDiagnostics::Refused);
        };
        let clock = cost.tick();
        let created = SnapshotSourceOwner::create(&api.client);
        cost.duration("snapshot-create", clock);
        let mut snapshot = match created {
            Ok(snapshot) => snapshot,
            Err(error) => {
                return Ok(BulkDiagnostics::RetireOwner {
                    request_error: Some(error),
                    release_error: None,
                });
            }
        };
        #[cfg(test)]
        receipt::record(json!({
            "snapshot":snapshot.handle(),
            "attachment":api.session, "outcome":"attempted",
        }));
        let result = (|| {
            let clock = cost.tick();
            let selected = snapshot.project(config);
            cost.duration("project-and-source-names", clock);
            let project = match selected? {
                Ok(project) => project,
                Err(_reason) => {
                    #[cfg(test)]
                    receipt::refusal(&vize_l0::cstr!(
                        "snapshot source project refuses: {_reason:?}"
                    ));
                    return Ok(None);
                }
            };
            cost.count("native-source-names", project.source_names().len());
            if !uris.iter().all(|uri| project.contains_uri(uri)) {
                #[cfg(test)]
                receipt::refusal("requested URI is not an exact native source-name member");
                return Ok(None);
            }
            let params = json!({"snapshot": snapshot.handle(), "project": project.descriptor().id});
            let mut categories = Vec::with_capacity(4);
            let methods = diagnostic_methods(&project.descriptor().compiler_options);
            #[cfg(test)]
            receipt::record(json!({
                "snapshot":snapshot.handle(),"snapshotProjects":[project.descriptor()],
                "project":project.descriptor(),"sourceFileNames":project.source_names(),
                "categoryMethods":methods,"attachment":api.session,"outcome":"attempted",
                "relatedSnapshotSources":[],
            }));
            for method in &methods {
                // Deliberately omit `file`: native 7.0.2 accepts this on each
                // category endpoint. The SDK's grouped endpoints are absent.
                let clock = cost.tick();
                let response = block_on(api.client.raw_json_request(method, params.clone()));
                cost.duration(method, clock);
                let value = response?;
                let clock = cost.tick();
                let Ok(diagnostics) =
                    serde_json::from_value::<Option<Vec<NativeDiagnostic>>>(value)
                else {
                    #[cfg(test)]
                    receipt::refusal("diagnostic response schema differs");
                    return Ok(None);
                };
                cost.duration("category-decode", clock);
                cost.count(method, diagnostics.as_ref().map_or(0, Vec::len));
                categories.push(diagnostics.unwrap_or_default());
            }
            let clock = cost.tick();
            let converted = project_diagnostics_observed(
                &categories,
                uris,
                &self.documents,
                |uri| {
                    let clock = cost.tick();
                    let response = project.read(uri);
                    cost.duration("related-snapshot-text", clock);
                    match response? {
                        SourceTextOutcome::Complete(view) => {
                            cost.count("related-text-bytes", view.text().len());
                            cost.count("related-encoded-bytes", view.encoded_bytes().len());
                            #[cfg(test)]
                            receipt::related_source(json!({
                                "requestedUri":uri,"snapshot":view.snapshot_handle(),
                                "project":view.project_descriptor(),"fileName":view.file_name(),
                                "path":view.path(),"text":view.text(),"encodedBytes":view.encoded_bytes(),
                            }));
                            Ok::<_, CorsaError>(positions::Positions::new(view.text()))
                        }
                        SourceTextOutcome::Refused {
                            reason: _reason,
                            encoded: _encoded,
                        } => {
                            #[cfg(test)]
                            receipt::related_source(json!({
                                "requestedUri":uri,"refusal":vize_l0::cstr!("{_reason:?}"),
                                "encodedBytes":_encoded.as_ref().map(|bytes| bytes.as_bytes()),
                            }));
                            Ok(None)
                        }
                    }
                },
                Some(&cost),
            );
            cost.duration("diagnostic-conversion-including-related-reads", clock);
            let converted = converted?;
            #[cfg(test)]
            if converted.is_none() {
                receipt::refusal(
                    "complete diagnostic conversion lacks owned text or valid coordinates",
                );
            }
            Ok(converted)
        })();
        // Eager release on successful conversion, bounded refusal, and error,
        // while the owning LSP process and its attachment are still alive.
        let clock = cost.tick();
        let cleanup = snapshot.release();
        cost.duration("snapshot-release", clock);
        let outcome = match (result, cleanup) {
            (Ok(Some(result)), Ok(())) => BulkDiagnostics::Complete(result),
            (Ok(None), Ok(())) | (Err(CorsaError::Unsupported(_)), Ok(())) => {
                BulkDiagnostics::Refused
            }
            (request, release) => BulkDiagnostics::RetireOwner {
                request_error: request.err(),
                release_error: release.err(),
            },
        };
        cost.outcome(match &outcome {
            BulkDiagnostics::Complete(_) => "complete",
            BulkDiagnostics::Refused => "refused",
            BulkDiagnostics::RetireOwner { .. } => "retire-owner",
        });
        #[cfg(test)]
        receipt::finish(&outcome);
        Ok(outcome)
    }
}

fn diagnostic_methods(options: &serde_json::Value) -> Vec<&'static str> {
    let mut methods = vec![
        "getSyntacticDiagnostics",
        "getSemanticDiagnostics",
        "getSuggestionDiagnostics",
    ];
    // core.CompilerOptions.GetEmitDeclarations includes Composite as well.
    if options["declaration"] == true || options["composite"] == true {
        methods.push("getDeclarationDiagnostics");
    }
    methods
}

#[cfg(test)]
mod tests;
