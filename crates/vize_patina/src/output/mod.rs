//! Output formatters for lint diagnostics.

mod text;

pub use text::*;

use crate::diagnostic::{render_help, HelpRenderTarget};
use crate::linter::LintResult;
use serde::Serialize;
use vize_carton::{FxHashMap, SmallVec, String};

/// Output format for lint results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// Rich terminal output with colors and code snippets
    #[default]
    Text,
    /// JSON output for tooling integration
    Json,
}

/// Format lint results according to the specified format
pub fn format_results(
    results: &[LintResult],
    sources: &[(String, String)],
    format: OutputFormat,
) -> String {
    match format {
        OutputFormat::Text => format_text(results, sources),
        OutputFormat::Json => format_json(results, sources),
    }
}

/// JSON output structure for a single file
#[derive(Debug, Serialize)]
pub struct JsonFileResult {
    pub file: String,
    pub messages: Vec<JsonMessage>,
    #[serde(rename = "errorCount")]
    pub error_count: usize,
    #[serde(rename = "warningCount")]
    pub warning_count: usize,
}

/// JSON output structure for a single message
#[derive(Debug, Serialize)]
pub struct JsonMessage {
    #[serde(rename = "ruleId")]
    pub rule_id: &'static str,
    pub severity: u8,
    pub message: String,
    pub line: u32,
    pub column: u32,
    #[serde(rename = "endLine")]
    pub end_line: u32,
    #[serde(rename = "endColumn")]
    pub end_column: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}

/// Format results as JSON
fn format_json(results: &[LintResult], sources: &[(String, String)]) -> String {
    let source_indices: FxHashMap<&str, SourceLineIndex> = sources
        .iter()
        .map(|(filename, source)| (filename.as_str(), SourceLineIndex::new(source.as_str())))
        .collect();

    let json_results: Vec<JsonFileResult> = results
        .iter()
        .map(|r| JsonFileResult {
            file: r.filename.clone(),
            messages: r
                .diagnostics
                .iter()
                .map(|d| {
                    let (line, column, end_line, end_column) = source_indices
                        .get(r.filename.as_str())
                        .map(|source| {
                            let (line, column) = source.offset_to_line_col(d.start);
                            let (end_line, end_column) = source.offset_to_line_col(d.end);
                            (line, column, end_line, end_column)
                        })
                        .unwrap_or((1, d.start + 1, 1, d.end + 1));

                    JsonMessage {
                        rule_id: d.rule_name,
                        severity: match d.severity {
                            crate::diagnostic::Severity::Error => 2,
                            crate::diagnostic::Severity::Warning => 1,
                        },
                        // Use formatted message with [vize:RULE] prefix
                        message: d.formatted_message(),
                        line,
                        column,
                        end_line,
                        end_column,
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

struct SourceLineIndex {
    source_len: usize,
    line_starts: SmallVec<[usize; 64]>,
}

impl SourceLineIndex {
    fn new(source: &str) -> Self {
        let bytes = source.as_bytes();
        let mut line_starts = SmallVec::new();
        line_starts.push(0);

        for (index, &byte) in bytes.iter().enumerate() {
            if byte == b'\n' {
                line_starts.push(index + 1);
            }
        }

        Self {
            source_len: bytes.len(),
            line_starts,
        }
    }

    fn offset_to_line_col(&self, offset: u32) -> (u32, u32) {
        let offset = (offset as usize).min(self.source_len);
        let line_index = self
            .line_starts
            .partition_point(|&line_start| line_start <= offset)
            .saturating_sub(1);
        let line_start = self.line_starts.get(line_index).copied().unwrap_or(0);
        let line = line_index as u32 + 1;
        let column = offset.saturating_sub(line_start) as u32 + 1;

        (line, column)
    }
}

#[cfg(test)]
mod tests {
    use crate::{format_results, Linter, OutputFormat};

    #[test]
    fn json_output_uses_source_line_columns() {
        let source = r#"<script setup lang="ts">
const items = [1]
</script>

<template>
  <div v-for="item in items">{{ item }}</div>
</template>
"#;
        let filename = vize_carton::String::from("Component.vue");
        let result = Linter::new().lint_sfc(source, &filename);
        let output = format_results(
            &[result],
            &[(filename, vize_carton::String::from(source))],
            OutputFormat::Json,
        );

        assert!(output.contains(r#""line": 6"#), "{output}");
        assert!(output.contains(r#""column": 8"#), "{output}");
    }
}
