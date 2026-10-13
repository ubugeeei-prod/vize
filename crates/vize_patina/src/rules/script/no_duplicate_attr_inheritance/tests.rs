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

#[test]
fn no_attrs_in_an_sfc_is_healthy_even_with_explicit_true() {
    let source = r#"<script setup>defineOptions({ inheritAttrs: true })</script><template><div /></template>"#;
    assert_eq!(findings(&lint_sfc(source)), none());
}

#[test]
fn nested_and_root_forwarding_have_distinct_identifier_ranges() {
    let source = r#"<script setup></script><template><div v-bind="$attrs"><span v-bind="$attrs" /></div></template>"#;
    let result = lint_sfc(source);
    let actual = findings(&result);
    let starts: Vec<_> = source
        .match_indices("$attrs")
        .map(|(start, _)| start as u32)
        .collect();
    assert_eq!(
        actual,
        starts
            .into_iter()
            .map(|start| (
                "script/no-duplicate-attr-inheritance",
                Severity::Warning,
                start,
                start + 6,
                "Set \"inheritAttrs\" to false.",
            ))
            .collect::<Vec<_>>()
    );
}

#[test]
fn foreign_template_and_expression_bindings_are_not_vm_attrs() {
    for body in [
        r#"<div><span v-for="$attrs in rows" v-bind="$attrs" /></div>"#,
        r#"<div><span v-for="$\u0061ttrs in rows" v-bind="$attrs" /></div>"#,
        r#"<Comp v-slot="$attrs"><span v-bind="$attrs" /></Comp>"#,
        r#"<Comp v-slot="$attrs" v-bind="$attrs"><span /></Comp>"#,
        r#"<Comp v-slot="{ value: $attrs }"><span v-bind="$attrs" /></Comp>"#,
        r#"<div v-bind="(($attrs) => $attrs)(local)" />"#,
        r#"<div v-bind="{ '$attrs': local }" />"#,
    ] {
        let source = format!("<script setup></script><template>{body}</template>");
        assert_eq!(findings(&lint_sfc(&source)), none(), "{body}");
    }
}

#[test]
fn renaming_an_attrs_slot_key_does_not_shadow_the_vm_reference() {
    let source = r#"<script setup></script><template><Comp v-slot="{ $attrs: local }"><div v-bind="$attrs" /></Comp></template>"#;
    let start = source.rfind("$attrs").unwrap() as u32;
    assert_eq!(
        findings(&lint_sfc(source)),
        vec![(
            "script/no-duplicate-attr-inheritance",
            Severity::Warning,
            start,
            start + 6,
            "Set \"inheritAttrs\" to false.",
        )]
    );
}

#[test]
fn conditional_root_branches_each_keep_their_forwarding_reference() {
    let source = r#"<script setup></script><template><div v-if="ok" v-bind="$attrs" /><span v-else v-bind="$attrs" /></template>"#;
    let starts: Vec<_> = source
        .match_indices("$attrs")
        .map(|(start, _)| start as u32)
        .collect();
    assert_eq!(
        findings(&lint_sfc(source)),
        starts
            .into_iter()
            .map(|start| (
                "script/no-duplicate-attr-inheritance",
                Severity::Warning,
                start,
                start + 6,
                "Set \"inheritAttrs\" to false.",
            ))
            .collect::<Vec<_>>()
    );
}

#[test]
fn scriptless_sfc_reports_the_original_gui_forwarding_reference() {
    let source = "<template>\n  <Select v-bind=\"$attrs\" multiple />\n</template>\n";
    assert_eq!(
        findings(&lint_sfc(source)),
        vec![(
            "script/no-duplicate-attr-inheritance",
            Severity::Warning,
            29,
            35,
            "Set \"inheritAttrs\" to false.",
        )]
    );
}

struct ScriptNamesake;

impl crate::rules::script::ScriptRule for ScriptNamesake {
    fn meta(&self) -> &'static crate::rules::script::ScriptRuleMeta {
        &super::META
    }
    fn uses_ast(&self) -> bool {
        true
    }
    fn uses_template_ast(&self) -> bool {
        true
    }
    fn check_program_with_sfc<'a>(
        &self,
        _program: &'a oxc_ast::ast::Program<'a>,
        _source: &str,
        offset: usize,
        _sfc: crate::rules::script::SfcScriptContext<'_>,
        result: &mut crate::rules::script::ScriptLintResult,
    ) {
        result.add_diagnostic(crate::diagnostic::LintDiagnostic::warn(
            super::META.name,
            "configured script callback",
            offset as u32,
            offset as u32 + 1,
        ));
    }
}

#[test]
fn script_namesake_has_no_scriptless_capability_and_keeps_ordinary_dispatch() {
    let mut linter = Linter::new().with_enabled_rules(Some(vec![super::META.name.into()]));
    linter
        .script_rule_overrides
        .insert(super::META.name, Box::new(ScriptNamesake));
    assert_eq!(
        findings(&linter.lint_sfc(
            "<template><div v-bind=\"$attrs\" /></template>",
            "Probe.vue"
        )),
        none()
    );
    let result = linter.lint_sfc(
        "<script setup>const value = 1;</script><template><div /></template>",
        "Probe.vue",
    );
    assert_eq!(
        findings(&result),
        vec![(
            super::META.name,
            Severity::Warning,
            14,
            15,
            "configured script callback"
        )]
    );
}
