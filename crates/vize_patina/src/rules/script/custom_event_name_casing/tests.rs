use super::{CustomEventNameCasing, EventNameCasing};
use crate::diagnostic::Severity;
use crate::linter::{LintResult, Linter};
use crate::rules::script::ScriptLinter;

// Template-half tests (#3413 B) live in the `template` submodule.
mod template;

fn create_linter() -> ScriptLinter {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(CustomEventNameCasing::default()));
    linter
}

fn create_kebab_linter() -> ScriptLinter {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(CustomEventNameCasing::new(
        EventNameCasing::KebabCase,
    )));
    linter
}

/// Lint a full SFC end-to-end with only this rule enabled, exercising the
/// engine path that supplies the parsed `<template>` AST to the rule.
fn lint_sfc(sfc: &str) -> LintResult {
    Linter::new()
        .with_enabled_rules(Some(vec!["script/custom-event-name-casing".into()]))
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
fn test_valid_camel_case_setup_emit() {
    let source = r#"
const emit = defineEmits(['myEvent'])
emit('myEvent')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_single_word_event() {
    let source = r#"
const emit = defineEmits(['change'])
emit('change')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_update_model_value() {
    // The `update:` prefix used by `v-model` is permitted.
    let result = create_linter().lint("this.$emit('update:modelValue', value)", 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_invalid_kebab_case_setup_emit() {
    let source = r#"
const emit = defineEmits(['my-event'])
emit('my-event')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_configured_kebab_case_allows_kebab_emit() {
    let source = r#"
const emit = defineEmits(['my-event'])
emit('my-event')
"#;
    let result = create_kebab_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_configured_kebab_case_reports_camel_emit() {
    let source = r#"
const emit = defineEmits(['myEvent'])
emit('myEvent')
"#;
    let result = create_kebab_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
    assert_eq!(
        result.diagnostics[0].message.as_str(),
        "Custom event name 'myEvent' is not kebab-case."
    );
}

#[test]
fn test_invalid_pascal_case_dollar_emit() {
    let result = create_linter().lint("this.$emit('MyEvent')", 0);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_kebab_case_dollar_emit() {
    let result = create_linter().lint("this.$emit('my-event')", 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_context_emit_kebab() {
    // A setup-context member call (`ctx.emit('...')`) is checked too.
    let result = create_linter().lint("ctx.emit('my-event')", 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_valid_context_emit_camel() {
    let result = create_linter().lint("ctx.emit('myEvent')", 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_dynamic_event_name_not_checked() {
    // A non-string-literal event name carries no literal to inspect.
    let source = r#"
const emit = defineEmits(['myEvent'])
const name = 'my-event'
emit(name)
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_unassigned_define_emits_call_not_tracked() {
    // Without a binding the `emit(...)` identifier cannot be resolved, so the
    // bare `emit` identifier call is not treated as an emit.
    let source = r#"
defineEmits(['my-event'])
emit('my-event')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_custom_emit_binding_name() {
    let source = r#"
const myEmit = defineEmits(['change'])
myEmit('my-event')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_options_api_this_emit_in_method() {
    let source = r#"
export default {
  methods: {
submit() {
  this.$emit('my-event')
}
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_plain_identifier_call_not_emit() {
    // A call to some other function is not an emit, even with a kebab string.
    let result = create_linter().lint("notify('my-event')", 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_multiple_invalid_events() {
    let source = r#"
const emit = defineEmits(['my-event', 'OtherEvent'])
emit('my-event')
emit('OtherEvent')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 2);
}

#[test]
fn test_offset_applied() {
    let result = create_linter().lint("this.$emit('my-event')", 30);
    assert_eq!(result.error_count, 1);
    assert_eq!(result.diagnostics[0].start, 30 + 11);
}

#[test]
fn test_configured_kebab_case_allows_template_emit() {
    let sfc = r#"<script setup>
const emit = defineEmits(['my-event'])
</script>
<template>
  <button @click="emit('my-event')" />
</template>"#;
    let result = Linter::new()
        .with_enabled_rules(Some(vec!["script/custom-event-name-casing".into()]))
        .with_custom_event_name_casing(EventNameCasing::KebabCase)
        .lint_sfc(sfc, "Probe.vue");
    assert_eq!(findings(&result), none());
}
