use crate::{file_uri::file_uri_to_path, lsp_client::LspDiagnostic};
use lsp_types::{
    Diagnostic, DiagnosticSeverity, DocumentDiagnosticReport, DocumentDiagnosticReportResult,
    NumberOrString, Position, Range,
};
use vize_s0::{String, cstr};

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
