//! Read-only authored-file overlays, bypassing virtual project materialization.

use super::{CorsaProjectClient, EditorLspSession, project_identity::ProjectIdentityError};
use crate::corsa_bridge::{CorsaBridgeError, OriginalProgramError};
use corsa::{
    api::{ConfigResponse, FileChanges},
    runtime::block_on,
};
use lsp_types::DocumentDiagnosticReportResult;
use std::path::{Path, PathBuf};
use vize_l0::cstr;

impl CorsaProjectClient {
    pub(crate) fn original_program_diagnostics(
        &mut self,
        uri: &str,
        projected: &str,
        config: &Path,
        needs_forced_module: bool,
    ) -> Result<(DocumentDiagnosticReportResult, ConfigResponse, PathBuf), OriginalProgramError>
    {
        let backend =
            |error| OriginalProgramError::Backend(CorsaBridgeError::CommunicationError(error));
        if self.materialized_project_session || self.document_texts.contains_key(uri) {
            return Err(backend(cstr!(
                "authored Program requires an unmaterialized, unopened source"
            )));
        }
        let session = self
            .session
            .as_mut()
            .ok_or(OriginalProgramError::UnsupportedConfiguredProject)?;
        // Flush cached filesystem/configuration state before the actual
        // compiler parses root membership and inherited options.
        block_on(session.refresh(Some(FileChanges::InvalidateAll {
            invalidate_all: true,
        })))
        .map_err(|error| backend(cstr!("cannot refresh authored configuration: {error}")))?;
        let effective = block_on(session.client().parse_config_file(
            super::super::session::uri_document_identifier(&crate::file_uri::path_to_file_uri(
                config,
            )),
        ))
        .map_err(|error| backend(cstr!("cannot resolve authored configuration: {error}")))?;
        let path = crate::file_uri::file_uri_to_path(uri)
            .ok_or(OriginalProgramError::InvalidSourcePath)?;
        if !effective.file_names.iter().any(|name| {
            let candidate = Path::new(name);
            let candidate = if candidate.is_absolute() {
                candidate.to_path_buf()
            } else {
                self.cwd.join(candidate)
            };
            candidate.canonicalize().ok().as_ref() == Some(&path)
        }) {
            return Err(OriginalProgramError::UnconfiguredSource);
        }
        if needs_forced_module
            && !effective
                .options
                .get("moduleDetection")
                .is_some_and(|mode| mode == 3 || mode == "force")
        {
            return Err(OriginalProgramError::UnsupportedModuleGoal);
        }
        // An independent overlay session cannot replace an editor's live
        // document, trigger disk materialization or inherit its stale revision.
        let mut session =
            EditorLspSession::spawn(self.executable.as_str(), &self.cwd, &self.project_root)
                .map_err(backend)?;
        let result = (|| {
            session.mirror(uri, projected).map_err(backend)?;
            let configuration = |session: &mut EditorLspSession| {
                session
                    .diagnosing_configuration(uri)
                    .map_err(|error| match error {
                        ProjectIdentityError::Communication(error) => backend(error),
                        ProjectIdentityError::Unconfigured => {
                            OriginalProgramError::UnconfiguredSource
                        }
                    })
            };
            let diagnosing_config = configuration(&mut session)?;
            if diagnosing_config != config {
                return Err(OriginalProgramError::UnconfiguredSource);
            }
            let report = session.diagnostics(uri).map_err(backend)?;
            if configuration(&mut session)? != diagnosing_config {
                return Err(OriginalProgramError::UnconfiguredSource);
            }
            Ok((report, effective, diagnosing_config))
        })();
        // Graceful shutdown closes/reaps every overlay even after a failed
        // request. The bridge worker continues this cleanup after a timeout.
        let cleanup = session.shutdown().map_err(backend);
        match (result, cleanup) {
            (Ok(report), Ok(())) => Ok(report),
            (Err(error), _) | (_, Err(error)) => Err(error),
        }
    }
}
