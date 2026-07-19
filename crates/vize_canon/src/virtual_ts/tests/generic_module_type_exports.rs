//! Module-scope type exports of generic SFCs must re-declare the
//! `<script setup generic="...">` parameters they reference. The plain check
//! drops diagnostics that map to no source position, so a dangling generic
//! name in `Slots`/`Exposed` or a constraint-violating `Props<unknown>`
//! fallback only surfaced when Corsa compiled the materialized project for
//! declaration emit (#3065).

use super::{assert_virtual_ts_snapshot, generate_virtual_ts_with_offsets};
use crate::sfc_typecheck::{SfcTypeCheckOptions, type_check_sfc};
use vize_croquis::{Analyzer, AnalyzerOptions};

fn generate_virtual_ts_from_sfc(source: &str) -> vize_carton::String {
    let options = SfcTypeCheckOptions::new("test.vue").with_virtual_ts();
    type_check_sfc(source, &options)
        .virtual_ts
        .unwrap_or_default()
}

#[test]
fn snapshot_virtual_ts_generic_slots_component() {
    let source = r#"<script setup lang="ts" generic="T extends string = 'a'">
const { value = undefined } = defineProps<{
  value?: T;
}>();

const emit = defineEmits<{
  "update:value": [value: T];
}>();

defineSlots<{
  default: (props: { value: T | undefined }) => unknown;
}>();

function onClick(): void {
  if (value !== undefined) emit("update:value", value);
}
</script>

<template>
  <button type="button" @click="onClick">
    <slot :value="value" />
  </button>
</template>"#;

    let virtual_ts = generate_virtual_ts_from_sfc(source);
    assert_virtual_ts_snapshot("virtual_ts_generic_slots_component", virtual_ts.as_str());
}

#[test]
fn snapshot_virtual_ts_multi_generic_exposed_component() {
    let source = r#"<script setup lang="ts" generic="Mode extends 'row' | 'cell', TEvent extends Mode">
const props = defineProps<{
  mode: Mode;
  event?: TEvent;
}>();

defineSlots<{
  default: (props: { mode: Mode; event: TEvent | undefined }) => unknown;
}>();

defineExpose<{
  current: () => Mode;
}>();
</script>

<template>
  <slot :mode="props.mode" :event="props.event" />
</template>"#;

    let virtual_ts = generate_virtual_ts_from_sfc(source);
    assert_virtual_ts_snapshot(
        "virtual_ts_multi_generic_exposed_component",
        virtual_ts.as_str(),
    );
}

#[test]
fn test_script_setup_generic_param_injected_into_hoisted_type() {
    // A type declared in `<script setup generic="T">` that references the
    // generic parameter is lifted to module scope; the generic must be
    // re-declared on it so `T` resolves there (a residual of the repro-8
    // hoisting fix). Bare uses like `Option[]` still resolve via `= any`.
    let script = r#"type Option = { key: T; label: string }

defineProps<{
  options: Option[]
  current: T | undefined
}>()
"#;

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup_with_generic(script, Some("T extends string"));
    let summary = analyzer.finish();

    let output =
        generate_virtual_ts_with_offsets(&summary, Some(script), None, 0, 0, &Default::default());

    let (module_scope, _setup_scope) = output
        .code
        .split_once("// ========== Setup Scope ==========")
        .expect("setup scope marker present");

    assert!(
        module_scope.contains("type Option<T extends string = any> = { key: T; label: string }"),
        "hoisted type should gain the SFC generic parameter so `T` resolves at module scope:\n{}",
        output.code
    );
}
