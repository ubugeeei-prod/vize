//! `defineModel`, `withDefaults`, and destructure defaults from #12744.
//!
//! These go through `Linter::lint_sfc` so the script prefilter is part of the
//! check. A `ScriptLinter` test would skip that filter.

use crate::linter::Linter;

const RULE: &str = "script/require-valid-default-prop";

fn findings(sfc: &str) -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = Linter::new()
        .with_enabled_rules(Some(vec![RULE.into()]))
        .lint_sfc(sfc, "Probe.vue")
        .diagnostics
        .into_iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE)
        .map(|diagnostic| {
            (
                sfc[diagnostic.start as usize..diagnostic.end as usize].to_string(),
                diagnostic.message.to_string(),
            )
        })
        .collect();
    found.sort();
    found
}

#[test]
fn define_model_defaults_match_the_type_argument_or_the_type_option() {
    let sfc = r#"<script setup lang="ts">
const label = defineModel<string>({ default: 123 })
const page = defineModel<number>("page", { default: "one" })
const open = defineModel("open", { type: Boolean, default: "no" })
</script>
"#;
    assert_eq!(
        findings(sfc),
        vec![
            (
                "\"no\"".to_string(),
                "Type of the default value for 'open' prop must be a boolean.".to_string(),
            ),
            (
                "\"one\"".to_string(),
                "Type of the default value for 'page' prop must be a number.".to_string(),
            ),
            (
                "123".to_string(),
                "Type of the default value for 'modelValue' prop must be a string.".to_string(),
            ),
        ]
    );
}

#[test]
fn a_matching_define_model_default_is_silent() {
    let sfc = r#"<script setup lang="ts">
const label = defineModel<string>({ default: "ok" })
const page = defineModel<number>(`page`, { default: 1 })
</script>
"#;
    assert_eq!(findings(sfc), Vec::<(String, String)>::new());
}

#[test]
fn with_defaults_checks_the_object_including_an_assertion() {
    let sfc = r#"<script setup lang="ts">
const props = withDefaults(defineProps<{ label?: string }>(), { label: 123 })
const other = withDefaults(defineProps<{ hint?: string }>(), { hint: 1 } as const)
</script>
<template><p>{{ props.label }} {{ other.hint }}</p></template>
"#;
    assert_eq!(
        findings(sfc),
        vec![
            (
                "1".to_string(),
                "Type of the default value for 'hint' prop must be a string.".to_string(),
            ),
            (
                "123".to_string(),
                "Type of the default value for 'label' prop must be a string.".to_string(),
            ),
        ]
    );
}

#[test]
fn a_non_object_with_defaults_argument_is_skipped() {
    let sfc = r#"<script setup lang="ts">
const defaults = { label: 123 }
const props = withDefaults(defineProps<{ label?: string }>(), defaults)
</script>
<template><p>{{ props.label }}</p></template>
"#;
    assert_eq!(findings(sfc), Vec::<(String, String)>::new());
}

#[test]
fn a_runtime_destructure_default_is_checked_beside_the_descriptor() {
    let sfc = r#"<script setup lang="ts">
const { label = 123, size } = defineProps({
  label: String,
  size: { type: String, default: 5 },
})
</script>
<template><p>{{ label }} {{ size }}</p></template>
"#;
    assert_eq!(
        findings(sfc),
        vec![
            (
                "123".to_string(),
                "Type of the default value for 'label' prop must be a string.".to_string(),
            ),
            (
                "5".to_string(),
                "Type of the default value for 'size' prop must be a string.".to_string(),
            ),
        ]
    );
}

#[test]
fn a_type_based_destructure_default_is_checked() {
    let sfc = r#"<script setup lang="ts">
const { label = 123 } = defineProps<{ label?: string }>()
</script>
<template><p>{{ label }}</p></template>
"#;
    assert_eq!(
        findings(sfc),
        vec![(
            "123".to_string(),
            "Type of the default value for 'label' prop must be a string.".to_string(),
        )]
    );
}

#[test]
fn an_assignment_array_literal_is_valid_and_a_function_is_the_value() {
    let sfc = r#"<script setup lang="ts">
const { items = [], foo = (() => []) } = defineProps({
  items: Array,
  foo: { type: Array },
})
</script>
<template><p>{{ items }} {{ foo }}</p></template>
"#;
    assert_eq!(
        findings(sfc),
        vec![(
            "(() => [])".to_string(),
            "Type of the default value for 'foo' prop must be a array.".to_string(),
        )]
    );
}

#[test]
fn a_type_alias_after_an_import_is_still_resolved() {
    let sfc = r#"<script setup lang="ts">
import { read } from "./read"
type Label = string
const label = defineModel<Label>({ default: 123 })
</script>
<template><p>{{ label }} {{ read }}</p></template>
"#;
    assert_eq!(
        findings(sfc),
        vec![(
            "123".to_string(),
            "Type of the default value for 'modelValue' prop must be a string.".to_string(),
        )]
    );
}

#[test]
fn an_imported_props_type_checks_destructure_defaults() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("types.ts"),
        "export interface CardProps {\n  size: string;\n  label?: string;\n  count?: number;\n}\n",
    )
    .unwrap();
    let source = r#"<script setup lang="ts">
import type { CardProps } from "./types";

const { count = "many", label = 3, size } = defineProps<CardProps>();
</script>

<template>
  <p>{{ count }} {{ label }} {{ size }}</p>
</template>
"#;
    let filename = dir.path().join("DefaultImported.vue");
    let result = Linter::new()
        .with_enabled_rules(Some(vec![RULE.into()]))
        .lint_sfc(source, filename.to_str().unwrap());
    let mut messages: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE)
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    messages.sort_unstable();
    assert_eq!(
        messages,
        [
            "Type of the default value for 'count' prop must be a number.",
            "Type of the default value for 'label' prop must be a string.",
        ]
    );
}

#[test]
fn a_shadowed_define_model_is_not_a_compiler_macro() {
    let sfc = r#"<script setup lang="ts">
const defineModel = <T,>(_options: { default: T }) => "x"
const label = defineModel<string>({ default: 123 })
</script>
<template><p>{{ label }}</p></template>
"#;
    assert_eq!(findings(sfc), Vec::<(String, String)>::new());
}
