//! Failure text for native type-aware assertions.
//!
//! A missing or unexpected rule still prints every returned diagnostic.
//! `type/corsa-runtime` is `TypeAwareFailure::Runtime` or `TypeAwareFailure::Query`;
//! the diagnostic message is the concrete failure. Expected rules and negative
//! controls stay the same: a runtime diagnostic does not satisfy another rule,
//! and it does not fail a negative control for a different rule.

use crate::{LintDiagnostic, LintResult, Severity};
use vize_l0::{String, append, cstr};

const RUNTIME_RULE: &str = "type/corsa-runtime";
const RUNTIME_HELP: &str = "Requested type-aware lint could not initialize its native runtime.";
const QUERY_HELP: &str = "Type-aware lint could not complete with the selected native runtime.";

/// Complete returned diagnostics, plus concrete runtime and query failure text.
pub(in crate::linter::native_type_aware) fn failure_context(result: &LintResult) -> String {
    let mut text = cstr!(
        "returned diagnostics for {} (errors={}, warnings={}): {:#?}",
        result.filename,
        result.error_count,
        result.warning_count,
        result.diagnostics
    );
    let runtime = runtime_failures(result);
    if !runtime.is_empty() {
        text.push_str("\nruntime/query failure:");
        text.push_str(&runtime);
    }
    text
}

#[track_caller]
pub(in crate::linter::native_type_aware) fn assert_reports(result: &LintResult, rule: &str) {
    assert_rule(result, rule, true, None);
}

#[track_caller]
pub(in crate::linter::native_type_aware) fn assert_reports_note(
    result: &LintResult,
    rule: &str,
    note: &str,
) {
    assert_rule(result, rule, true, Some(note));
}

#[track_caller]
pub(in crate::linter::native_type_aware) fn assert_absent(result: &LintResult, rule: &str) {
    assert_rule(result, rule, false, None);
}

#[track_caller]
pub(in crate::linter::native_type_aware) fn assert_absent_note(
    result: &LintResult,
    rule: &str,
    note: &str,
) {
    assert_rule(result, rule, false, Some(note));
}

#[track_caller]
pub(in crate::linter::native_type_aware) fn assert_reports_message(
    result: &LintResult,
    rule: &str,
    fragment: &str,
) {
    let found = result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.rule_name == rule && diagnostic.message.contains(fragment));
    if found {
        return;
    }
    let context = failure_context(result);
    panic!("expected `{rule}` containing {fragment:?}\n{context}");
}

#[track_caller]
pub(in crate::linter::native_type_aware) fn assert_absent_rules(
    result: &LintResult,
    rules: &[&str],
) {
    let found = result
        .diagnostics
        .iter()
        .any(|diagnostic| rules.contains(&diagnostic.rule_name));
    if !found {
        return;
    }
    let context = failure_context(result);
    panic!("expected none of {rules:?}\n{context}");
}

#[track_caller]
fn assert_rule(result: &LintResult, rule: &str, present: bool, note: Option<&str>) {
    let found = result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.rule_name == rule);
    if found == present {
        return;
    }
    let expectation = if present {
        cstr!("expected diagnostic `{rule}`")
    } else {
        cstr!("did not expect diagnostic `{rule}`")
    };
    let note = note.map(|note| cstr!("\n{note}")).unwrap_or_default();
    let context = failure_context(result);
    panic!("{expectation}{note}\n{context}");
}

fn runtime_failures(result: &LintResult) -> String {
    let mut text = String::default();
    for diagnostic in result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == RUNTIME_RULE)
    {
        let kind = failure_kind(diagnostic.help.as_deref());
        append!(text, "\n  {kind}: {}", diagnostic.message);
    }
    text
}

fn failure_kind(help: Option<&str>) -> &'static str {
    match help {
        Some(help) if help.starts_with(RUNTIME_HELP) => "TypeAwareFailure::Runtime",
        Some(help) if help.starts_with(QUERY_HELP) => "TypeAwareFailure::Query",
        _ => RUNTIME_RULE,
    }
}

fn fixture(diagnostics: Vec<LintDiagnostic>) -> LintResult {
    let errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .count();
    let warnings = diagnostics.len() - errors;
    LintResult {
        filename: "Component.vue".into(),
        diagnostics,
        error_count: errors,
        warning_count: warnings,
    }
}

fn runtime(message: &str) -> LintDiagnostic {
    LintDiagnostic::error(RUNTIME_RULE, message, 0, 0).with_help(
        "Requested type-aware lint could not initialize its native runtime. Check the reported failure, install a compatible native TypeScript runtime, or configure `typeChecker.corsaPath`, `CORSA_PATH`, or `TSGO_PATH` to its executable.",
    )
}

fn query(message: &str) -> LintDiagnostic {
    LintDiagnostic::error(RUNTIME_RULE, message, 0, 0).with_help(
        "Type-aware lint could not complete with the selected native runtime. Use a compatible runtime or configure `typeChecker.corsaPath`, `CORSA_PATH`, or `TSGO_PATH` to its executable.",
    )
}

