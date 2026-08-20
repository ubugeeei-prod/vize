use super::NoUnusedProperties;
use crate::diagnostic::Severity;
use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleCategory};

// The positive direction (a prop that must be reported) lives in the `reports`
// submodule; this file keeps the helpers and the silent-direction cases.
mod directive_lexer_reports;
mod directive_lexer_statement_blocks;
mod directive_lexer_tsx_multiline;
mod model_modifiers;
mod reports;

/// Lint a full SFC with only this rule enabled.
///
/// `vue/no-unused-properties` is a member of `SEMANTIC_TEMPLATE_RULES` and of
/// `SHARED_SFC_DESCRIPTOR_RULES`, so enabling it alone still produces both the
/// croquis analysis and the SFC descriptor the rule reads.
fn lint_sfc(sfc: &str) -> LintResult {
    Linter::new()
        .with_enabled_rules(Some(vec!["vue/no-unused-properties".into()]))
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

/// The finding an unused `name` produces at its exact written declaration.
fn unused(
    sfc: &str,
    name: &str,
    declaration: &str,
) -> (&'static str, Severity, u32, u32, std::string::String) {
    // A declaration that appears twice would make the expected range ambiguous,
    // so the first match is only trustworthy when it is the only one.
    assert_eq!(
        sfc.matches(declaration).count(),
        1,
        "declaration {declaration:?} must occur exactly once"
    );
    let start = sfc.find(declaration).expect("prop declaration") as u32;
    (
        "vue/no-unused-properties",
        Severity::Warning,
        start,
        start + declaration.len() as u32,
        format!("Prop '{name}' is defined but never used"),
    )
}

/// [`findings`] with owned messages, so it compares against [`unused`].
fn owned(result: &LintResult) -> Vec<(&'static str, Severity, u32, u32, std::string::String)> {
    findings(result)
        .into_iter()
        .map(|(rule, severity, start, end, message)| {
            (rule, severity, start, end, message.to_string())
        })
        .collect()
}

#[test]
fn test_meta() {
    let rule = NoUnusedProperties::default();
    assert_eq!(rule.meta().name, "vue/no-unused-properties");
    assert_eq!(rule.meta().category, RuleCategory::StronglyRecommended);
}

#[test]
fn test_should_ignore() {
    let rule = NoUnusedProperties::default();
    assert!(rule.should_ignore("_internal"));
    assert!(!rule.should_ignore("count"));
}

// --- The prop is referenced: exactly zero findings -------------------------

#[test]
fn ignores_a_prop_read_in_an_interpolation() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string }>();
</script>

<template>
  <div>{{ msg }}</div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_prop_read_in_a_directive_expression() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string }>();
</script>

<template>
  <div :title="msg"></div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_prop_read_only_inside_a_v_for_body() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string; rows: string[] }>();
</script>

<template>
  <ul><li v-for="row in rows" :key="row">{{ msg }}</li></ul>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_prop_read_through_dollar_props() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string }>();
</script>

<template>
  <div>{{ $props.msg }}</div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_prop_read_in_a_dynamic_directive_argument() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string }>();
</script>

<template>
  <div :[msg]="1"></div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

// --- The blanket suppressors must survive ---------------------------------

#[test]
fn ignores_everything_when_the_props_object_is_captured() {
    // `props[key]` is invisible to any scan, so a captured props object
    // suppresses the whole component. This is the issue's headline repro, and
    // staying silent on it is the sound direction.
    let sfc = r#"<script setup lang="ts">
const props = defineProps<{ msg: string }>();
</script>

<template>
  <div>hi</div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_everything_when_define_props_is_wrapped() {
    let sfc = r#"<script setup lang="ts">
const props = withDefaults(defineProps<{ msg?: string }>(), { msg: 'x' });
</script>

<template>
  <div>hi</div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_prop_used_only_in_the_script() {
    let sfc = r#"<script setup lang="ts">
const { msg } = defineProps<{ msg: string }>();
const upper = msg.toUpperCase();
</script>

<template>
  <div>{{ upper }}</div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_prop_named_by_a_sibling_options_api_block() {
    let sfc = r#"<script>
export default { methods: { show() { return this.msg; } } };
</script>

<script setup lang="ts">
defineProps<{ msg: string }>();
</script>

<template>
  <div>hi</div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_an_underscore_prefixed_prop() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ _internal: string }>();
</script>

<template>
  <div>hi</div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_the_options_api_props_spelling() {
    // Croquis exposes only `defineProps` props, so this spelling declares
    // nothing here and is documented as out of scope.
    let sfc = r#"<script>
export default { props: { msg: String } };
</script>

<template>
  <div>hi</div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_component_with_no_props_at_all() {
    let sfc = r#"<script setup lang="ts">
const a = 1;
</script>

<template>
  <div>{{ a }}</div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

// --- Under-match probes: a reference the scan must not miss ---------------
//
// This rule reports the *absence* of a reference, so an under-match is the
// false positive. Each of these keeps the rule silent.

#[test]
fn ignores_a_prop_named_only_inside_a_template_string_literal() {
    // Over-approximating within an expression only suppresses a report.
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string }>();
</script>

<template>
  <div :title="'msg'"></div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_prop_whose_name_a_v_for_alias_reuses() {
    // Shadowing is deliberately not honoured: treating the alias as a
    // reference only suppresses.
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string; rows: string[] }>();
</script>

<template>
  <ul><li v-for="msg in rows" :key="msg">{{ msg }}</li></ul>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_non_ascii_prop_named_by_a_sibling_options_api_block() {
    // Identifiers are not ASCII-only, so a byte-wise token scan drops this
    // reference entirely and reports the prop as unused.
    let sfc = r#"<script>
export default { methods: { show() { return this.ラベル; } } };
</script>

<script setup>
defineProps({ ラベル: String });
</script>

<template>
  <div>hi</div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_non_ascii_prop_read_in_an_interpolation() {
    let sfc = r#"<script setup>
defineProps({ étiquette: String });
</script>

<template>
  <div>{{ étiquette }}</div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}
