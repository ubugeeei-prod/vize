//! Rich terminal output using oxc_diagnostics.

#![allow(clippy::disallowed_macros)]

use crate::diagnostic::{HelpRenderTarget, LintDiagnostic, Severity, render_help};
use crate::linter::LintResult;
use crate::output::rule_docs_path;
use oxc_diagnostics::{GraphicalReportHandler, GraphicalTheme, NamedSource, OxcDiagnostic};
use oxc_span::Span;
#[allow(clippy::disallowed_types)] // Required by oxc_diagnostics API
use std::sync::Arc;
use vize_carton::FxHashMap;
use vize_carton::String;
use vize_carton::ToCompactString;

/// Format lint results as rich terminal output
#[allow(clippy::disallowed_types)] // Arc required by oxc_diagnostics API
pub fn format_text(results: &[LintResult], sources: &[(String, String)]) -> String {
    format_graphical(results, sources, HelpRenderTarget::PlainText)
}

/// Format lint results as a full ANSI report.
#[allow(clippy::disallowed_types)] // Arc required by oxc_diagnostics API
pub fn format_ansi(results: &[LintResult], sources: &[(String, String)]) -> String {
    let mut output = format_graphical(results, sources, HelpRenderTarget::Ansi);
    let (errors, warnings) = result_counts(results);

    if !output.is_empty() {
        output.push('\n');
    }
    output.push_str(&format_summary(errors, warnings, results.len()));
    output.push('\n');
    output
}

#[allow(clippy::disallowed_types)] // Arc required by oxc_diagnostics API
fn format_graphical(
    results: &[LintResult],
    sources: &[(String, String)],
    help_target: HelpRenderTarget,
) -> String {
    let mut output = String::default();
    let handler = GraphicalReportHandler::new_themed(GraphicalTheme::unicode());

    // Create a map of filename to source
    let source_map: FxHashMap<&str, &str> = sources
        .iter()
        .map(|(f, s)| (f.as_str(), s.as_str()))
        .collect();

    for result in results {
        if result.diagnostics.is_empty() {
            continue;
        }

        // Get source for this file
        let source = source_map
            .get(result.filename.as_str())
            .copied()
            .unwrap_or("");

        let named_source = Arc::new(NamedSource::new(&result.filename, source.to_owned()));

        for diagnostic in &result.diagnostics {
            let oxc_diag = to_oxc_diagnostic(diagnostic, help_target);
            let report = oxc_diag.with_source_code(Arc::clone(&named_source));

            // Render using oxc_diagnostics
            let mut buf = String::default();
            if handler.render_report(&mut buf, report.as_ref()).is_ok() {
                output.push_str(&buf);
                output.push('\n');
            }
        }
    }

    output
}

fn to_oxc_diagnostic(diagnostic: &LintDiagnostic, help_target: HelpRenderTarget) -> OxcDiagnostic {
    let formatted_msg = format!("[vize:{}] {}", diagnostic.rule_name, diagnostic.message);

    let mut diag = match diagnostic.severity {
        Severity::Error => OxcDiagnostic::error(formatted_msg),
        Severity::Warning => OxcDiagnostic::warn(formatted_msg),
    };

    diag = diag.with_label(Span::new(diagnostic.start, diagnostic.end));

    let docs_path = rule_docs_path(diagnostic.rule_name);
    let help = diagnostic
        .help
        .as_ref()
        .map(|help| format!("{}\n\nReference: {}", help, docs_path))
        .unwrap_or_else(|| format!("Reference: {}", docs_path));
    diag = diag.with_help(render_help(&help, help_target));

    for label in &diagnostic.labels {
        diag = diag
            .and_label(Span::new(label.start, label.end).label(label.message.to_compact_string()));
    }

    diag
}

fn result_counts(results: &[LintResult]) -> (usize, usize) {
    let errors = results.iter().map(|result| result.error_count).sum();
    let warnings = results.iter().map(|result| result.warning_count).sum();
    (errors, warnings)
}

/// Format a summary line
pub fn format_summary(error_count: usize, warning_count: usize, file_count: usize) -> String {
    let mut parts = Vec::new();

    if error_count > 0 {
        parts.push(format!(
            "{} error{}",
            error_count,
            if error_count == 1 { "" } else { "s" }
        ));
    }

    if warning_count > 0 {
        parts.push(format!(
            "{} warning{}",
            warning_count,
            if warning_count == 1 { "" } else { "s" }
        ));
    }

    if parts.is_empty() {
        format!("No problems found in {} file(s)", file_count).into()
    } else {
        format!(
            "{} in {} file{}",
            parts.join(", "),
            file_count,
            if file_count == 1 { "" } else { "s" }
        )
        .into()
    }
}
