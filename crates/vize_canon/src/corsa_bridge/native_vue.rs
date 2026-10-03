//! Explicit checking of one genuine original SFC projection in a real configuration.

use super::{CorsaBridge, CorsaBridgeError, DiagnosingConfiguration, OriginalProgramError};
use lsp_types::{DocumentDiagnosticReport, DocumentDiagnosticReportResult, Range};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use vize_l0::{Span, String, cstr, line_index::LineBreaks};
use vize_l1_to_l2::native_file::NativeSfc;
use vize_l4::targets::ts::{
    MappingError,
    vue::{VueProjection, VueProjectionError, project_vue},
};

mod identity;

/// A typed refusal keeps the original native SFC untouched.
#[derive(Debug)]
pub enum NativeVueError {
    Projection(VueProjectionError),
    Identity(OriginalProgramError),
    ProjectionCollision,
    UnconfiguredProjection,
    Backend(CorsaBridgeError),
    IncompleteDiagnosticReport,
}
impl std::fmt::Display for NativeVueError {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(out, "native Vue check refused: {self:?}")
    }
}
impl std::error::Error for NativeVueError {}

/// Complete diagnostics and authored mapping retain every genuine native owner.
/// The original SFC owner cannot be discarded while diagnostics are retained.
/// ```compile_fail
/// use std::path::Path;
/// use vize_canon::CorsaBridge;
/// use vize_l1_to_l2::native_file::NativeSfcObservation;
/// async fn discard(bridge: &CorsaBridge, original: NativeSfcObservation<'_>, path: &Path) {
///     let checked = bridge.check_native_vue(original.admitted().unwrap(), path).await.unwrap();
///     drop(original);
///     let _ = checked.report();
/// }
/// ```
pub struct NativeVueCheck<'o, 'a> {
    projection: VueProjection<'o, 'a>,
    source_path: PathBuf,
    source_uri: String,
    source_digest: String,
    projection_uri: String,
    report: DocumentDiagnosticReportResult,
    configuration: corsa::api::ConfigResponse,
    project: corsa::api::ProjectResponse,
    diagnostic_configuration_path: PathBuf,
    diagnosing_configuration: DiagnosingConfiguration,
    authored_spans: Vec<Result<Span, MappingError>>,
}

impl<'o, 'a> NativeVueCheck<'o, 'a> {
    #[must_use]
    pub fn projection(&self) -> &VueProjection<'o, 'a> {
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
    /// SHA-256 of the complete original SFC bytes in this checked snapshot.
    #[must_use]
    pub fn source_digest(&self) -> &str {
        &self.source_digest
    }
    /// Canonical private derived filename actually admitted by the configured checker.
    #[must_use]
    pub fn projection_uri(&self) -> &str {
        &self.projection_uri
    }
    #[must_use]
    pub fn report(&self) -> &DocumentDiagnosticReportResult {
        &self.report
    }
    #[must_use]
    pub fn configuration(&self) -> &corsa::api::ConfigResponse {
        &self.configuration
    }
    #[must_use]
    pub fn project(&self) -> &corsa::api::ProjectResponse {
        &self.project
    }
    /// Actual configuration identity reported by the process returning diagnostics.
    #[must_use]
    pub fn diagnostic_configuration_path(&self) -> &Path {
        &self.diagnostic_configuration_path
    }
    /// Full observations from the same LSP session that returned this report.
    #[must_use]
    pub fn diagnosing_configuration(&self) -> &DiagnosingConfiguration {
        &self.diagnosing_configuration
    }
    /// Every primary diagnostic has one authored span or a typed mapping refusal.
    pub fn authored_spans(&self) -> &[Result<Span, MappingError>] {
        &self.authored_spans
    }
}

impl CorsaBridge {
    /// Check an existing authored SFC through its genuine completed native owner.
    /// The checker admits its private projected filename using the actual root
    /// configuration and immutable filesystem callbacks, without disk writes.
    pub async fn check_native_vue<'o, 'a>(
        &self,
        original: NativeSfc<'o, 'a>,
        authored_path: &Path,
    ) -> Result<NativeVueCheck<'o, 'a>, NativeVueError> {
        let projection = project_vue(original).map_err(NativeVueError::Projection)?;
        let bounds =
            identity::Bounds::checked(&self.config, authored_path, projection.source_kind())?;
        let source = projection.file().artifact().source();
        bounds.verify(source)?;
        let source_uri = crate::file_uri::path_to_file_uri(&bounds.source);
        let projection_uri = crate::file_uri::path_to_file_uri(&bounds.projected);
        let source_digest = Sha256::digest(source.as_bytes())
            .iter()
            .map(|byte| cstr!("{byte:02x}"))
            .collect::<String>();
        let projected_text = String::from(projection.document().as_str());
        let checked_source = String::from(source);
        let checked_bounds = bounds.clone();
        let checked_uri = projection_uri.clone();
        let (
            report,
            configuration,
            project,
            diagnostic_configuration_path,
            diagnosing_configuration,
        ) = self
            .with_client(move |client| {
                Ok((|| {
                    checked_bounds.verify(&checked_source)?;
                    let report = client.native_vue_diagnostics(
                        &checked_uri,
                        &projected_text,
                        &checked_bounds.config,
                        &checked_bounds.projected,
                    )?;
                    checked_bounds.verify(&checked_source)?;
                    Ok(report)
                })())
            })
            .await
            .map_err(NativeVueError::Backend)??;
        bounds.verify(source)?;
        let DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(full)) = &report
        else {
            return Err(NativeVueError::IncompleteDiagnosticReport);
        };
        let authored_spans = full
            .full_document_diagnostic_report
            .items
            .iter()
            .map(|raw| map_range(&projection, raw.range))
            .collect();
        Ok(NativeVueCheck {
            projection,
            source_path: bounds.source,
            source_uri,
            source_digest,
            projection_uri,
            report,
            configuration,
            project,
            diagnostic_configuration_path,
            diagnosing_configuration,
            authored_spans,
        })
    }
}

fn map_range(projection: &VueProjection<'_, '_>, range: Range) -> Result<Span, MappingError> {
    let byte = |at: lsp_types::Position| {
        LineBreaks::Lsp
            .position_to_offset(projection.document().as_str(), at.line, at.character)
            .and_then(|offset| u32::try_from(offset).ok())
            .ok_or(MappingError::InvalidUtf16Boundary)
    };
    projection.map_span(Span::new(byte(range.start)?, byte(range.end)?))
}

#[cfg(test)]
mod failure;
