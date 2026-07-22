use crate::linter::{LintResult, Linter};
use crate::rules::script::NoUnusedEmitDeclarations;
use crate::rules::script::ScriptLinter;

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
fn test_unassigned_define_emits_not_tracked() {
    // Without a binding we cannot track usage, so nothing is reported even
    // though no emit call exists.
    let source = r#"
defineEmits(['change', 'update'])
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
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

// ---------------------------------------------------------------------------
// Template usage (#3209): emit calls in template expressions count as usage.
// ---------------------------------------------------------------------------

#[test]
fn test_sfc_template_handler_marks_used_issue_3209() {
    // Exact reproduction from #3209: the captured binding is called only from
    // a template event handler, which must count as usage.
    let sfc = r#"<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
</script>

<template>
  <button @click='emit("save")'>Save</button>
</template>
"#;
    let result = lint_sfc(sfc);
    assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
}

#[test]
fn test_sfc_template_handler_marks_used_array_form() {
    let sfc = r#"<script setup>
const emit = defineEmits(['save'])
</script>

<template>
  <button @click="emit('save')">Save</button>
</template>
"#;
    assert_eq!(lint_sfc(sfc).warning_count, 0);
}

#[test]
fn test_sfc_unused_everywhere_still_warns() {
    // Declared but called neither in the script nor in the template: the
    // template pass must not blanket-suppress the rule.
    let sfc = r#"<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
</script>

<template>
  <button>Save</button>
</template>
"#;
    let result = lint_sfc(sfc);
    assert_eq!(result.warning_count, 1);
    assert_eq!(
        result.diagnostics[0].rule_name,
        "script/no-unused-emit-declarations"
    );
}

#[test]
fn test_sfc_template_partial_usage_unused_event_still_warns() {
    // Only `save` is emitted from the template; `cancel` must keep warning.
    let sfc = r#"<script setup lang="ts">
const emit = defineEmits<{ save: []; cancel: [] }>();
</script>

<template>
  <button @click='emit("save")'>Save</button>
</template>
"#;
    let result = lint_sfc(sfc);
    assert_eq!(result.warning_count, 1);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(diagnostic.rule_name, "script/no-unused-emit-declarations");
    assert_eq!(
        diagnostic.message,
        "The 'cancel' event is declared but never emitted."
    );
    // The span must anchor the `cancel` key inside the whole-SFC source.
    assert_eq!(
        &sfc[diagnostic.start as usize..diagnostic.end as usize],
        "cancel"
    );
}

#[test]
fn test_sfc_template_aliased_binding_marks_used() {
    // The scan must follow the actual captured binding name, not a
    // hard-coded `emit`.
    let sfc = r#"<script setup>
const fire = defineEmits(['save'])
</script>

<template>
  <button @click="fire('save')">Save</button>
</template>
"#;
    assert_eq!(lint_sfc(sfc).warning_count, 0);
}

#[test]
fn test_sfc_template_dynamic_event_name_still_warns() {
    // A dynamic event name cannot be matched to a declared name; the template
    // treatment mirrors the script-side handling of `emit(name)`.
    let sfc = r#"<script setup>
const emit = defineEmits(['save'])
const name = 'save'
</script>

<template>
  <button @click="emit(name)">Save</button>
</template>
"#;
    assert_eq!(lint_sfc(sfc).warning_count, 1);
}

#[test]
fn test_sfc_template_dollar_emit_marks_used() {
    // `$emit` in the template dispatches the same declared events as the
    // captured binding.
    let sfc = r#"<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
</script>

<template>
  <button @click="$emit('save')">Save</button>
</template>
"#;
    assert_eq!(lint_sfc(sfc).warning_count, 0);
}

#[test]
fn test_sfc_template_member_dollar_emit_does_not_count() {
    // `child.$emit(...)` dispatches on another instance, not this component.
    let sfc = r#"<script setup>
const emit = defineEmits(['save'])
</script>

<template>
  <Child ref="child" @click="child.$emit('save')" />
</template>
"#;
    assert_eq!(lint_sfc(sfc).warning_count, 1);
}

#[test]
fn test_sfc_template_v_for_and_v_if_handlers_mark_used() {
    let sfc = r#"<script setup lang="ts">
const emit = defineEmits<{ pick: [id: number]; clear: [] }>();
const items = [1, 2, 3];
const ready = true;
</script>

<template>
  <ul>
    <li v-for="item in items" :key="item">
      <button @click="emit('pick', item)">Pick</button>
    </li>
  </ul>
  <button v-if="ready" @click="emit('clear')">Clear</button>
</template>
"#;
    assert_eq!(lint_sfc(sfc).warning_count, 0);
}

#[test]
fn test_sfc_template_longhand_v_on_marks_used() {
    let sfc = r#"<script setup>
const emit = defineEmits(['save'])
</script>

<template>
  <button v-on:click="emit('save')">Save</button>
</template>
"#;
    assert_eq!(lint_sfc(sfc).warning_count, 0);
}

#[test]
fn test_sfc_template_optional_chaining_and_whitespace_mark_used() {
    let sfc = r#"<script setup>
const emit = defineEmits(['save'])
</script>

<template>
  <button @click="emit?.( 'save' )">Save</button>
</template>
"#;
    assert_eq!(lint_sfc(sfc).warning_count, 0);
}

#[test]
fn test_sfc_template_interpolation_marks_used() {
    let sfc = r#"<script setup>
const emit = defineEmits(['save'])
</script>

<template>
  <span>{{ emit('save') }}</span>
</template>
"#;
    assert_eq!(lint_sfc(sfc).warning_count, 0);
}

#[test]
fn test_sfc_template_inline_statements_mark_all_used() {
    // Both events of a multi-statement inline handler count.
    let sfc = r#"<script setup>
const emit = defineEmits(['save', 'close'])
</script>

<template>
  <button @click="emit('save'); emit('close')">Save</button>
</template>
"#;
    assert_eq!(lint_sfc(sfc).warning_count, 0);
}

#[test]
fn test_sfc_template_similar_identifier_does_not_count() {
    // `emits(...)` / `myemit(...)` are different identifiers: the whole-token
    // boundary must keep them from masking the unused `emit` binding.
    let sfc = r#"<script setup>
const emit = defineEmits(['save'])
const emits = (name) => name
const myemit = (name) => name
</script>

<template>
  <button @click="emits('save')">A</button>
  <button @click="myemit('save')">B</button>
</template>
"#;
    assert_eq!(lint_sfc(sfc).warning_count, 1);
}

#[test]
fn test_sfc_template_member_call_does_not_count() {
    // Only identifier callees are tracked, matching the script-side collector.
    let sfc = r#"<script setup>
const emit = defineEmits(['save'])
const bus = { emit: (name) => name }
</script>

<template>
  <button @click="bus.emit('save')">Save</button>
</template>
"#;
    assert_eq!(lint_sfc(sfc).warning_count, 1);
}

#[test]
fn test_sfc_template_bare_reference_does_not_count() {
    // `@click="emit"` calls the function with the native event object, which
    // never dispatches `save`; the declared event stays unused.
    let sfc = r#"<script setup>
const emit = defineEmits(['save'])
</script>

<template>
  <button @click="emit">Save</button>
</template>
"#;
    assert_eq!(lint_sfc(sfc).warning_count, 1);
}

#[test]
fn test_sfc_without_template_still_warns() {
    let sfc = r#"<script setup>
const emit = defineEmits(['save'])
</script>
"#;
    assert_eq!(lint_sfc(sfc).warning_count, 1);
}

#[test]
fn test_sfc_script_side_usage_still_counts() {
    // Sanity for the SFC path: script-side emits keep counting as before.
    let sfc = r#"<script setup>
const emit = defineEmits(['save'])
function onSave() {
  emit('save')
}
</script>

<template>
  <button @click="onSave">Save</button>
</template>
"#;
    assert_eq!(lint_sfc(sfc).warning_count, 0);
}
