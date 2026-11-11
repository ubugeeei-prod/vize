//! A selected file that cannot be read is a lint error, not a skipped file.

use std::io;
use std::path::Path;
use vize_carton::source_io as fs;
use vize_l0::{String, ToCompactString, cstr, profile, profiler::global_profiler};
use vize_patina::{LintDiagnostic, LintResult};

const UNREADABLE: &str = "lint/unreadable";

/// Read one selected lint file.
///
/// Any read error, including invalid UTF-8, becomes one error diagnostic so
/// the run exits 1 and the file stays in the report.
pub(super) fn read_lint_source(path: &Path) -> Result<String, LintResult> {
    match profile!("cli.lint.file.read", fs::read_to_string(path)) {
        Ok(source) => {
            global_profiler().record_fs_read_to_string(source.len());
            Ok(source.into())
        }
        Err(error) => {
            global_profiler().record_fs_read_to_string_failure();
            Err(unreadable_result(path, &error))
        }
    }
}

fn unreadable_result(path: &Path, error: &io::Error) -> LintResult {
    let diagnostic = LintDiagnostic::error(
        UNREADABLE,
        cstr!("Failed to read {}: {}", path.display(), error),
        0,
        0,
    );
    LintResult {
        filename: path.to_string_lossy().to_compact_string(),
        diagnostics: vec![diagnostic],
        error_count: 1,
        warning_count: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::read_lint_source;
    use crate::commands::lint::aggregate::LintRunAccumulator;
    use serde_json::json;
    use std::path::Path;
    use vize_l0::String;
    use vize_patina::OutputFormat;

    fn write_bytes(path: &Path, bytes: &[u8]) {
        std::fs::write(path, bytes).unwrap();
    }

    #[test]
    fn invalid_utf8_stays_in_the_report() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("Latin1.vue");
        write_bytes(
            &path,
            b"<template>\n  <img src=\"/a.png\" title=\"caf\xe9\" />\n</template>\n",
        );
        let result = read_lint_source(&path).unwrap_err();
        assert_eq!(result.error_count, 1);
        assert_eq!(result.warning_count, 0);
        assert_eq!(result.diagnostics[0].rule_name, "lint/unreadable");
        assert_eq!(
            result.diagnostics[0].message,
            format!(
                "Failed to read {}: stream did not contain valid UTF-8",
                path.display()
            )
        );
        let json = vize_patina::format_results(
            std::slice::from_ref(&result),
            &[(result.filename.clone(), String::default())],
            OutputFormat::Json,
        );
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&json).unwrap(),
            json!([{
                "file": path.to_string_lossy(),
                "messages": [{
                    "ruleId": "lint/unreadable",
                    "ruleDocsPath": "docs/content/guide/lint-execution.md",
                    "severity": 2,
                    "message": format!(
                        "[vize:lint/unreadable] Failed to read {}: stream did not contain valid UTF-8",
                        path.display()
                    ),
                    "line": 1,
                    "column": 1,
                    "endLine": 1,
                    "endColumn": 1
                }],
                "errorCount": 1,
                "warningCount": 0
            }])
        );
        let quiet = LintRunAccumulator::new(false).push((
            path,
            result.filename.clone(),
            String::default(),
            result,
        ));
        let (retained, totals) = quiet.into_parts();
        assert!(retained.is_empty());
        assert_eq!(totals, Some((1, 0)));
    }

    #[test]
    fn a_file_that_disappears_after_selection_is_an_error() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("Gone.vue");
        let error = std::fs::read_to_string(&path).unwrap_err();
        let result = read_lint_source(&path).unwrap_err();
        assert_eq!(result.error_count, 1);
        assert_eq!(
            result.diagnostics[0].message,
            format!("Failed to read {}: {error}", path.display())
        );
    }

    #[test]
    fn readable_utf8_is_returned() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("Clean.vue");
        write_bytes(&path, b"<template><img alt=\"\" /></template>\n");
        let source = read_lint_source(&path).unwrap();
        assert_eq!(source, "<template><img alt=\"\" /></template>\n");
    }
}
