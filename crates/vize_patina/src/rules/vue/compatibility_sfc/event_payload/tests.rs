use super::PreferSingleEventPayload;
use crate::linter::Linter;
use crate::rule::RuleRegistry;

/// Each report as its covered text and message.
fn reported(source: &str) -> Vec<(&str, vize_l0::String)> {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(PreferSingleEventPayload));
    let mut result = Linter::with_registry(registry).lint_sfc(source, "Emits.vue");
    result
        .diagnostics
        .sort_by_key(|diagnostic| diagnostic.start);
    result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                source
                    .get(diagnostic.start as usize..diagnostic.end as usize)
                    .unwrap_or_default(),
                vize_l0::String::from(diagnostic.message.as_str()),
            )
        })
        .collect()
}

fn ranges(source: &str) -> Vec<&str> {
    reported(source)
        .into_iter()
        .map(|(range, _)| range)
        .collect()
}

#[test]
fn reports_the_issue_reproduction_with_upstream_messages() {
    let source = r#"<script setup lang="ts">
const emit = defineEmits<{
  move: [x: number, y: number];
  select: [id: string];
}>();

function report(): void {
  emit("move", 1, 2);
  emit("select", "a");
}
</script>

<template>
  <button type="button" @click="report">report</button>
</template>"#;
    assert_eq!(
        reported(source),
        [
            (
                "y: number",
                "Declare a single payload parameter instead of multiple parameters for the \"move\" event."
                    .into()
            ),
            (
                "2",
                "Pass a single payload object instead of multiple arguments when emitting the \"move\" event."
                    .into()
            ),
        ]
    );
}

#[test]
fn reads_every_type_declaration_shape() {
    let source = r#"<script setup lang="ts">
interface Events {
  resize(width: number, height: number): void;
  (e: 'drag' | 'drop', x: number, y: number, z: number): void;
  ok: [value: number];
  rest: [first: number, ...others: string[]];
}
defineEmits<Events>()
</script>"#;
    assert_eq!(
        ranges(source),
        [
            "height: number",
            "y: number, z: number",
            "y: number, z: number",
            "...others: string[]",
        ]
    );
    let function = "<script setup lang=\"ts\">defineEmits<(e: 'pick', id: string, at: number) => void>()</script>";
    assert_eq!(ranges(function), ["at: number"]);
}

#[test]
fn reads_runtime_validators_in_macros_and_options() {
    let setup = "<script setup>\ndefineEmits({ ok: (value) => true, update: (value, extra) => value != null, list: null })\n</script>";
    assert_eq!(ranges(setup), ["extra"]);
    let options = "<script>\nexport default {\n  emits: { change: (payload) => true, update: function (value, extra, more) { return true } },\n}\n</script>";
    assert_eq!(ranges(options), ["extra, more"]);
    let array =
        "<script setup>\nconst emit = defineEmits(['update'])\nemit('update', 1)\n</script>";
    assert_eq!(ranges(array), Vec::<&str>::new());
}

#[test]
fn reads_every_emit_call_upstream_checks() {
    let source = r#"<script>
export default {
  setup(props, { emit }) {
    emit('a', 1, 2)
    emit('a', { one: 1, two: 2 })
  },
  methods: {
    go() {
      this.$emit('b', x, y)
      this.$emit(name, x, y, z)
      other.emit('c', x, y)
    },
  },
}
</script>"#;
    assert_eq!(
        reported(source),
        [
            (
                "2",
                "Pass a single payload object instead of multiple arguments when emitting the \"a\" event."
                    .into()
            ),
            (
                "y",
                "Pass a single payload object instead of multiple arguments when emitting the \"b\" event."
                    .into()
            ),
            (
                "y, z",
                "Pass a single payload object instead of multiple arguments when emitting the \"unknown\" event."
                    .into()
            ),
        ]
    );
    let context = "<script>\nexport default {\n  setup(props, ctx) { ctx.emit(`go`, 1, 2); emit('go', 1, 2) },\n}\n</script>";
    assert_eq!(ranges(context), ["2"]);
}

#[test]
fn reads_template_calls_of_dollar_emit_and_the_setup_binding() {
    let source = r#"<script setup>
const notify = defineEmits(['update', 'pick'])
</script>
<template>
  <button @click="$emit('update', value, extra)" />
  <button @click="$emit('update', { value, extra })" />
  <button @click="notify('pick', a, b, c)" />
  <Child v-slot="{ notify }"><button @click="notify('pick', a, b)" /></Child>
  <button @click="emit('pick', a, b)" />
</template>"#;
    assert_eq!(ranges(source), ["extra", "b, c"]);
}

#[test]
fn a_single_payload_and_unrelated_calls_stay_quiet() {
    let source = r#"<script setup lang="ts">
import { bus } from './bus'
const emit = defineEmits<{ change: [payload: { a: number; b: number }]; (e: 'ping'): void }>()
emit('change', { a: 1, b: 2 })
emit('ping')
bus.emit('change', 1, 2)
function local(emit: (...args: unknown[]) => void) { emit('x', 1, 2) }
</script>"#;
    assert_eq!(ranges(source), Vec::<&str>::new());
}
