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
/// worker pool as a one-element pretty array. `serde_json` writes an array
/// element the same way at every position, so splicing those elements in input
/// order is byte for byte what `serde_json::to_string_pretty` writes for the
/// whole array. Only files with a diagnostic get a line index.
pub(super) fn format_json(results: &[LintResult], sources: &[(String, String)]) -> String {
    if results.is_empty() {
        return "[]".into();
    }
    let source_map: FxHashMap<&str, &str> = sources
        .iter()
        .map(|(filename, source)| (filename.as_str(), source.as_str()))
        .collect();
    // `[\n  {..}\n]`: the element keeps its `\n  ` lead-in and drops the brackets.
    let render = |result: &LintResult| {
        serde_json::to_string_pretty(std::slice::from_ref(&file_result(result, &source_map)))
            .unwrap_or_else(|_| "[\n  {}\n]".to_owned())
    };

    #[cfg(not(target_arch = "wasm32"))]
    let files: Vec<_> = {
        use rayon::prelude::*;
        results.par_iter().map(render).collect()
    };
    #[cfg(target_arch = "wasm32")]
    let files: Vec<_> = results.iter().map(render).collect();

    let mut output = String::with_capacity(files.iter().map(|file| file.len()).sum::<usize>());
    output.push('[');
    for (index, file) in files.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        let element = file.get(1..file.len().saturating_sub(2));
        output.push_str(element.unwrap_or_default());
    }
    output.push_str("\n]");
    output
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
