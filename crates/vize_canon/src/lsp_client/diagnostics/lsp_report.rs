use crate::{file_uri::file_uri_to_path, lsp_client::LspDiagnostic};
use lsp_types::{
    Diagnostic, DiagnosticSeverity, DocumentDiagnosticReport, DocumentDiagnosticReportResult,
    NumberOrString, Position, Range,
};
use vize_l0::{String, cstr};

pub(super) fn extract_diagnostics_with_provenance(
    report: DocumentDiagnosticReportResult,
) -> Vec<super::PhasedDiagnostic> {
    extract_diagnostics(report)
        .into_iter()
        .map(super::PhasedDiagnostic::unknown)
        .collect()
}

pub(super) fn extract_diagnostics(report: DocumentDiagnosticReportResult) -> Vec<Diagnostic> {
    match report {
        DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(report)) => {
            report.full_document_diagnostic_report.items
        }
        DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Unchanged(_)) => {
            Vec::new()
        }
        DocumentDiagnosticReportResult::Partial(_) => Vec::new(),
    }
}

pub(super) fn lsp_diagnostic_to_native(diagnostic: LspDiagnostic) -> Diagnostic {
    Diagnostic {
        range: Range::new(
            Position::new(
                diagnostic.range.start.line,
                diagnostic.range.start.character,
            ),
            Position::new(diagnostic.range.end.line, diagnostic.range.end.character),
        ),
        severity: diagnostic.severity.and_then(lsp_severity_from_i32),
        code: diagnostic.code.map(json_code_to_lsp_code),
        code_description: None,
        source: diagnostic.source.map(|source| source.into()),
        message: diagnostic.message.into(),
        related_information: None,
        tags: None,
        data: None,
    }
}

pub(super) fn read_file_uri(uri: &str) -> Option<String> {
    let path = file_uri_to_path(uri)?;
    std::fs::read_to_string(path).ok().map(Into::into)
}

fn lsp_severity_from_i32(severity: i32) -> Option<DiagnosticSeverity> {
    match severity {
        1 => Some(DiagnosticSeverity::ERROR),
        2 => Some(DiagnosticSeverity::WARNING),
        3 => Some(DiagnosticSeverity::INFORMATION),
        4 => Some(DiagnosticSeverity::HINT),
        _ => None,
    }
}

fn json_code_to_lsp_code(code: serde_json::Value) -> NumberOrString {
    match code {
        serde_json::Value::Number(number) => {
            if let Some(value) = number.as_i64() {
                NumberOrString::Number(value as i32)
            } else {
                NumberOrString::String(cstr!("{number}").into())
            }
        }
        serde_json::Value::String(string) => NumberOrString::String(string),
        other => NumberOrString::String(cstr!("{other}").into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_lsp_report_retains_unknown_phase_even_for_a_semantic_1xxx_code() {
        let report = serde_json::from_value(serde_json::json!({
            "kind": "full",
            "items": [{
                "range": {"start": {"line": 0, "character": 7}, "end": {"line": 0, "character": 10}},
                "code": 1192, "severity": 1, "source": "ts", "message": "no default export"
            }]
        })).unwrap();
        let rows = extract_diagnostics_with_provenance(report);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].phase, crate::corsa_bridge::DiagnosticPhase::Unknown);
        assert_eq!(rows[0].diagnostic.code, Some(NumberOrString::Number(1192)));
    }
}
