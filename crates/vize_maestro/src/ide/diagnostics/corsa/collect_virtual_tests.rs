use super::{
    corsa_diagnostic_code, deduplicate_diagnostics, is_authored_vue_import_extension_diagnostic,
    is_generated_vue_ts_import_extension_diagnostic, is_inferred_implicit_any_suggestion,
};
use tower_lsp::lsp_types::{Diagnostic, NumberOrString, Position, Range};
use vize_canon::corsa_bridge::{LspDiagnostic, LspPosition, LspRange};

#[test]
fn corsa_diagnostic_codes_preserve_lsp_number_and_string_shapes() {
    assert_eq!(
        corsa_diagnostic_code(serde_json::json!(2322)),
        NumberOrString::Number(2322),
    );
    assert_eq!(
        corsa_diagnostic_code(serde_json::json!("TS2322")),
        NumberOrString::String("TS2322".to_string()),
    );
}

#[test]
fn only_ts7044_hints_are_suppressed() {
    let diagnostic = |severity, code| LspDiagnostic {
        range: LspRange {
            start: LspPosition {
                line: 0,
                character: 0,
            },
            end: LspPosition {
                line: 0,
                character: 1,
            },
        },
        severity,
        code,
        source: Some("ts".into()),
        message: "diagnostic".into(),
        related_information: None,
    };

    assert!(is_inferred_implicit_any_suggestion(&diagnostic(
        Some(4),
        Some(serde_json::json!(7044)),
    )));
    assert!(is_inferred_implicit_any_suggestion(&diagnostic(
        Some(4),
        Some(serde_json::json!("TS7044")),
    )));
    assert!(!is_inferred_implicit_any_suggestion(&diagnostic(
        Some(1),
        Some(serde_json::json!(7044)),
    )));
    assert!(!is_inferred_implicit_any_suggestion(&diagnostic(
        Some(4),
        Some(serde_json::json!(7043)),
    )));
    assert!(!is_inferred_implicit_any_suggestion(&diagnostic(
        None, None
    )));
}

#[test]
fn generated_vue_ts_import_extension_diagnostics_are_suppressed() {
    let virtual_ts = "import Child from './Child.vue.ts';\nimport plain from './plain.ts';\n";
    let diagnostic = |start, end| {
        LspDiagnostic {
        range: LspRange {
            start: LspPosition {
                line: 0,
                character: start,
            },
            end: LspPosition {
                line: 0,
                character: end,
            },
        },
        severity: Some(1),
        code: Some(serde_json::json!(5097)),
        source: Some("ts".into()),
        message: "An import path can only end with a '.ts' extension when 'allowImportingTsExtensions' is enabled."
            .into(),
        related_information: None,
    }
    };

    assert!(is_generated_vue_ts_import_extension_diagnostic(
        virtual_ts,
        &diagnostic(18, 34),
    ));

    let authored_ts_import = LspDiagnostic {
        range: LspRange {
            start: LspPosition {
                line: 1,
                character: 18,
            },
            end: LspPosition {
                line: 1,
                character: 30,
            },
        },
        ..diagnostic(18, 34)
    };
    assert!(!is_generated_vue_ts_import_extension_diagnostic(
        virtual_ts,
        &authored_ts_import,
    ));
}

#[test]
fn mapped_authored_vue_import_extension_diagnostics_are_suppressed() {
    let content = "<script setup lang=\"ts\">\nimport Child from './Child.vue'\nimport plain from './plain.ts'\n</script>\n";
    let diagnostic = LspDiagnostic {
        range: LspRange {
            start: LspPosition {
                line: 0,
                character: 0,
            },
            end: LspPosition {
                line: 0,
                character: 1,
            },
        },
        severity: Some(1),
        code: Some(serde_json::json!("TS5097")),
        source: Some("ts".into()),
        message: "An import path can only end with a '.ts' extension when 'allowImportingTsExtensions' is enabled."
            .into(),
        related_information: None,
    };

    assert!(is_authored_vue_import_extension_diagnostic(
        content,
        &diagnostic,
        1,
        18,
        1,
        31,
    ));
    assert!(!is_authored_vue_import_extension_diagnostic(
        content,
        &diagnostic,
        2,
        18,
        2,
        30,
    ));
    assert!(!is_authored_vue_import_extension_diagnostic(
        "import authored from './Child.vue.ts'\n",
        &diagnostic,
        0,
        21,
        0,
        37,
    ));
}

#[test]
fn exact_diagnostics_are_stably_deduplicated() {
    let original = Diagnostic {
        range: Range {
            start: Position {
                line: 1,
                character: 19,
            },
            end: Position {
                line: 1,
                character: 30,
            },
        },
        severity: Some(tower_lsp::lsp_types::DiagnosticSeverity::ERROR),
        code: Some(NumberOrString::Number(2304)),
        source: Some("vize/types".into()),
        message: "Cannot find name 'missingList'.".into(),
        ..Default::default()
    };
    let distinct = Diagnostic {
        message: "Cannot find name 'anotherBinding'.".into(),
        ..original.clone()
    };
    let distinct_data = Diagnostic {
        data: Some(serde_json::json!({ "origin": "second-pass" })),
        ..original.clone()
    };

    assert_eq!(
        deduplicate_diagnostics(vec![
            original.clone(),
            distinct.clone(),
            original.clone(),
            distinct.clone(),
            distinct_data.clone(),
            distinct_data.clone(),
        ]),
        vec![original, distinct, distinct_data],
    );
}
