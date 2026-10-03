//! Explicit checking of genuine completed JS/TS Modules at their authored path.

use std::path::{Path, PathBuf};

use lsp_types::{DocumentDiagnosticReport, DocumentDiagnosticReportResult, Range};
use sha2::{Digest, Sha256};
use vize_l0::{Span, String, cstr, line_index::LineBreaks};
use vize_l2::file::FileArtifact;
use vize_l4::targets::ts::{
    MappingError, ProgramProjection, ProjectionError, SourceKind, project_program,
};

use super::{CorsaBridge, CorsaBridgeError};

/// Refusals preserve the original File; no legacy checker fallback is attempted.
#[derive(Debug)]
pub enum OriginalProgramError {
    Projection(ProjectionError),
    MissingConfiguredProject,
    InvalidSourcePath,
    OutsideProject,
    SourceKindMismatch,
    UnsupportedModuleGoal,
    UnsupportedConfiguredProject,
    UnconfiguredSource,
    SourceChanged,
    ConfigurationChanged,
    Backend(CorsaBridgeError),
    IncompleteDiagnosticReport,
}

impl std::fmt::Display for OriginalProgramError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "original Program check refused: {self:?}")
    }
}

impl std::error::Error for OriginalProgramError {}

/// One complete raw backend report and its exact authored ranges retain the
/// same genuine projection and File. Related-document/raw fields are preserved.
///
/// The original owner cannot be dropped while its checker result is live:
/// ```compile_fail
/// use std::path::Path;
/// use vize_canon::CorsaBridge;
/// use vize_l2::file::FileArtifact;
/// async fn discard(bridge: &CorsaBridge, file: FileArtifact<'_>, path: &Path) {
///     let result = bridge.check_original_program(&file, path).await.unwrap();
///     drop(file);
///     let _ = result.report();
/// }
/// ```
pub struct OriginalProgramCheck<'file, 'arena> {
    projection: ProgramProjection<'file, 'arena>,
    source_path: PathBuf,
    source_uri: String,
    source_digest: String,
    report: DocumentDiagnosticReportResult,
    configuration: corsa::api::ConfigResponse,
    diagnostic_configuration_path: PathBuf,
    authored_spans: Vec<Result<Span, MappingError>>,
}

impl<'file, 'arena> OriginalProgramCheck<'file, 'arena> {
    #[must_use]
    pub fn projection(&self) -> &ProgramProjection<'file, 'arena> {
        &self.projection
    }
    #[must_use]
    pub fn source_path(&self) -> &Path {
        &self.source_path
    }
    #[must_use]
    pub fn source_uri(&self) -> &str {
        &self.source_uri
    }
    /// SHA-256 of the complete original bytes checked in this snapshot.
    #[must_use]
    pub fn source_digest(&self) -> &str {
        &self.source_digest
    }
    /// Complete ordered raw report, including related documents and every field.
    #[must_use]
    pub fn report(&self) -> &DocumentDiagnosticReportResult {
        &self.report
    }
    /// One exact byte span or typed mapping refusal for each primary diagnostic.
    pub fn authored_spans(&self) -> &[Result<Span, MappingError>] {
        &self.authored_spans
    }
    /// Actual normalized options and authored root-file membership from Corsa.
    #[must_use]
    pub fn configuration(&self) -> &corsa::api::ConfigResponse {
        &self.configuration
    }
    /// Absolute configured project attested by the process returning diagnostics.
    #[must_use]
    pub fn diagnostic_configuration_path(&self) -> &Path {
        &self.diagnostic_configuration_path
    }
}

