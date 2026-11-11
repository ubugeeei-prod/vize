use crate::linter::{LintResult, Linter};
use crate::rules::script::NoUnusedEmitDeclarations;
use crate::rules::script::ScriptLinter;

// Template-usage tests (#3209) live in the `template` submodule.
mod above_declaration;
mod options_api;
mod template;

fn create_linter() -> ScriptLinter {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(NoUnusedEmitDeclarations));
    linter
}

/// Lint a full SFC end-to-end with only this rule enabled, exercising the
/// engine path that supplies the raw `<template>` source to the rule (#3209).
fn lint_sfc(sfc: &str) -> LintResult {
    Linter::new()
        .with_enabled_rules(Some(vec!["script/no-unused-emit-declarations".into()]))
        .lint_sfc(sfc, "test.vue")
}

#[test]
fn test_valid_all_emitted_array() {
    let source = r#"
const emit = defineEmits(['change', 'update'])
emit('change')
emit('update')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_invalid_one_unused_array() {
    let source = r#"
const emit = defineEmits(['change', 'unused'])
emit('change')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_none_emitted() {
    let source = r#"
const emit = defineEmits(['change', 'update'])
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 2);
}

#[test]
fn test_valid_emitted_inside_function() {
    let source = r#"
const emit = defineEmits(['change'])
function onClick() {
  emit('change')
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_unassigned_define_emits_reports_unused_declarations() {
    let source = r#"
defineEmits(['change', 'update'])
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 2);
}

#[test]
fn test_unassigned_define_emits_counts_template_dollar_emit() {
    let sfc = r#"<script setup lang="ts">
defineEmits<{ save: []; cancel: [] }>();
</script>
<template><button @click="$emit('save')" /></template>"#;
    let result = lint_sfc(sfc);
    let diagnostics: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "script/no-unused-emit-declarations")
        .collect();
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(
        diagnostics[0].message,
        "The 'cancel' event is declared but never emitted."
    );
}

#[test]
fn unassigned_union_call_signature_reports_only_the_unused_event() {
    let sfc = r#"<script setup lang="ts">
defineEmits<{ (e: "save" | "cancel" | "delete"): void }>();
</script>
<template>
  <button @click="$emit('save')" />
  <button @click="$emit('cancel')" />
</template>"#;
    let result = lint_sfc(sfc);
    let diagnostics: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "script/no-unused-emit-declarations")
        .collect();
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(
        diagnostics[0].message,
        "The 'delete' event is declared but never emitted."
    );
}

#[test]
fn test_type_based_property_form() {
    let source = r#"
const emit = defineEmits<{ change: [id: number]; unused: [] }>()
emit('change')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_type_based_call_signature_form() {
    let source = r#"
const emit = defineEmits<{
  (e: 'change', id: number): void
  (e: 'unused'): void
}>()
emit('change')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_type_based_all_used() {
    let source = r#"
const emit = defineEmits<{ change: [id: number] }>()
emit('change')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_type_query_does_not_escape_analysis() {
    let source = r#"
const emit = defineEmits(['a'])
type Emit = typeof emit
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_emit_passed_to_a_call_escapes_analysis() {
    let source = r#"
import { useClose } from "./use-close";
const emit = defineEmits<{ close: [] }>();
const { close } = useClose(emit);
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_type_cast_emit_argument_escapes_analysis() {
    let source = r#"
const emit = defineEmits<{ close: [] }>()
useVModel(props, 'modelValue', emit as any)
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_emit_stored_escapes_analysis() {
    let source = r#"
const emit = defineEmits(['close', 'save'])
const handlers = { emit }
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_emit_bind_escapes_analysis() {
    let source = r#"
const emit = defineEmits<{ save: [value: boolean] }>()
const handlers = { save: emit.bind(null, 'save') }
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_emit_returned_escapes_analysis() {
    let source = r#"
const emit = defineEmits(['close'])
function use() {
  return emit
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_wrapped_emit_references_escape_analysis() {
    for reference in [
        "emit as FieldEmit",
        "emit as unknown as FieldEmit",
        "emit!",
        "<FieldEmit>emit",
    ] {
        let source = format!(
            r#"
type FieldEmit = (event: "change", value: string) => void
const emit = defineEmits<{{ change: [value: string] }}>()
useField({{ emit: {reference} }})
"#
        );
        let result = create_linter().lint(&source, 0);
        assert_eq!(result.warning_count, 0, "reference: {reference}");
    }
}

#[test]
fn test_satisfies_wrapped_object_escapes_analysis() {
    let source = r#"
type FieldOptions = { emit: (event: "change", value: string) => void }
const emit = defineEmits<{ change: [value: string] }>()
useField({ emit } satisfies FieldOptions)
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_emit_stored_in_options_variable_escapes_analysis() {
    let source = r#"
const emit = defineEmits<{ change: [value: string] }>()
const options = { emit }
useField(options)
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_sfc_emit_passed_to_a_composable_escapes_analysis() {
    let sfc = r#"<script setup lang="ts">
import { useClose } from "./use-close";

const emit = defineEmits<{ close: [] }>();
const { close } = useClose(emit);
</script>

<template>
  <button type="button" @click="close">Close</button>
</template>
"#;
    let result = lint_sfc(sfc);
    assert_eq!(
        result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == "script/no-unused-emit-declarations")
            .count(),
        0,
        "{:?}",
        result.diagnostics
    );
}

#[test]
fn test_calls_before_define_emits_count_as_usage() {
    let source = r#"
function save() { emit('save') }
const emit = defineEmits(['save'])
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_type_reference_in_sibling_script_keeps_declaration_span() {
    let sfc = r#"<script lang="ts">
export type Events<T> = {
  'update:modelValue': [value: T]
  unused: []
}
</script>
<script setup lang="ts" generic="T">
const emit = defineEmits<Events<T>>()
emit('update:modelValue', value)
</script>"#;
    let result = lint_sfc(sfc);
    let diagnostics: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "script/no-unused-emit-declarations")
        .collect();
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(
        diagnostics[0].message,
        "The 'unused' event is declared but never emitted."
    );
    let expected = sfc.find("  unused: []").unwrap() as u32 + 2;
    assert_eq!(diagnostics[0].start, expected);
}

#[test]
fn test_runtime_object_form() {
    let source = r#"
const emit = defineEmits({
  change: null,
  unused: null
})
emit('change')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_custom_emit_binding_name() {
    let source = r#"
const myEmit = defineEmits(['change', 'unused'])
myEmit('change')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_dynamic_emit_argument_does_not_mark() {
    // A non-string-literal argument can't be matched to a declared name, so the
    // declared events remain unused.
    let source = r#"
const emit = defineEmits(['change'])
const name = 'change'
emit(name)
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_no_define_emits() {
    let source = r#"
const props = defineProps<{ name: string }>()
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}
