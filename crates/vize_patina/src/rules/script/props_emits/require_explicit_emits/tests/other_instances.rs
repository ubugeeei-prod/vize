//! Which dispatchers belong to the component (#9076), and the finding shape.

use super::{lint_sfc, options, reported, setup};

#[test]
fn emits_on_other_instances_are_not_this_components() {
    let body = "export default { emits: ['change'], methods: { pick(picker) {
        picker.$emit('pick', 1)
        ;(this.$datepicker).$emit('week-number-click', 1)
        this.$emit('change'); this.$emit('pick'); ;(this).$emit('other')
    } } }";
    assert_eq!(reported(&options(body)), vec!["'pick'", "'other'"]);
}

#[test]
fn explicit_empty_declarations_report_the_full_finding() {
    for declaration in ["[]", "{}"] {
        let sfc = setup(&format!(
            "const emit = defineEmits({declaration}); emit('save')"
        ));
        let result = lint_sfc(&sfc);
        assert_eq!((result.error_count, result.warning_count), (0, 1));
        let diagnostic = &result.diagnostics[0];
        let start = sfc.find("'save'").unwrap() as u32;
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
        let sfc = setup(&format!(
            "const emit = defineEmits(['save']); {shadow}; emit('save')"
        ));
        assert_eq!(reported(&sfc), Vec::<&str>::new(), "{shadow}");
    }
}

#[test]
fn only_plain_calls_of_the_binding_dispatch() {
    for (expression, expected) in [
        ("emit('cancel')", vec!["'cancel'"]),
        ("(emit)('cancel')", vec!["'cancel'"]),
        ("emit?.('cancel')", vec!["'cancel'"]),
        // Upstream does not look through TypeScript wrappers.
        ("(emit as typeof emit)('cancel')", vec![]),
        ("emit!('cancel')", vec![]),
    ] {
        let sfc = setup(&format!(
            "const emit = defineEmits(['save']); function run() {{ {expression} }}"
        ));
        assert_eq!(reported(&sfc), expected, "{expression}");
    }
}

#[test]
fn a_shadow_block_does_not_silence_the_original_dispatcher_afterwards() {
    let sfc = setup(
        "const emit = defineEmits(['save']); { const emit = other; emit('local') } emit('cancel')",
    );
    assert_eq!(reported(&sfc), vec!["'cancel'"]);
}

#[test]
fn untyped_define_emits_does_not_invent_template_event_restrictions() {
    let sfc = r#"<script setup>const emit = defineEmits()</script>
<template><button @click="emit('save')" @keyup="$emit('cancel')">Save</button></template>"#;
    let result = lint_sfc(sfc);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!((result.error_count, result.warning_count), (0, 0));
}
