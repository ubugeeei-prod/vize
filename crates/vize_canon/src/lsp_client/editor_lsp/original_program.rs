//! Independent authored diagnosing process retained across complete checks.

use super::{
    CorsaProjectClient, EditorLspSession, project_configuration::ConfigurationError,
    project_identity::ProjectIdentityError,
};
use crate::{
    corsa_bridge::{CorsaBridgeError, DiagnosingConfiguration, OriginalProgramError},
    corsa_session_cache::CorsaSessionKey,
};
use corsa::{
    api::{ConfigResponse, FileChanges, UpdateSnapshotParams},
    runtime::block_on,
};
use lsp_types::DocumentDiagnosticReportResult;
use sha2::{Digest, Sha256};
use std::{
    io::Read,
    path::{Path, PathBuf},
};
use vize_l0::{String, cstr};

type OriginalReport = (
    DocumentDiagnosticReportResult,
    ConfigResponse,
    PathBuf,
    DiagnosingConfiguration,
);

pub(in crate::lsp_client) struct OriginalDiagnosingSession {
    key: Option<CorsaSessionKey>,
    editor: EditorLspSession,
    project_opened: bool,
}

impl CorsaProjectClient {
    pub(crate) fn original_program_diagnostics(
        &mut self,
        uri: &str,
        projected: &str,
        config: &Path,
        needs_forced_module: bool,
    ) -> Result<OriginalReport, OriginalProgramError> {
        let result =
            self.original_program_diagnostics_inner(uri, projected, config, needs_forced_module);
        let uncacheable = self
            .original_diagnosing_session
            .as_ref()
            .is_some_and(|owner| owner.key.is_none());
        if result.is_err() || uncacheable {
            let cleanup = self.retire_original_diagnosing_session().map_err(backend);
            return match (result, cleanup) {
                (Ok(report), Ok(())) => Ok(report),
                (Err(error), cleanup) => {
                    if let Err(cleanup) = cleanup {
                        // Keep the existing typed refusal/first-error precedence,
                        // while retaining the full secondary cleanup diagnostic.
                        tracing::error!(error = %cleanup, "authored diagnosing cleanup failed");
                    }
                    Err(error)
                }
                (_, Err(error)) => Err(error),
            };
        }
        result
    }

    fn original_program_diagnostics_inner(
        &mut self,
        uri: &str,
        projected: &str,
        config: &Path,
        needs_forced_module: bool,
    ) -> Result<OriginalReport, OriginalProgramError> {
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
        let key = diagnosing_key(self.executable.as_str(), config);
        if self
            .original_diagnosing_session
            .as_ref()
            .is_some_and(|owner| key.is_none() || owner.key != key)
        {
            self.retire_original_diagnosing_session().map_err(backend)?;
        }
        if self.original_diagnosing_session.is_none() {
            self.original_diagnosing_session = Some(OriginalDiagnosingSession {
                key,
                editor: EditorLspSession::spawn(
                    self.executable.as_str(),
                    &self.cwd,
                    &self.project_root,
                )
                .map_err(backend)?,
                project_opened: false,
            });
        }
        let owner = self
            .original_diagnosing_session
            .as_mut()
            .ok_or_else(|| backend(cstr!("authored diagnosing session is missing")))?;
        let (report, configuration, receipt) = owner.query(uri, projected, config, &effective)?;
        Ok((report, effective, configuration, receipt))
    }

    pub(in crate::lsp_client) fn retire_original_diagnosing_session(
        &mut self,
    ) -> Result<(), String> {
        self.original_diagnosing_session
            .take()
            .map_or(Ok(()), |mut owner| owner.editor.shutdown())
    }
}

