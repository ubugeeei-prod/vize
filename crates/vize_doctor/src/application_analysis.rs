//! Adapters from Vize whole-project analysis into doctor health findings.
//!
//! Enable the `application-analysis` feature to reuse an existing application
//! graph without reparsing source. Adapters preserve authored spans, related
//! files, stable source diagnostic identities, and invalidation inputs.

mod profile;

use std::{fmt, path::Component};

use vize_carton::{String, cstr};
use vize_croquis_cf::{CrossFileAnalyzer, CrossFileDiagnostic, CrossFileResult, FileId};

use crate::{
    AnalysisProvenance, DoctorFinding, DoctorReport, FindingEvidence, FindingFix, FixSafety,
    RelatedLocation, RuleCost, SourceLocation,
};

use self::profile::{failure_scenario, profile_for};

/// Source-integrity failure discovered while adapting an analysis result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplicationAnalysisError {
    /// A diagnostic references a file missing from its analyzer registry.
    MissingSource {
        /// Stable source diagnostic code.
        diagnostic_code: String,
        /// Unresolved application-graph file identifier.
        file_id: u32,
        /// Whether the unresolved identifier came from related evidence.
        related: bool,
    },
    /// A registered path cannot be represented relative to the workspace.
    SourceOutsideWorkspace {
        /// Stable source diagnostic code.
        diagnostic_code: String,
        /// Registered path that violated the workspace boundary.
        path: String,
        /// Whether the invalid path came from related evidence.
        related: bool,
    },
}

impl fmt::Display for ApplicationAnalysisError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSource {
                diagnostic_code,
                file_id,
                related,
            } => {
                let role = if *related { "related" } else { "primary" };
                write!(
                    formatter,
                    "{role} source {file_id} is missing for {diagnostic_code}"
                )
            }
            Self::SourceOutsideWorkspace {
                diagnostic_code,
                path,
                related,
            } => {
                let role = if *related { "related" } else { "primary" };
                write!(
                    formatter,
                    "{role} source path {path} is outside the workspace for {diagnostic_code}"
                )
            }
        }
    }
}

impl std::error::Error for ApplicationAnalysisError {}

/// Adapts whole-project diagnostics into deterministic doctor findings.
///
/// The supplied analyzer must be the analyzer that produced `result`. A stale
/// or unrelated registry returns [`ApplicationAnalysisError`] instead of emitting
/// a finding with a fabricated path.
pub fn findings_from_application_graph(
    analyzer: &CrossFileAnalyzer,
    result: &CrossFileResult,
) -> Result<Vec<DoctorFinding>, ApplicationAnalysisError> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| adapt_diagnostic(analyzer, diagnostic))
        .collect()
}

/// Builds a scored report directly from an existing whole-project analysis.
pub fn report_from_application_graph(
    workspace: impl Into<String>,
    analyzer: &CrossFileAnalyzer,
    result: &CrossFileResult,
) -> Result<DoctorReport, ApplicationAnalysisError> {
    let findings = findings_from_application_graph(analyzer, result)?;
    Ok(DoctorReport::new(workspace, findings))
}

fn adapt_diagnostic(
    analyzer: &CrossFileAnalyzer,
    diagnostic: &CrossFileDiagnostic,
) -> Result<DoctorFinding, ApplicationAnalysisError> {
    let code = doctor_code(diagnostic.code());
    let primary = source_location(
        analyzer,
        diagnostic,
        diagnostic.primary_file,
        diagnostic.primary_offset,
        diagnostic.primary_end_offset,
        false,
    )?;
    let profile = profile_for(&diagnostic.kind, diagnostic.severity);
    let evidence = FindingEvidence::new(profile.evidence, diagnostic.message.clone())
        .with_location(primary.clone())
        .with_detail("sourceDiagnostic", diagnostic.code());
    let mut invalidation_inputs = vec![primary.path.clone()];

    let mut finding = DoctorFinding::new(
        code,
        profile.category,
        profile.assessment,
        primary,
        rule_title(diagnostic.code()),
        diagnostic.message.clone(),
        AnalysisProvenance::new("whole-project-diagnostic-graph", RuleCost::Moderate),
    )
    .with_failure_scenario(failure_scenario(profile.category))
    .with_evidence(evidence);

    for (file_id, offset, message) in &diagnostic.related_files {
        let location = source_location(analyzer, diagnostic, *file_id, *offset, *offset, true)?;
        invalidation_inputs.push(location.path.clone());
        finding = finding.with_related(RelatedLocation::new(location, message.clone()));
    }

    finding.provenance = finding
        .provenance
        .with_invalidation_inputs(invalidation_inputs);
    if let Some(suggestion) = &diagnostic.suggestion {
        finding = finding.with_fix(FindingFix::new(
            FixSafety::ReviewRequired,
            suggestion.clone(),
        ));
    }
    Ok(finding)
}

fn source_location(
    analyzer: &CrossFileAnalyzer,
    diagnostic: &CrossFileDiagnostic,
    file_id: FileId,
    start: u32,
    end: u32,
    related: bool,
) -> Result<SourceLocation, ApplicationAnalysisError> {
    let path =
        analyzer
            .get_file_path(file_id)
            .ok_or_else(|| ApplicationAnalysisError::MissingSource {
                diagnostic_code: diagnostic.code().into(),
                file_id: file_id.as_u32(),
                related,
            })?;
    let relative = match analyzer.registry().project_root() {
        Some(root) => path.strip_prefix(root).map_err(|_| {
            outside_workspace_error(diagnostic, path.to_string_lossy().as_ref(), related)
        })?,
        None if path.is_relative() => path,
        None => {
            return Err(outside_workspace_error(
                diagnostic,
                path.to_string_lossy().as_ref(),
                related,
            ));
        }
    };
    let normalized = normalize_source_path(relative).ok_or_else(|| {
        outside_workspace_error(diagnostic, path.to_string_lossy().as_ref(), related)
    })?;
    Ok(SourceLocation::new(normalized, start, end))
}

fn normalize_source_path(path: &std::path::Path) -> Option<String> {
    let mut normalized = String::default();
    for component in path.components() {
        match component {
            Component::Normal(segment) => {
                let segment = segment.to_str()?;
                if !normalized.is_empty() {
                    normalized.push('/');
                }
                normalized.push_str(segment);
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    (!normalized.is_empty()).then_some(normalized)
}

fn outside_workspace_error(
    diagnostic: &CrossFileDiagnostic,
    path: &str,
    related: bool,
) -> ApplicationAnalysisError {
    ApplicationAnalysisError::SourceOutsideWorkspace {
        diagnostic_code: diagnostic.code().into(),
        path: path.into(),
        related,
    }
}

fn doctor_code(source_code: &str) -> String {
    let rule = source_code.rsplit('/').next().unwrap_or(source_code);
    cstr!(
        "VIZE_DOCTOR_CF_{}",
        rule.replace('-', "_").to_ascii_uppercase()
    )
}

fn rule_title(source_code: &str) -> String {
    let rule = source_code.rsplit('/').next().unwrap_or(source_code);
    let mut title = String::default();
    for word in rule.split('-') {
        if !title.is_empty() {
            title.push(' ');
        }
        let mut characters = word.chars();
        if let Some(first) = characters.next() {
            title.push(first.to_ascii_uppercase());
            title.push_str(characters.as_str());
        }
    }
    title
}
