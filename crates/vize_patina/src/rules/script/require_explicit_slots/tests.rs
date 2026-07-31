use super::RequireExplicitSlots;
use crate::diagnostic::Severity;
use crate::linter::{LintResult, Linter};
use crate::rules::script::ScriptLinter;

// Template-half tests (#3414 B) live in the `template` submodule.
mod template;

fn create_linter() -> ScriptLinter {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(RequireExplicitSlots));
    linter
}

/// Lint a full SFC end-to-end with only this rule enabled, exercising the
/// engine path that supplies the parsed `<template>` AST to the rule.
fn lint_sfc(sfc: &str) -> LintResult {
    Linter::new()
        .with_enabled_rules(Some(vec!["script/require-explicit-slots".into()]))
        .lint_sfc(sfc, "Probe.vue")
}

/// The full identity of every finding: rule, severity, byte range, message.
fn findings(result: &LintResult) -> Vec<(&'static str, Severity, u32, u32, &str)> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.rule_name,
                diagnostic.severity,
                diagnostic.start,
                diagnostic.end,
                diagnostic.message.as_str(),
            )
        })
        .collect()
}

fn none() -> Vec<(&'static str, Severity, u32, u32, &'static str)> {
    Vec::new()
}

/// [`findings`] with owned messages, so it compares against [`undeclared`].
fn owned(result: &LintResult) -> Vec<(&'static str, Severity, u32, u32, std::string::String)> {
    findings(result)
        .into_iter()
        .map(|(rule, severity, start, end, message)| {
            (rule, severity, start, end, message.to_string())
        })
        .collect()
}

/// The finding the `<slot>` written as `tag` produces, as the full tuple.
fn undeclared(
    sfc: &str,
    tag: &str,
    name: &str,
) -> (&'static str, Severity, u32, u32, std::string::String) {
    let start = sfc.find(tag).expect("slot start tag");
    (
        "script/require-explicit-slots",
        Severity::Warning,
        start as u32,
        (start + tag.len()) as u32,
        format!(
            "Slot '{name}' is rendered in the template but not declared in defineSlots<...>()."
        ),
    )
}

#[test]
fn test_invalid_use_slots_without_define_slots() {
    // TS syntax present (defineProps<T>()), useSlots() consumed, no defineSlots.
    let result = create_linter().lint(
        "const props = defineProps<{ id: number }>()\nconst slots = useSlots()",
        0,
    );
    assert_eq!(result.warning_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_use_slots_with_type_annotation() {
    // TS signalled by a plain type annotation.
    let result = create_linter().lint("const n: number = 1\nconst slots = useSlots()", 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_invalid_use_slots_with_interface() {
    let result = create_linter().lint("interface Props { id: number }\nconst s = useSlots()", 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_invalid_use_slots_called_without_assignment() {
    // A bare `useSlots()` expression statement still counts as consumption.
    let result = create_linter().lint("const x: string = ''\nuseSlots()", 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_valid_define_slots_typed() {
    let result = create_linter().lint(
        "defineSlots<{ default(props: { msg: string }): unknown }>()\nconst slots = useSlots()",
        0,
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_define_slots_alone_satisfies() {
    // `defineSlots` present (even before `useSlots`) means slots are declared.
    let result = create_linter().lint(
        "const props = defineProps<{ id: number }>()\ndefineSlots<{ default(): unknown }>()\nconst slots = useSlots()",
        0,
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_no_use_slots() {
    // No slot consumption at all.
    let result = create_linter().lint("const props = defineProps<{ id: number }>()", 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_javascript_use_slots() {
    // No TypeScript syntax anywhere => treated as JS, not flagged. In JS the
    // type-only `defineSlots<T>()` fix is not available, so flagging would be
    // a false positive.
    let result = create_linter().lint("const slots = useSlots()", 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_javascript_use_slots_with_props() {
    // Runtime `defineProps([...])` carries no TS syntax => still JS.
    let result = create_linter().lint(
        "const props = defineProps(['id'])\nconst slots = useSlots()",
        0,
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_define_slots_typed_only_ts_token() {
    // The only TS token is the `defineSlots<T>()` type argument; since
    // `defineSlots` is present the block is valid regardless.
    let result = create_linter().lint(
        "defineSlots<{ header(): unknown }>()\nconst slots = useSlots()",
        0,
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_no_warn_use_slots_in_string_literal() {
    // The pattern inside a string literal must not be flagged.
    let result = create_linter().lint("const code: string = \"const s = useSlots()\"", 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_multiple_use_slots_reports_once_at_first() {
    let source = "const x: number = 0\nconst a = useSlots()\nconst b = useSlots()";
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
    let first = source.find("useSlots()").unwrap() as u32;
    assert_eq!(result.diagnostics[0].start, first);
}

#[test]
fn test_offset_applied() {
    let source = "const x: number = 0\nconst slots = useSlots()";
    let result = create_linter().lint(source, 100);
    assert_eq!(result.warning_count, 1);
    let call_start = source.find("useSlots()").unwrap() as u32 + 100;
    assert_eq!(result.diagnostics[0].start, call_start);
}
