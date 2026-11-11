use super::*;
use crate::linter::Linter;
use crate::rule::Rule;
use crate::rule::RuleRegistry;

fn lint(rule: Box<dyn Rule>, source: &str) -> crate::linter::LintResult {
    let mut registry = RuleRegistry::new();
    registry.register(rule);
    Linter::with_registry(registry).lint_sfc(source, "Component.vue")
}

#[test]
fn reports_style_mode_and_content_outside_blocks() {
    let style = lint(
        Box::new(EnforceStyleAttribute::default()),
        "<template><div /></template><style module>.x{}</style>",
    );
    assert_eq!(style.warning_count, 1);
    let outside = lint(
        Box::new(NoContentOutsideBlock),
        "stray<!-- kept --><template><div /></template>",
    );
    assert_eq!(outside.warning_count, 1);
    let bom = lint(
        Box::new(NoContentOutsideBlock),
        "\u{feff}<template><div /></template>\n",
    );
    assert_eq!(bom.warning_count, 0, "{:?}", bom.diagnostics);
    let chunks = lint(
        Box::new(NoContentOutsideBlock),
        "<template><div /></template>\nfirst\n<!-- note -->\nsecond\n",
    );
    assert_eq!(chunks.warning_count, 2, "{:?}", chunks.diagnostics);
}

#[test]
fn reports_setup_macro_compatibility_failures() {
    let model = lint(
        Box::new(NoDefineModelDefault),
        "<script setup>defineModel({ default: 1 })</script><template />",
    );
    assert_eq!(model.warning_count, 1);
    let variable = lint(
        Box::new(RequireMacroVariableName),
        "<script setup>const p = defineProps({ value: String })</script><template />",
    );
    assert_eq!(variable.warning_count, 1);
    let shadow = lint(
        Box::new(NoPropsShadow),
        "<script setup>defineProps({ value: String }); const value = 1</script><template />",
    );
    assert_eq!(shadow.warning_count, 1);
}

#[test]
fn reports_next_tick_native_events_and_unsorted_options() {
    let next_tick = lint(
        Box::new(NextTickStyle),
        "<script setup>import { nextTick } from 'vue'; nextTick(() => ready())</script><template />",
    );
    assert_eq!(next_tick.warning_count, 1);
    let event = lint(
        Box::new(NoShadowNativeEvents),
        "<script setup>const emit = defineEmits(['click'])</script><template />",
    );
    assert_eq!(event.warning_count, 1);
    let sorted = lint(
        Box::new(SortKeys::default()),
        "<script>export default { methods: { zebra() {}, alpha() {} } }</script><template />",
    );
    assert_eq!(sorted.warning_count, 1);
}

#[test]
fn next_tick_style_resolves_vue_imports_and_parenthesized_awaits() {
    let alias = lint(
        Box::new(NextTickStyle),
        "<script>import { nextTick as tick } from 'vue'; tick(() => undefined)</script><template />",
    );
    assert_eq!(alias.warning_count, 1, "{:?}", alias.diagnostics);

    let unrelated = lint(
        Box::new(NextTickStyle),
        "<script>import { nextTick } from './scheduler'; nextTick(() => undefined); nextTick()</script><template />",
    );
    assert_eq!(unrelated.warning_count, 0, "{:?}", unrelated.diagnostics);

    let awaited = lint(
        Box::new(NextTickStyle),
        "<script>import { nextTick } from 'vue'; async function run() { await (nextTick()) }</script><template />",
    );
    assert_eq!(awaited.warning_count, 0, "{:?}", awaited.diagnostics);

    let namespace = lint(
        Box::new(NextTickStyle),
        "<script setup>import * as Vue from 'vue'; Vue.nextTick(() => undefined)</script><template />",
    );
    assert_eq!(namespace.warning_count, 1, "{:?}", namespace.diagnostics);

    let split_scripts = lint(
        Box::new(NextTickStyle),
        "<script>import { nextTick } from 'vue'</script><script setup>nextTick(() => undefined)</script><template />",
    );
    assert_eq!(
        split_scripts.warning_count, 1,
        "{:?}",
        split_scripts.diagnostics
    );

    let identifier_suffix = lint(
        Box::new(NextTickStyle),
        "<script>import { nextTick } from 'vue'; myawait(nextTick())</script><template />",
    );
    assert_eq!(
        identifier_suffix.warning_count, 1,
        "{:?}",
        identifier_suffix.diagnostics
    );
}

fn reported<'s>(rule: Box<dyn Rule>, source: &'s str) -> Vec<&'s str> {
    let mut result = lint(rule, source);
    result
        .diagnostics
        .sort_by_key(|diagnostic| diagnostic.start);
    result
        .diagnostics
        .iter()
        .filter_map(|diagnostic| source.get(diagnostic.start as usize..diagnostic.end as usize))
        .collect()
}

#[test]
fn native_events_read_type_declarations() {
    let source = "<script setup lang=\"ts\">\ninterface Base { keydown: [] }\nconst emit = defineEmits<Base & { click: []; saved: [] } & { (e: 'focus' | 'x'): void }>()\n</script>";
    assert_eq!(
        reported(Box::new(NoShadowNativeEvents), source),
        ["keydown: []", "click: [];", "(e: 'focus' | 'x'): void"]
    );
    let function =
        "<script setup lang=\"ts\">defineEmits<(e: 'blur' | 'Input') => void>()</script>";
    assert_eq!(reported(Box::new(NoShadowNativeEvents), function).len(), 2);
}

#[test]
fn native_events_track_the_emit_binding_and_the_template() {
    let source = "<script setup lang=\"ts\">\nimport { bus, emit } from './bus'\nconst notify = defineEmits(['saved'])\nbus.emit('click'); bus.$emit('change'); emit('input'); notify('click'); notify(`focus`)\n</script>\n<template><button @click=\"notify('keydown')\" @focus=\"$emit('blur')\" @input=\"emit('input')\" /></template>";
    assert_eq!(
        reported(Box::new(NoShadowNativeEvents), source),
        ["'click'", "`focus`", "'keydown'", "'blur'"]
    );
}

#[test]
fn native_events_read_options_components() {
    let source = "<script>\nexport default {\n  emits: ['click'],\n  setup(props, { emit }) { emit('focus') },\n  methods: { go() { this.$emit('blur') } },\n}\n</script>\n<template><button @click=\"$emit('input')\" /></template>";
    assert_eq!(
        reported(Box::new(NoShadowNativeEvents), source),
        ["'click'", "'focus'", "'blur'", "'input'"]
    );
}

#[test]
fn setup_macros_ignore_member_calls() {
    let source = "<script setup lang=\"ts\">\nconst form = { defineModel: (o: unknown) => o, useSlots: () => [] }\nconst initial = form.defineModel({ default: 0 })\nconst names = form.useSlots()\nconst cast = defineModel({ default: 1 }) as unknown\nconst props = withDefaults(defineProps<{ a?: string }>(), {})\n</script>";
    assert!(reported(Box::new(NoDefineModelDefault), source).is_empty());
    assert!(reported(Box::new(RequireMacroVariableName), source).is_empty());
    let model = "<script setup>const m = defineModel('m', { default: 1 }); const n = defineModel(name, { default: 1 })</script>";
    assert_eq!(
        reported(Box::new(NoDefineModelDefault), model),
        ["default: 1"]
    );
}