fn panic_text(check: impl FnOnce()) -> std::string::String {
    let error = std::panic::catch_unwind(std::panic::AssertUnwindSafe(check))
        .expect_err("assertion should fail");
    if let Some(message) = error.downcast_ref::<std::string::String>() {
        return message.clone();
    }
    if let Some(message) = error.downcast_ref::<&str>() {
        return (*message).to_owned();
    }
    panic!("unexpected panic payload");
}

#[test]
fn native_type_aware_assertion_prints_the_complete_diagnostic_set() {
    let returned = fixture(vec![
        LintDiagnostic::warn("type/no-floating-promises", "floating call", 12, 20)
            .with_label("call", 12, 20),
        runtime("checker exited: socket closed"),
    ]);
    let context = failure_context(&returned);
    for fragment in [
        "Component.vue",
        "errors=1",
        "warnings=1",
        "type/no-floating-promises",
        "Warning",
        "floating call",
        "start: 12",
        "end: 20",
        "labels:",
        "call",
        "help:",
        "fix:",
        "type/corsa-runtime",
        "checker exited: socket closed",
        "TypeAwareFailure::Runtime",
        RUNTIME_HELP,
    ] {
        assert!(context.contains(fragment), "{context}");
    }
}

#[test]
fn native_type_aware_assertion_captures_query_failure_text() {
    let context = failure_context(&fixture(vec![query(
        "symbol owner is unknown to this connection",
    )]));
    assert!(context.contains("TypeAwareFailure::Query"), "{context}");
    assert!(
        context.contains("symbol owner is unknown to this connection"),
        "{context}"
    );
    assert!(!context.contains("TypeAwareFailure::Runtime"), "{context}");
}

#[test]
fn native_type_aware_assertion_labels_unclassified_runtime_text() {
    let context = failure_context(&fixture(vec![LintDiagnostic::error(
        RUNTIME_RULE,
        "plain runtime text",
        0,
        0,
    )]));
    assert!(
        context.contains("type/corsa-runtime: plain runtime text"),
        "{context}"
    );
    assert!(!context.contains("TypeAwareFailure::Runtime"), "{context}");
    assert!(!context.contains("TypeAwareFailure::Query"), "{context}");
}

#[test]
fn native_type_aware_assertion_prints_an_empty_returned_set() {
    let returned = fixture(vec![]);
    let context = failure_context(&returned);
    assert!(context.contains("errors=0"), "{context}");
    assert!(context.contains("warnings=0"), "{context}");
    assert!(context.contains("[]"), "{context}");
    assert!(!context.contains("runtime/query failure"), "{context}");
    assert_absent(&returned, "type/no-floating-promises");
}

#[test]
fn native_type_aware_assertion_keeps_expected_rule_and_negative_control() {
    let returned = fixture(vec![
        LintDiagnostic::warn("type/no-floating-promises", "floating call", 4, 12),
        runtime("checker exited: late"),
    ]);
    assert_reports(&returned, "type/no-floating-promises");
    assert_reports_note(
        &returned,
        "type/no-floating-promises",
        "control flow still reports",
    );
    assert_reports_message(&returned, "type/no-floating-promises", "floating call");
    assert_absent(&returned, "type/no-unsafe-template-binding");
    assert_absent_rules(
        &returned,
        &["type/require-typed-props", "type/require-typed-emits"],
    );

    let missing = fixture(vec![runtime("checker exited: hidden")]);
    let message = panic_text(|| assert_reports(&missing, "type/no-floating-promises"));
    assert!(
        message.contains("expected diagnostic `type/no-floating-promises`"),
        "{message}"
    );
    assert!(
        message.contains("TypeAwareFailure::Runtime: checker exited: hidden"),
        "{message}"
    );
    assert!(message.contains("type/corsa-runtime"), "{message}");
    assert!(message.contains("checker exited: hidden"), "{message}");

    let unexpected = fixture(vec![LintDiagnostic::warn(
        "type/no-floating-promises",
        "floating call",
        1,
        2,
    )]);
    let message = panic_text(|| {
        assert_absent_note(
            &unexpected,
            "type/no-floating-promises",
            "handled chains stay quiet",
        );
    });
    assert!(
        message.contains("did not expect diagnostic `type/no-floating-promises`"),
        "{message}"
    );
    assert!(message.contains("handled chains stay quiet"), "{message}");
    assert!(message.contains("floating call"), "{message}");

    let message = panic_text(|| {
        assert_reports_note(
            &missing,
            "type/no-floating-promises",
            "control flow still reports",
        );
    });
    assert!(message.contains("control flow still reports"), "{message}");
    assert!(
        message.contains("TypeAwareFailure::Runtime: checker exited: hidden"),
        "{message}"
    );

    let message = panic_text(|| {
        assert_reports_message(
            &unexpected,
            "type/no-floating-promises",
            "Template interpolation",
        );
    });
    assert!(message.contains("Template interpolation"), "{message}");
    assert!(message.contains("floating call"), "{message}");

    let message = panic_text(|| {
        assert_absent_rules(
            &unexpected,
            &[
                "type/no-floating-promises",
                "type/no-unsafe-template-binding",
            ],
        );
    });
    assert!(message.contains("expected none of"), "{message}");
    assert!(message.contains("type/no-floating-promises"), "{message}");
    assert!(message.contains("floating call"), "{message}");
}
