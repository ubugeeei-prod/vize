use super::NoDuplicateAttrInheritance;
use crate::diagnostic::Severity;
use crate::linter::{LintResult, Linter};
use crate::rules::script::ScriptLinter;

// Template-half tests (#3414 C) live in the `template` submodule.
mod template;

fn create_linter() -> ScriptLinter {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(NoDuplicateAttrInheritance));
    linter
}

/// Lint a full SFC end-to-end with only this rule enabled, exercising the
/// engine path that supplies the parsed `<template>` AST to the rule.
fn lint_sfc(sfc: &str) -> LintResult {
    Linter::new()
        .with_enabled_rules(Some(vec!["script/no-duplicate-attr-inheritance".into()]))
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

#[test]
fn test_invalid_define_options_inherit_attrs_true() {
    let result = create_linter().lint("defineOptions({ inheritAttrs: true })", 0);
    assert_eq!(result.warning_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_export_default_inherit_attrs_true() {
    let result = create_linter().lint("export default { inheritAttrs: true }", 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_invalid_define_component_inherit_attrs_true() {
    let result = create_linter().lint(
        "import { defineComponent } from 'vue'\n\
             export default defineComponent({ name: 'Foo', inheritAttrs: true })",
        0,
    );
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_invalid_identifier_bound_options() {
    let result = create_linter().lint(
        "const options = { inheritAttrs: true }\nexport default options",
        0,
    );
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_invalid_string_literal_key() {
    let result = create_linter().lint("defineOptions({ 'inheritAttrs': true })", 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_invalid_satisfies_wrapper() {
    let result = create_linter().lint(
        "export default { inheritAttrs: true } satisfies Record<string, unknown>",
        0,
    );
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_valid_inherit_attrs_false() {
    let result = create_linter().lint("defineOptions({ inheritAttrs: false })", 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_export_default_inherit_attrs_false() {
    let result = create_linter().lint("export default { inheritAttrs: false }", 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_no_inherit_attrs_option() {
    let result = create_linter().lint("defineOptions({ name: 'Foo' })", 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_empty_options() {
    let result = create_linter().lint("export default {}", 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_non_literal_value() {
    // A computed / variable value is not a literal `true`; do not guess.
    let result = create_linter().lint(
        "const inherit = true\nexport default { inheritAttrs: inherit }",
        0,
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_computed_key() {
    let result = create_linter().lint(
        "const key = 'inheritAttrs'\nexport default { [key]: true }",
        0,
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_no_default_export() {
    let result = create_linter().lint("const inheritAttrs = true", 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_offset_applied() {
    let source = "defineOptions({ inheritAttrs: true })";
    let result = create_linter().lint(source, 40);
    assert_eq!(result.warning_count, 1);
    let true_start = source.find("true").unwrap() as u32 + 40;
    assert_eq!(result.diagnostics[0].start, true_start);
}
