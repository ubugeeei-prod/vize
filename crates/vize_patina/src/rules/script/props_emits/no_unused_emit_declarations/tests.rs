use crate::linter::{LintResult, Linter};
use crate::rules::script::NoUnusedEmitDeclarations;
use crate::rules::script::ScriptLinter;

// Template-usage tests (#3209) live in the `template` submodule.
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
fn test_type_cast_emit_argument_does_not_escape_upstream_analysis() {
    let source = r#"
const emit = defineEmits<{ close: [] }>()
useVModel(props, 'modelValue', emit as any)
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_emit_stored_does_not_count_as_direct_usage() {
    let source = r#"
const emit = defineEmits(['close', 'save'])
const handlers = { emit }
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 2);
}

#[test]
fn test_emit_bind_does_not_count_as_emitting_or_escaping() {
    let source = r#"
const emit = defineEmits<{ save: [value: boolean] }>()
const handlers = { save: emit.bind(null, 'save') }
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_emit_returned_does_not_count_as_direct_usage() {
    let source = r#"
const emit = defineEmits(['close'])
function use() {
  return emit
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
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
fn test_calls_before_define_emits_do_not_count_as_usage() {
    let source = r#"
function save() { emit('save') }
const emit = defineEmits(['save'])
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 1);
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

#[test]
fn test_options_api_reports_unused_array_emit() {
    let sfc = r#"<template><button @click="$emit('save')" /></template>
<script>
export default {
  emits: ['save', 'cancel'],
  methods: { submit() { this.$emit('save') } }
}
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
        "The 'cancel' event is declared but never emitted."
    );
}

#[test]
fn test_options_api_dynamic_emit_name_does_not_mark_declarations_used() {
    let sfc = r#"<template />
<script>
export default {
  emits: ['open', 'close'],
  methods: { toggle(active) { this.$emit(active ? 'close' : 'open') } }
}
</script>"#;
    let result = lint_sfc(sfc);
    assert_eq!(
        result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == "script/no-unused-emit-declarations")
            .count(),
        2
    );
}

#[test]
fn test_options_api_setup_emit_bindings_are_tracked_by_symbol() {
    for setup in [
        "setup(props, { emit }) { emit('save') }",
        "setup(props, context) { context.emit('save') }",
    ] {
        let sfc = format!("<script>export default {{ emits: ['save'], {setup} }}</script>");
        let result = lint_sfc(&sfc);
        assert_eq!(
            result
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.rule_name == "script/no-unused-emit-declarations")
                .count(),
            0,
            "{setup}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_options_api_emit_passed_to_composable_escapes_analysis() {
    let sfc = r#"<script>
export default {
  emits: ['save'],
  setup(props, { emit }) { useFeature({ props, emit }) }
}
</script>"#;
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
fn test_options_api_setup_context_passed_to_composable_escapes_analysis() {
    let source = r#"
export default {
  emits: ['open', 'close'],
  setup(props, context) {
    useDropdown(props, context)
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0, "{:?}", result.diagnostics);
}

#[test]
fn options_api_keeps_literal_events_beside_dynamic_declarations() {
    let result = lint_sfc(
        r#"<template><button @click="$emit('accepted')" /></template>
<script>
const dynamic = 'change'
export default { emits: ['click', dynamic] }
</script>"#,
    );
    let messages: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "script/no-unused-emit-declarations")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert_eq!(messages.len(), 1, "{:?}", result.diagnostics);
    assert_eq!(
        messages,
        ["The 'click' event is declared but never emitted."]
    );
}

#[test]
fn test_define_model_uses_its_generated_update_event() {
    let source = r#"
const emit = defineEmits(['update:open', 'cancel'])
const open = defineModel('open')
emit('cancel')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
}

#[test]
fn define_model_options_object_uses_the_default_update_event() {
    let source = r#"
const emit = defineEmits(['update:modelValue', 'cancel'])
const model = defineModel({ default: null })
emit('cancel')
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
}

#[test]
fn options_api_inline_template_emit_counts_as_usage() {
    let source = r#"
const ModalForm = defineComponent({
  emits: ['close'],
  template: '<button @click="$emit(\'close\')">Close</button>'
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
}

#[test]
fn options_api_inline_template_literal_emit_counts_as_usage() {
    let source = r#"
const ModalForm = defineComponent({
  emits: ['close'],
  template: `<button @click="$emit('close')">Close</button>`
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
}

#[test]
fn options_api_this_alias_emit_counts_as_usage() {
    let source = r#"
export default {
  emits: ['show', 'hide'],
  methods: {
    toggle(open) {
      const that = this
      that.$emit(open ? 'show' : 'hide')
      that.$emit('show')
      that.$emit('hide')
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
}

#[test]
fn test_options_api_unrelated_emit_calls_do_not_hide_unused_declarations() {
    let sfc = r#"<script>
export default {
  emits: ['save'],
  setup(props, { emit }) {
    service.emit('save')
    function shadow(emit) { emit('save') }
    shadow(console.log)
  }
}
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
        "The 'save' event is declared but never emitted."
    );
}
