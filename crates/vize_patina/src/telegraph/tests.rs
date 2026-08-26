use super::{
    Emitter, FormatEmitter, LintResult, LintTransmission, LspEmitter, Telegraph, offset_to_line_col,
};
use crate::diagnostic::LintDiagnostic;
use crate::output::OutputFormat;
use vize_s0::ToCompactString;

#[test]
fn test_telegraph_with_text() {
    let telegraph = Telegraph::with_text();
    assert_eq!(telegraph.len(), 1);
}

#[test]
fn test_telegraph_with_json() {
    let telegraph = Telegraph::with_json();
    assert_eq!(telegraph.len(), 1);
}

#[test]
fn test_telegraph_with_format() {
    let telegraph = Telegraph::with_format(OutputFormat::Markdown);
    assert_eq!(telegraph.len(), 1);
}

#[test]
fn test_format_emitter_transmit_all_renders_single_report() {
    let mut telegraph = Telegraph::new();
    telegraph.add_emitter(Box::new(FormatEmitter::new(OutputFormat::Html)));
    let result = LintResult {
        filename: "test.vue".to_compact_string(),
        diagnostics: vec![LintDiagnostic::warn(
            "vue/no-v-html",
            "Avoid raw HTML",
            0,
            3,
        )],
        error_count: 0,
        warning_count: 1,
    };
    let outputs = telegraph.transmit_all(&[(result, "abc".to_compact_string())]);

    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0].matches("<!doctype html>").count(), 1);
    assert!(outputs[0].contains("docs/content/rules/vue.md"));
}

#[test]
fn test_lsp_diagnostic_conversion() {
    let result = LintResult {
        filename: "test.vue".to_compact_string(),
        diagnostics: vec![
            LintDiagnostic::error("vue/require-v-for-key", "Missing key", 50, 70)
                .with_help("Add :key attribute"),
        ],
        error_count: 1,
        warning_count: 0,
    };

    let lsp_diagnostics = LspEmitter::to_lsp_diagnostics(&result);
    assert_eq!(lsp_diagnostics.len(), 1);
    assert_eq!(lsp_diagnostics[0].severity, 1);
    assert_eq!(lsp_diagnostics[0].code, "vue/require-v-for-key");
}

#[test]
fn test_lsp_diagnostic_with_source() {
    let source = "line1\nline2\nline3 v-for=\"item in items\"";
    let result = LintResult {
        filename: "test.vue".to_compact_string(),
        diagnostics: vec![LintDiagnostic::error(
            "vue/require-v-for-key",
            "Missing key",
            18,
            44,
        )],
        error_count: 1,
        warning_count: 0,
    };

    let lsp_diagnostics = LspEmitter::to_lsp_diagnostics_with_source(&result, source);
    assert_eq!(lsp_diagnostics.len(), 1);
    assert_eq!(lsp_diagnostics[0].range.start.line, 2);
}

#[test]
fn test_lsp_emitter_emit_uses_source_for_line_column() {
    let source = "line1\nline2\nline3 v-for=\"item in items\"";
    let result = LintResult {
        filename: "test.vue".to_compact_string(),
        diagnostics: vec![LintDiagnostic::error(
            "vue/require-v-for-key",
            "Missing key",
            18,
            44,
        )],
        error_count: 1,
        warning_count: 0,
    };
    let transmission = LintTransmission::new(result, source);

    let json = LspEmitter.emit(&transmission);
    assert!(
        json.contains("\"line\": 2"),
        "expected emit() to report line 2, got: {json}"
    );
    assert!(!json.contains("\"character\": 18"));
}

#[test]
fn test_lsp_emitter_emit_all_renders_single_json_array() {
    let first = LintResult {
        filename: "first.vue".to_compact_string(),
        diagnostics: vec![LintDiagnostic::error("vue/first", "First", 4, 7)],
        error_count: 1,
        warning_count: 0,
    };
    let second = LintResult {
        filename: "second.vue".to_compact_string(),
        diagnostics: vec![LintDiagnostic::warn("vue/second", "Second", 3, 5)],
        error_count: 0,
        warning_count: 1,
    };
    let transmissions = [
        LintTransmission::new(first, "one\nbad"),
        LintTransmission::new(second, "xy\nzz"),
    ];

    let json = LspEmitter.emit_all(&transmissions);
    let parsed: serde_json::Value =
        serde_json::from_str(&json).expect("emit_all should return valid JSON");
    let diagnostics = parsed.as_array().expect("LSP output should be an array");

    assert_eq!(diagnostics.len(), 2);
    assert_eq!(diagnostics[0]["code"], "vue/first");
    assert_eq!(diagnostics[0]["range"]["start"]["line"], 1);
    assert_eq!(diagnostics[0]["range"]["start"]["character"], 0);
    assert_eq!(diagnostics[1]["code"], "vue/second");
    assert_eq!(diagnostics[1]["range"]["start"]["line"], 1);
    assert_eq!(diagnostics[1]["range"]["start"]["character"], 0);
    assert!(
        !json.contains("]["),
        "emit_all must not concatenate multiple JSON arrays: {json}"
    );
}

#[test]
fn test_offset_to_line_col() {
    let source = "abc\ndef\nghi";
    assert_eq!(offset_to_line_col(source, 0), (0, 0));
    assert_eq!(offset_to_line_col(source, 3), (0, 3));
    assert_eq!(offset_to_line_col(source, 4), (1, 0));
    assert_eq!(offset_to_line_col(source, 8), (2, 0));
}

#[test]
fn test_offset_to_line_col_counts_utf16_code_units() {
    let source = "a\u{1F980}b\nc";
    let crab_offset = "a".len();
    let b_offset = "a\u{1F980}".len();
    let newline_offset = "a\u{1F980}b".len();
    let c_offset = "a\u{1F980}b\n".len();

    assert_eq!(offset_to_line_col(source, crab_offset), (0, 1));
    assert_eq!(offset_to_line_col(source, b_offset), (0, 3));
    assert_eq!(offset_to_line_col(source, newline_offset), (0, 4));
    assert_eq!(offset_to_line_col(source, c_offset), (1, 0));
}
