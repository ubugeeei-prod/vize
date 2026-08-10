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

/// Extracts the `__VizeGenericComponentConstructor` declaration body.
fn generic_constructor_of(virtual_ts: &str) -> &str {
    let start = virtual_ts
        .find("type __VizeGenericComponentConstructor")
        .expect("generic component constructor present");
    let rest = &virtual_ts[start..];
    // Every field line ends in `;`, so the declaration ends at the first line
    // that closes the object literal.
    let mut end = 0;
    for line in rest.split_inclusive('\n') {
        end += line.len();
        if line.starts_with('}') {
            break;
        }
    }
    &rest[..end]
}

#[test]
fn generic_constructor_instantiates_generic_module_aliases() {
    // #3354: the SFC's type parameters are in scope inside the generic
    // constructor, so an alias that re-declared them must be instantiated.
    // Bare `Slots`/`Emits`/`Exposed` are legal TypeScript — they silently
    // resolve to the alias defaults — so this only surfaces as wrong inferred
    // types downstream, never as a compile error in the virtual module.
    let source = r#"<script setup lang="ts" generic="T extends string = 'fallback'">
defineProps<{ name: T }>();
defineSlots<{ default(props: { item: T }): unknown }>();
const emit = defineEmits<{ pick: [value: T] }>();
defineExpose<{ current: T }>();
void emit;
</script>

<template>
  <div><slot :item="name" /></div>
</template>"#;

    let virtual_ts = generate_virtual_ts_from_sfc(source);
    let constructor = generic_constructor_of(&virtual_ts);
    let generic_instance = virtual_ts
        .split_once("type __VizeGenericComponentInstance")
        .expect("generic instance present")
        .1
        .split_once("type __VizeGenericComponentConstructor")
        .expect("generic constructor follows instance")
        .0;

    for expected in [
        "$slots: Slots<T>",
        "$emit: __VizeStrictPublicEmit<Emits<T>>",
        "__VizeShallowUnwrapRef<Exposed<T>>",
    ] {
        assert!(
            generic_instance.contains(expected),
            "generic instance must contain `{expected}`:\n{generic_instance}"
        );
    }
    assert!(
        constructor.contains("__EmitProps<Emits<T>>"),
        "generic input constructor must instantiate emit props:\n{constructor}"
    );

    // The non-generic constructor has no parameters in scope, so it must keep
    // the bare aliases and rely on their declared defaults.
    let (before_generic, _) = virtual_ts
        .split_once("type __VizeGenericComponentConstructor")
        .expect("generic constructor present");
    let instance = before_generic
        .rsplit_once("type __VizeComponentInstance")
        .expect("non-generic instance present")
        .1;
    assert!(
        instance.contains("$slots: Slots;"),
        "the non-generic instance must keep the bare alias:\n{instance}"
    );
}

#[test]
fn generic_constructor_leaves_non_generic_aliases_bare() {
    // The counterpart: a generic SFC whose `defineSlots`/`defineExpose` types do
    // not reference the parameter produce non-generic aliases. Instantiating
    // those would emit `Slots<T>` for `type Slots = {...}`, which is a hard
    // TS2315 in the virtual module, so the reference must stay bare.
    let source = r#"<script setup lang="ts" generic="T extends string = 'fallback'">
defineProps<{ name: T }>();
defineSlots<{ default(props: { label: string }): unknown }>();
defineExpose<{ ready: boolean }>();
</script>

<template>
  <div><slot label="x" /></div>
</template>"#;

    let virtual_ts = generate_virtual_ts_from_sfc(source);
    let constructor = generic_constructor_of(&virtual_ts);
    let generic_instance = virtual_ts
        .split_once("type __VizeGenericComponentInstance")
        .expect("generic instance present")
        .1
        .split_once("type __VizeGenericComponentConstructor")
        .expect("generic constructor follows instance")
        .0;

    assert!(
        virtual_ts.contains("export type Slots = {"),
        "slots alias should take no parameters here:\n{virtual_ts}"
    );
    assert!(
        generic_instance.contains("$slots: Slots;"),
        "a non-generic alias must not be instantiated:\n{generic_instance}"
    );
    assert!(
        !constructor.contains("Slots<"),
        "a non-generic alias must not be instantiated:\n{constructor}"
    );
    assert!(
        !generic_instance.contains("Exposed<T>") && !constructor.contains("Exposed<T>"),
        "a non-generic Exposed alias must not be instantiated:\n{generic_instance}{constructor}"
    );
}
