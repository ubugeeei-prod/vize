use super::{
    TYPECHECK_UNAVAILABLE_HINT_MESSAGE, TYPECHECK_UNAVAILABLE_NOTICE_MESSAGE,
    service::typecheck_unavailable_hint, sources,
};
use tower_lsp::lsp_types::{DiagnosticSeverity, NumberOrString};

#[test]
fn unavailable_typecheck_diagnostic_points_to_typescript_7() {
    let hint = typecheck_unavailable_hint();

    assert_eq!(hint.source.as_deref(), Some(sources::TYPE_CHECKER));
    assert_eq!(hint.severity, Some(DiagnosticSeverity::HINT));
    assert_eq!(
        hint.code,
        Some(NumberOrString::String("typecheck-unavailable".to_string()))
    );
    assert_eq!(hint.message, TYPECHECK_UNAVAILABLE_HINT_MESSAGE);
    assert_ts7_guidance(&hint.message);
}

#[test]
fn unavailable_typecheck_notice_points_to_typescript_7() {
    assert_ts7_guidance(TYPECHECK_UNAVAILABLE_NOTICE_MESSAGE);
}

fn assert_ts7_guidance(message: &str) {
    assert!(message.contains("typescript@^7"));
    assert!(message.contains("typeChecker.corsaPath"));
    assert!(!message.contains("@typescript/native-preview"));
}
