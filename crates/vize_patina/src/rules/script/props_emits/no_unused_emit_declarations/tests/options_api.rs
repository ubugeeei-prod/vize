use super::{create_linter, lint_sfc};

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