impl OriginalDiagnosingSession {
    fn query(
        &mut self,
        uri: &str,
        projected: &str,
        config: &Path,
        effective: &ConfigResponse,
    ) -> Result<
        (
            DocumentDiagnosticReportResult,
            PathBuf,
            DiagnosingConfiguration,
        ),
        OriginalProgramError,
    > {
        self.editor.mirror(uri, projected).map_err(backend)?;
        let configuration = self
            .editor
            .diagnosing_configuration(uri)
            .map_err(identity_error)?;
        if configuration != config {
            return Err(OriginalProgramError::UnconfiguredSource);
        }
        if self.editor.configured_api.is_none() {
            self.editor.configured_api =
                Some(self.editor.diagnosing_api(uri).map_err(custody_error)?);
        }
        let params = UpdateSnapshotParams {
            open_project: if self.project_opened {
                None
            } else {
                Some(
                    config
                        .to_str()
                        .ok_or(OriginalProgramError::InvalidSourcePath)?
                        .into(),
                )
            },
            file_changes: Some(FileChanges::InvalidateAll {
                invalidate_all: true,
            }),
            overlay_changes: None,
        };
        // Refresh the actual diagnosing graph in the existing before-observation
        // RPC. Keep its project ref; releasing a snapshot does not release it.
        let before = self
            .editor
            .configured_api
            .as_ref()
            .ok_or_else(|| backend(cstr!("authored diagnosing attachment is missing")))?
            .observe_with_params(uri, config, &effective.options, params)
            .map_err(custody_error)?;
        self.project_opened = true;
        let report = self.editor.diagnostics(uri).map_err(backend)?;
        if self
            .editor
            .diagnosing_configuration(uri)
            .map_err(identity_error)?
            != configuration
        {
            return Err(OriginalProgramError::UnconfiguredSource);
        }
        let api = self
            .editor
            .configured_api
            .as_ref()
            .ok_or_else(|| backend(cstr!("authored diagnosing attachment is missing")))?;
        let after = api
            .observe(uri, config, &effective.options)
            .map_err(custody_error)?;
        let receipt = DiagnosingConfiguration {
            session: api.session.clone(),
            before,
            after,
        };
        // Only the current authored overlay is opened. The next query must
        // acknowledge this close and its own open through the existing barrier.
        self.editor
            .synchronize(&Default::default())
            .map_err(backend)?;
        Ok((report, configuration, receipt))
    }
}

fn diagnosing_key(executable: &str, config: &Path) -> Option<CorsaSessionKey> {
    // Unknown/unreadable launchers preserve the previous fresh-process path.
    // Hash complete launch bytes; size/mtime alone cannot identify a runtime.
    let path = Path::new(executable).canonicalize().ok()?;
    let mut input = std::fs::File::open(&path).ok()?;
    let mut magic = [0_u8; 4];
    input.read_exact(&mut magic).ok()?;
    if !native_image(magic) {
        return None;
    }
    let mut digest = Sha256::new();
    digest.update(magic);
    let mut buffer = [0_u8; 8192];
    loop {
        let count = input.read(&mut buffer).ok()?;
        if count == 0 {
            break;
        }
        digest.update(buffer.get(..count)?);
    }
    let bytes = digest
        .finalize()
        .iter()
        .map(|byte| cstr!("{byte:02x}"))
        .collect::<String>();
    let identity = cstr!("{} sha256={bytes}", path.display());
    Some(CorsaSessionKey::new(config, identity.as_str(), ""))
}

// Scripts can delegate to a different binary without changing their own bytes.
// Only direct native images can hold a retained diagnosing owner.
fn native_image(magic: [u8; 4]) -> bool {
    matches!(
        magic,
        [0x7f, b'E', b'L', b'F']
            | [0xfe, 0xed, 0xfa, 0xce]
            | [0xce, 0xfa, 0xed, 0xfe]
            | [0xfe, 0xed, 0xfa, 0xcf]
            | [0xcf, 0xfa, 0xed, 0xfe]
            | [0xca, 0xfe, 0xba, 0xbe]
            | [0xbe, 0xba, 0xfe, 0xca]
    ) || cfg!(windows) && magic.starts_with(b"MZ")
}

fn backend(error: String) -> OriginalProgramError {
    OriginalProgramError::Backend(CorsaBridgeError::CommunicationError(error))
}

fn identity_error(error: ProjectIdentityError) -> OriginalProgramError {
    match error {
        ProjectIdentityError::Communication(error) => backend(error),
        ProjectIdentityError::Unconfigured => OriginalProgramError::UnconfiguredSource,
    }
}

fn custody_error(error: ConfigurationError) -> OriginalProgramError {
    match error {
        ConfigurationError::Communication(error) => backend(error),
        ConfigurationError::Unconfigured => OriginalProgramError::UnconfiguredSource,
        ConfigurationError::Changed => OriginalProgramError::ConfigurationChanged,
        #[cfg(not(unix))]
        ConfigurationError::Unsupported => OriginalProgramError::UnsupportedConfiguredProject,
    }
}

#[cfg(test)]
mod key_tests;
#[cfg(test)]
#[cfg(target_os = "linux")]
mod process_tests;
