//! Project-wide categories on the already retained diagnosing attachment.

use super::EditorLspSession;
use corsa::{CorsaError, runtime::block_on};
use lsp_types::Diagnostic;
use serde_json::json;
use std::path::Path;
use vize_l0::{String, cstr};

mod conversion;
mod positions;
#[cfg(test)]
mod receipt;

#[cfg(test)]
pub(super) use receipt::take as take_receipt;

use conversion::{NativeDiagnostic, project_diagnostics};

impl EditorLspSession {
    pub(in crate::lsp_client) fn bulk_diagnostics(
        &mut self,
        config: &Path,
        uris: &[String],
    ) -> Result<Option<Vec<Vec<Diagnostic>>>, String> {
        #[cfg(test)]
        receipt::reset();
        // Keep the original response-backed overlay barrier. An API socket
        // request alone does not acknowledge LSP notification installation.
        self.ready_workspace_request()?;
        let Some(api) = &self.configured_api else {
            return Ok(None);
        };
        let snapshot = block_on(api.client.update_snapshot(Default::default()))
            .map_err(|error| cstr!("Cannot update bulk diagnosing snapshot: {error}"))?;
        let result = (|| {
            // A single retained configured project makes default-project
            // ownership unambiguous. Mixed configured/inferred views fall back.
            let [project] = snapshot.projects.as_slice() else {
                return Ok(None);
            };
            if Path::new(&project.config_file_name) != config {
                return Ok(None);
            }
            let params = json!({"snapshot": snapshot.handle, "project": project.id});
            let names = block_on(
                api.client
                    .raw_json_request("getSourceFileNames", params.clone()),
            )?;
            let names: Vec<String> = serde_json::from_value(names)
                .map_err(|_| CorsaError::Unsupported("native source names payload"))?;
            if !conversion::requested_members_are_present(uris, &names) {
                return Ok(None);
            }
            let mut categories = Vec::with_capacity(4);
            let methods = diagnostic_methods(&project.compiler_options);
            for method in &methods {
                // Deliberately omit `file`: native 7.0.2 accepts this on each
                // category endpoint. The SDK's grouped endpoints are absent.
                let value = block_on(api.client.raw_json_request(method, params.clone()))?;
                let diagnostics: Option<Vec<NativeDiagnostic>> = serde_json::from_value(value)
                    .map_err(|_| CorsaError::Unsupported("native category payload"))?;
                categories.push(diagnostics.unwrap_or_default());
            }
            let converted = project_diagnostics(&categories, uris, &self.documents);
            #[cfg(test)]
            if converted.is_some() {
                receipt::record(json!({
                    "snapshot": snapshot.handle, "project": project,
                    "sourceFileNames": names, "categoryMethods": methods,
                    "attachment": api.session,
                }));
            }
            Ok(converted)
        })();
        // Eager release on successful conversion, bounded refusal, and error,
        // while the owning LSP process and its attachment are still alive.
        let cleanup = block_on(snapshot.release())
            .map_err(|error| cstr!("Cannot release bulk diagnosing snapshot: {error}"));
        match (result, cleanup) {
            (_, Err(error)) => Err(error),
            (Ok(result), Ok(())) => Ok(result),
            (Err(CorsaError::Unsupported(_)), Ok(())) => Ok(None),
            (Err(error), Ok(())) => Err(cstr!("Cannot request native bulk diagnostics: {error}")),
        }
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