impl CorsaBridge {
    /// Check an existing authored source in this bridge's configured project.
    ///
    /// The caller supplies its authoritative path, never a guessed virtual
    /// filename. The disk source must exactly match the original File both
    /// before and after the bounded check. The genuine L4 projection is sent
    /// only through a read-only LSP overlay; no source/configuration is written.
    /// A plain .js/.ts Module requires explicit moduleDetection: force in its
    /// root config or genuine import/export facts. .mjs/.mts are intrinsic Modules.
    pub async fn check_original_program<'file, 'arena>(
        &self,
        file: &'file FileArtifact<'arena>,
        authored_path: &Path,
    ) -> Result<OriginalProgramCheck<'file, 'arena>, OriginalProgramError> {
        let projection = project_program(file).map_err(OriginalProgramError::Projection)?;
        let root = self
            .config
            .working_dir
            .as_deref()
            .and_then(|root| root.canonicalize().ok())
            .ok_or(OriginalProgramError::MissingConfiguredProject)?;
        let config = [root.join("tsconfig.json"), root.join("jsconfig.json")]
            .into_iter()
            .find(|path| path.is_file())
            .ok_or(OriginalProgramError::MissingConfiguredProject)?;
        if root.to_str().is_none() || config.to_str().is_none() {
            return Err(OriginalProgramError::InvalidSourcePath);
        }
        // The existing LSP host selects the actual source's configuration.
        // A different explicit config cannot silently claim that identity.
        if self
            .config
            .tsconfig_path
            .as_ref()
            .is_some_and(|path| path.canonicalize().ok().as_ref() != Some(&config))
        {
            return Err(OriginalProgramError::MissingConfiguredProject);
        }
        if !authored_path.is_absolute() {
            return Err(OriginalProgramError::InvalidSourcePath);
        }
        let source_path = authored_path
            .canonicalize()
            .map_err(|_| OriginalProgramError::InvalidSourcePath)?;
        if source_path.to_str().is_none() {
            return Err(OriginalProgramError::InvalidSourcePath);
        }
        if !source_path.starts_with(&root) {
            return Err(OriginalProgramError::OutsideProject);
        }
        // Refuse a nested project rather than checking it under a root config
        // whose Module goal and options do not govern this actual source.
        if source_path
            .parent()
            .into_iter()
            .flat_map(Path::ancestors)
            .take_while(|parent| *parent != root)
            .any(|parent| {
                parent.join("tsconfig.json").is_file() || parent.join("jsconfig.json").is_file()
            })
        {
            return Err(OriginalProgramError::MissingConfiguredProject);
        }
        let configuration =
            std::fs::read(&config).map_err(|_| OriginalProgramError::MissingConfiguredProject)?;
        let extension = source_path.extension().and_then(|ext| ext.to_str());
        let correct_kind = match projection.source_kind() {
            SourceKind::JavaScript => matches!(extension, Some("js" | "mjs")),
            SourceKind::TypeScript => {
                matches!(extension, Some("ts" | "mts"))
                    && !source_path.to_string_lossy().ends_with(".d.ts")
                    && !source_path.to_string_lossy().ends_with(".d.mts")
            }
        };
        if !correct_kind {
            return Err(OriginalProgramError::SourceKindMismatch);
        }
        let needs_forced_module = !matches!(extension, Some("mjs" | "mts"))
            && file.imports().is_empty()
            && file.exports().is_empty();
        let original = file.artifact().source();
        verify_source(&source_path, original)?;
        let source_uri = crate::file_uri::path_to_file_uri(&source_path);
        let uri = source_uri.clone();
        let projected = String::from(projection.document().as_str());
        let source_digest = Sha256::digest(original.as_bytes())
            .iter()
            .map(|byte| cstr!("{byte:02x}"))
            .collect::<String>();
        let checked_path = source_path.clone();
        let checked_source = String::from(original);
        let checked_config = config.clone();
        let checked_configuration = configuration.clone();
        let (report, effective_configuration, diagnostic_configuration_path) = self
            .with_client(move |client| {
                // Bind the source again after entering the worker, so a
                // queued request cannot open a stale disk revision unnoticed.
                Ok((|| {
                    verify_source(&checked_path, &checked_source)?;
                    if std::fs::read(&checked_config).ok().as_deref()
                        != Some(checked_configuration.as_slice())
                    {
                        return Err(OriginalProgramError::ConfigurationChanged);
                    }
                    let report = client.original_program_diagnostics(
                        &uri,
                        &projected,
                        &checked_config,
                        needs_forced_module,
                    )?;
                    verify_source(&checked_path, &checked_source)?;
                    if std::fs::read(&checked_config).ok().as_deref()
                        != Some(checked_configuration.as_slice())
                    {
                        return Err(OriginalProgramError::ConfigurationChanged);
                    }
                    Ok(report)
                })())
            })
            .await
            .map_err(OriginalProgramError::Backend)??;
        verify_source(&source_path, original)?;
        if std::fs::read(&config).ok().as_deref() != Some(configuration.as_slice()) {
            return Err(OriginalProgramError::ConfigurationChanged);
        }
        let DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(full)) = &report
        else {
            return Err(OriginalProgramError::IncompleteDiagnosticReport);
        };
        let authored_spans = full
            .full_document_diagnostic_report
            .items
            .iter()
            .map(|raw| map_range(&projection, raw.range))
            .collect();
        Ok(OriginalProgramCheck {
            projection,
            source_path,
            source_uri,
            source_digest,
            report,
            configuration: effective_configuration,
            diagnostic_configuration_path,
            authored_spans,
        })
    }
}

fn verify_source(path: &Path, original: &str) -> Result<(), OriginalProgramError> {
    if std::fs::read(path).ok().as_deref() != Some(original.as_bytes()) {
        return Err(OriginalProgramError::SourceChanged);
    }
    Ok(())
}

fn map_range(projection: &ProgramProjection<'_, '_>, range: Range) -> Result<Span, MappingError> {
    let text = projection.document().as_str();
    let byte = |at: lsp_types::Position| {
        LineBreaks::Lsp
            .position_to_offset(text, at.line, at.character)
            .and_then(|offset| u32::try_from(offset).ok())
            .ok_or(MappingError::InvalidUtf16Boundary)
    };
    projection.map_span(Span::new(byte(range.start)?, byte(range.end)?))
}

#[cfg(test)]
mod failure;
#[cfg(test)]
mod tests;
