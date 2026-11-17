//! JSON output for tooling integration.

use crate::diagnostic::{HelpRenderTarget, render_help};
use crate::linter::LintResult;
use crate::output::shared::{SourceLineIndex, locate, rule_docs_path};
use serde::Serialize;
use vize_l0::{FxHashMap, String};

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
///
/// Files are independent: each one is located, rendered and serialized on the
/// worker pool, and the report is the files' pretty JSON joined in input
/// order, byte for byte what `serde_json::to_string_pretty` writes for the
/// whole array. Only files with a diagnostic get a line index.
pub(super) fn format_json(results: &[LintResult], sources: &[(String, String)]) -> String {
    if results.is_empty() {
        return "[]".into();
    }
    let source_map: FxHashMap<&str, &str> = sources
        .iter()
        .map(|(filename, source)| (filename.as_str(), source.as_str()))
        .collect();
    let render = |result: &LintResult| file_json(result, &source_map);

    #[cfg(not(target_arch = "wasm32"))]
    let files: Vec<String> = {
        use rayon::prelude::*;
        results.par_iter().map(render).collect()
    };
    #[cfg(target_arch = "wasm32")]
    let files: Vec<String> = results.iter().map(render).collect();

    let mut output =
        String::with_capacity(4 + files.iter().map(|file| file.len() + 2).sum::<usize>());
    output.push_str("[\n");
    for (index, file) in files.iter().enumerate() {
        if index > 0 {
            output.push_str(",\n");
        }
        output.push_str(file);
    }
    output.push_str("\n]");
    output
}

/// One file's element of the report array, pretty-printed one level deep.
fn file_json(result: &LintResult, source_map: &FxHashMap<&str, &str>) -> String {
    let element = serde_json::to_string_pretty(&file_result(result, source_map))
        .unwrap_or_else(|_| "{}".to_owned());
    nest(&element)
}

fn file_result(result: &LintResult, source_map: &FxHashMap<&str, &str>) -> JsonFileResult {
    let index = (!result.diagnostics.is_empty())
        .then(|| source_map.get(result.filename.as_str()))
        .flatten()
        .map(|source| SourceLineIndex::new(source));
    JsonFileResult {
        file: result.filename.clone(),
        messages: result
            .diagnostics
            .iter()
            .map(|d| {
                let location = locate(index.as_ref(), d.start, d.end);

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
        error_count: result.error_count,
        warning_count: result.warning_count,
    }
}

/// Pretty JSON escapes every newline inside a string, so each line break is
/// structure and indenting every line nests the element inside the array.
fn nest(element: &str) -> String {
    let mut nested = String::with_capacity(element.len() + element.len() / 8 + 2);
    for (index, line) in element.split('\n').enumerate() {
        if index > 0 {
            nested.push('\n');
        }
        nested.push_str("  ");
        nested.push_str(line);
    }
    nested
}

#[cfg(test)]
mod tests {
    use super::{file_result, format_json};
    use crate::{LintResult, Linter};
    use vize_l0::{FxHashMap, String};

    /// The per-file assembly is byte for byte the whole-array pretty JSON.
    #[test]
    fn per_file_assembly_matches_whole_array_serialization() {
        let sources = [
            (
                String::from("A.vue"),
                String::from(
                    "<template>\n  <div v-for=\"i in 3\">{{ i }}</div>\n  <p>\u{1F600}</p>\n</template>\n",
                ),
            ),
            (
                String::from("B.vue"),
                String::from("<template><p>ok</p></template>\n"),
            ),
            (
                String::from("C.vue"),
                String::from(
                    "<script setup>\nconst \"a\" = 1\n</script>\n<template><li v-for=\"x in y\"/></template>\n",
                ),
            ),
        ];
        let linter = Linter::new();
        let results: Vec<LintResult> = sources
            .iter()
            .map(|(filename, source)| linter.lint_sfc(source, filename))
            .collect();
        assert!(results.iter().any(|result| !result.diagnostics.is_empty()));
        assert!(results.iter().any(|result| result.diagnostics.is_empty()));
        let source_map: FxHashMap<&str, &str> = sources
            .iter()
            .map(|(filename, source)| (filename.as_str(), source.as_str()))
            .collect();
        let whole: Vec<_> = results
            .iter()
            .map(|result| file_result(result, &source_map))
            .collect();
        let expected = serde_json::to_string_pretty(&whole).expect("serializes");

        assert_eq!(format_json(&results, &sources).as_str(), expected);
        assert_eq!(
            format_json(&results[1..2], &sources).as_str(),
            serde_json::to_string_pretty(&whole[1..2]).expect("serializes")
        );
        assert_eq!(format_json(&[], &sources).as_str(), "[]");
    }
}
