//! The editor's side of the one diagnostic post-pass (P4-5a).
//!
//! Corsa's answers for every virtual document projecting one SFC are decoded
//! into finished diagnostics and handed to
//! [`vize_canon::projection::assemble_diagnostics`] once; this module only
//! decodes LSP positions and renders the assembled result as LSP diagnostics.

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range};
use vize_canon::ImportSourceMap;
use vize_canon::projection::{
    AssembledOrigin, AssemblyPolicy, AuthoredSource, FinishedDiagnostic, ProjectedDocument,
    assemble_diagnostics,
};
use vize_canon::virtual_ts::ProjectionMapping;
use vize_s0::line_index::LineBreaks;

use super::super::{VirtualTsResult, sources};
use super::log_preview::log_preview;
use super::message::strip_corsa_overlay_paths;

/// One virtual document synced to Corsa, ready for assembly.
pub(super) struct CorsaDocument {
    pub(super) code: String,
    pub(super) mapping: ProjectionMapping,
    pub(super) import_source_map: ImportSourceMap,
}

impl CorsaDocument {
    pub(super) fn from_result(result: VirtualTsResult) -> Self {
        Self {
            code: result.code,
            mapping: ProjectionMapping::from_spans(result.source_mappings),
            import_source_map: result.import_source_map,
        }
    }
}

/// A finished Corsa diagnostic; the payload keeps Corsa's own code spelling.
pub(super) type CorsaFinished = FinishedDiagnostic<Option<serde_json::Value>>;

/// Ask Corsa for one synced document's diagnostics, in generated coordinates.
pub(super) async fn fetch_finished_diagnostics(
    bridge: &std::sync::Arc<vize_canon::CorsaBridge>,
    virtual_uri: &str,
    document: &CorsaDocument,
    index: usize,
) -> Result<Vec<CorsaFinished>, vize_canon::CorsaBridgeError> {
    tracing::info!(
        "generated virtual ts ({} bytes), mappings_count={}",
        document.code.len(),
        document.mapping.len()
    );
    let corsa_diags = bridge.get_diagnostics(virtual_uri).await?;
    tracing::info!(
        "corsa returned {} raw diagnostics for {}",
        corsa_diags.len(),
        virtual_uri
    );
    Ok(corsa_diags
        .into_iter()
        .enumerate()
        .filter_map(|(i, diag)| {
            tracing::info!(
                "  raw diag[{}]: line {}-{}, message: {}",
                i,
                diag.range.start.line,
                diag.range.end.line,
                log_preview(&diag.message, 100)
            );
            finished_from_lsp(&document.code, diag, index)
        })
        .collect())
}

/// Decode one LSP diagnostic of the synced (import-rewritten) document.
pub(super) fn finished_from_lsp(
    code: &str,
    diagnostic: vize_canon::corsa_bridge::LspDiagnostic,
    document: usize,
) -> Option<CorsaFinished> {
    let start = LineBreaks::Lsp.position_to_offset(
        code,
        diagnostic.range.start.line,
        diagnostic.range.start.character,
    )?;
    let end = LineBreaks::Lsp
        .position_to_offset(
            code,
            diagnostic.range.end.line,
            diagnostic.range.end.character,
        )
        .unwrap_or(start.saturating_add(1));
    Some(FinishedDiagnostic {
        document,
        start,
        end,
        code: diagnostic.code.as_ref().and_then(parse_code),
        severity: diagnostic.severity.map(|severity| match severity {
            1..=3 => severity,
            _ => 4,
        }),
        message: diagnostic.message.as_str().into(),
        payload: diagnostic.code,
    })
}

fn parse_code(code: &serde_json::Value) -> Option<u32> {
    match code {
        serde_json::Value::Number(number) => number.as_u64().and_then(|n| n.try_into().ok()),
        serde_json::Value::String(text) => text.trim_start_matches("TS").parse().ok(),
        _ => None,
    }
}

/// Assemble every document's finished diagnostics for the authored SFC.
pub(super) fn assemble_corsa_diagnostics(
    content: &str,
    documents: &[CorsaDocument],
    finished: Vec<CorsaFinished>,
) -> Vec<Diagnostic> {
    let projected: Vec<_> = documents
        .iter()
        .map(|document| ProjectedDocument {
            generated: &document.code,
            import_map: &document.import_source_map,
            mapping: Some(&document.mapping),
            tsx: false,
        })
        .collect();
    // An editor shows every unused declaration the checker reports; Corsa
    // tags them as faded hints unless the project turns them into errors.
    let policy = AssemblyPolicy {
        report_unused: true,
    };
    assemble_diagnostics(&AuthoredSource::vue(content), &projected, finished, policy)
        .into_iter()
        .map(|assembled| {
            let (start_line, start_character) =
                LineBreaks::Lsp.offset_to_position(content, assembled.start.min(content.len()));
            let (end_line, end_character) =
                LineBreaks::Lsp.offset_to_position(content, assembled.end.min(content.len()));
            let code = match assembled.origin {
                AssembledOrigin::Checker(Some(raw))
                | AssembledOrigin::MissingVueImport(Some(raw))
                    if parse_code(&raw) == assembled.code =>
                {
                    Some(corsa_diagnostic_code(raw))
                }
                _ => assembled
                    .code
                    .map(|code| NumberOrString::Number(code as i32)),
            };
            Diagnostic {
                range: Range {
                    start: Position::new(start_line, start_character),
                    end: Position::new(end_line, end_character),
                },
                severity: assembled.severity.map(|severity| match severity {
                    1 => DiagnosticSeverity::ERROR,
                    2 => DiagnosticSeverity::WARNING,
                    3 => DiagnosticSeverity::INFORMATION,
                    _ => DiagnosticSeverity::HINT,
                }),
                code,
                source: Some(sources::TYPE_CHECKER.to_string()),
                message: strip_corsa_overlay_paths(&assembled.message),
                ..Default::default()
            }
        })
        .collect()
}

pub(in crate::ide) fn corsa_diagnostic_code(code: serde_json::Value) -> NumberOrString {
    match code {
        serde_json::Value::Number(number) => number.as_i64().map_or_else(
            || NumberOrString::String(number.to_string()),
            |value| {
                i32::try_from(value).map_or_else(
                    |_| NumberOrString::String(number.to_string()),
                    NumberOrString::Number,
                )
            },
        ),
        serde_json::Value::String(code) => NumberOrString::String(code),
        other => NumberOrString::String(other.to_string()),
    }
}

#[cfg(test)]
#[path = "collect_virtual_tests.rs"]
mod tests;
