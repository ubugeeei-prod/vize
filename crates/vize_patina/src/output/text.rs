//! Rich terminal output using oxc_diagnostics.

#![allow(clippy::disallowed_macros)]

use crate::linter::LintResult;
use oxc_diagnostics::{GraphicalReportHandler, GraphicalTheme, NamedSource};
#[allow(clippy::disallowed_types)] // Required by oxc_diagnostics API
use std::sync::Arc;
use vize_carton::FxHashMap;
use vize_carton::String;

/// Format lint results as rich terminal output
#[allow(clippy::disallowed_types)] // Arc required by oxc_diagnostics API
pub fn format_text(results: &[LintResult], sources: &[(String, String)]) -> String {
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
            let oxc_diag = diagnostic.clone().into_oxc_diagnostic();
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
