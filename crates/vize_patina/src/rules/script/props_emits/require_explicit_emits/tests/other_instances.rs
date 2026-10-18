//! `other.$emit(...)` emits on another instance (#9076).

use super::create_linter;

#[test]
fn emits_on_other_instances_are_not_this_components() {
    let source = r#"
export default {
  emits: ['change'],
  methods: {
    pick(picker) {
      picker.$emit('pick', new Date())
      ;(this.$datepicker).$emit('week-number-click', 1)
      this.$emit('change')
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn this_emits_still_need_a_declaration() {
    let source = r#"
export default {
  emits: ['change'],
  methods: {
    pick() {
      this.$emit('pick')
      ;(this).$emit('other')
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 2);
}

#[test]
fn untyped_define_emits_leaves_the_event_surface_unknown() {
    for source in [
        "const emit = defineEmits(); emit('save')",
        "const dispatch = defineEmits(); dispatch('cancel')",
    ] {
        let result = create_linter().lint(source, 0);
        assert!(
            result.diagnostics.is_empty(),
            "{source}: {:?}",
            result.diagnostics
        );
        assert_eq!((result.error_count, result.warning_count), (0, 0));
    }
}

#[test]
fn explicit_empty_declarations_still_reject_literal_events() {
    for declaration in ["[]", "{}"] {
        let source = format!("const emit = defineEmits({declaration}); emit('save')");
        let result = create_linter().lint(&source, 7);
        assert_eq!((result.error_count, result.warning_count), (0, 1));
        assert_eq!(result.diagnostics.len(), 1);
        let diagnostic = &result.diagnostics[0];
        let start = source.find("'save'").unwrap() as u32 + 7;
        assert_eq!(diagnostic.rule_name, "script/require-explicit-emits");
        assert_eq!(diagnostic.severity, crate::Severity::Warning);
        assert_eq!(
            diagnostic.message,
            "The emitted event 'save' is not declared in defineEmits or the emits option."
        );
        assert_eq!((diagnostic.start, diagnostic.end), (start, start + 6));
        assert!(diagnostic.fix.is_none());
        assert_eq!(diagnostic.labels.len(), 1);
        assert_eq!(diagnostic.labels[0].message, "undeclared emitted event");
        assert_eq!(
            (diagnostic.labels[0].start, diagnostic.labels[0].end),
            (start, start + 6)
        );
        assert_eq!(
            diagnostic.help.as_deref(),
            Some(
                "Add this event to the defineEmits declaration (or the Options API `emits` option) so the component's event surface is explicit."
            )
        );
    }
}

#[test]
fn shadowed_dispatchers_do_not_emit_this_components_events() {
    for shadow in [
        "function run(emit) { emit('cancel') }",
        "const run = (emit) => emit('cancel')",
        "function run({ emit }) { emit('cancel') }",
        "{ const emit = otherEmitter; emit('cancel') }",
        "try {} catch (emit) { emit('cancel') }",
        "for (const emit of others) { emit('cancel') }",
        "function run() { emit('cancel'); var emit = otherEmitter }",
    ] {
        let source = format!("const emit = defineEmits(['save']); {shadow}; emit('save')");
        let result = create_linter().lint(&source, 0);
        assert!(
            result.diagnostics.is_empty(),
            "{source}: {:?}",
            result.diagnostics
        );
        assert_eq!((result.error_count, result.warning_count), (0, 0));
    }
}

#[test]
fn unshadowed_nested_dispatches_retain_the_declared_component_contract() {
    for expression in [
        "emit('cancel')",
        "(emit)('cancel')",
        "(emit as typeof emit)('cancel')",
        "emit!('cancel')",
    ] {
        let source =
            format!("const emit = defineEmits(['save']); function run() {{ {expression} }}");
        let result = create_linter().lint(&source, 0);
        assert_eq!(
            (result.error_count, result.warning_count),
            (0, 1),
            "{source}"
        );
        assert_eq!(result.diagnostics.len(), 1, "{source}");
        let start = source.find("'cancel'").unwrap() as u32;
        assert_eq!(
            (result.diagnostics[0].start, result.diagnostics[0].end),
            (start, start + 8)
        );
    }
}

#[test]
fn a_shadow_block_does_not_silence_the_original_dispatcher_afterwards() {
    let source =
        "const emit = defineEmits(['save']); { const emit = other; emit('local') } emit('cancel')";
    let result = create_linter().lint(source, 0);
    assert_eq!((result.error_count, result.warning_count), (0, 1));
    assert_eq!(result.diagnostics.len(), 1);
    let start = source.find("'cancel'").unwrap() as u32;
    assert_eq!(
        (result.diagnostics[0].start, result.diagnostics[0].end),
        (start, start + 8)
    );
}

#[test]
fn untyped_define_emits_does_not_invent_template_event_restrictions() {
    let source = r#"<script setup>const emit = defineEmits()</script>
<template><button @click="emit('save')" @keyup="$emit('cancel')">Save</button></template>"#;
    let result = crate::Linter::new()
        .with_enabled_rules(Some(vec!["script/require-explicit-emits".into()]))
        .lint_sfc(source, "SaveButton.vue");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!((result.error_count, result.warning_count), (0, 0));
}
