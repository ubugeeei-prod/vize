//! JSON output for tooling integration.

use crate::diagnostic::{HelpRenderTarget, render_help};
use crate::linter::LintResult;
use crate::output::shared::{diagnostic_location, rule_docs_path, source_indices};
use serde::Serialize;
use vize_l0::String;

/// JSON output structure for a single file
#[derive(Debug, Serialize)]
pub struct JsonFileResult {
    /// Path of the linted source file.
    pub file: String,
    /// Diagnostics reported for the file.
    pub messages: Vec<JsonMessage>,
    #[serde(rename = "errorCount")]
    /// Number of error-severity messages.
    pub error_count: usize,
    #[serde(rename = "warningCount")]
    /// Number of warning-severity messages.
    pub warning_count: usize,
}

/// JSON output structure for a single message
#[derive(Debug, Serialize)]
pub struct JsonMessage {
    #[serde(rename = "ruleId")]
    /// Stable identifier of the rule that emitted the message.
    pub rule_id: &'static str,
    #[serde(rename = "ruleDocsPath")]
    /// Repository-relative documentation path for the rule.
    pub rule_docs_path: String,
    /// Numeric lint severity compatible with ESLint-style consumers.
    pub severity: u8,
    /// Formatted diagnostic message.
    pub message: String,
    /// One-based start line.
    pub line: u32,
    /// One-based start column.
    pub column: u32,
    #[serde(rename = "endLine")]
    /// One-based end line.
    pub end_line: u32,
    #[serde(rename = "endColumn")]
    /// One-based end column.
    pub end_column: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Optional plain-text remediation guidance.
    pub help: Option<String>,
}

/// Format results as JSON
pub(super) fn format_json(results: &[LintResult], sources: &[(String, String)]) -> String {
    let source_indices = source_indices(sources);
    let json_results: Vec<JsonFileResult> = results
        .iter()
        .map(|r| JsonFileResult {
            file: r.filename.clone(),
            messages: r
                .diagnostics
                .iter()
                .map(|d| {
                    let location =
                        diagnostic_location(r.filename.as_str(), d.start, d.end, &source_indices);

                    JsonMessage {
                        rule_id: d.rule_name,
                        rule_docs_path: rule_docs_path(d.rule_name),
                        severity: match d.severity {
                            crate::diagnostic::Severity::Error => 2,
                            crate::diagnostic::Severity::Warning => 1,
                        },
                        // Use formatted message with [vize:RULE] prefix
                        message: d.formatted_message(),
                        line: location.line,
                        column: location.column,
                        end_line: location.end_line,
                        end_column: location.end_column,
                        help: d
                            .help
                            .as_ref()
                            .map(|h| render_help(h, HelpRenderTarget::PlainText)),
                    }
                })
                .collect(),
            error_count: r.error_count,
            warning_count: r.warning_count,
        })
        .collect();

    serde_json::to_string_pretty(&json_results)
        .unwrap_or_else(|_| "[]".to_owned())
        .into()
}
