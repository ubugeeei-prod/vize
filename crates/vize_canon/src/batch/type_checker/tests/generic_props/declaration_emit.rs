use super::super::{BatchTypeChecker, DeclarationEmitOptions, relative_path};
use super::*;

/// `vize check --declaration` materializes the same virtual project the
/// diagnostics run uses, but Corsa's declaration program hard-fails on any
/// error inside generated regions (the check path drops diagnostics that map
/// to no source position). Generic SFCs used to lose their `generic="..."`
/// type parameters on the module-scope `Slots`/`Exposed` aliases and to
/// instantiate the constructor-fallback `Props` with constraint-violating
/// `unknown` arguments, so declaration emit aborted with TS2304/TS2344 while
/// the plain check passed (#3065).
#[test]
fn declaration_emit_preserves_generic_type_parameters() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "generic-declaration-emit",
        &[
            (
                "src/Generic.vue",
                r#"<script setup lang="ts" generic="T extends string = 'a'">
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
</template>
"#,
            ),
            (
                "src/Multi.vue",
                r#"<script setup lang="ts" generic="Mode extends 'row' | 'cell', TEvent extends Mode">
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
</template>
"#,
            ),
            (
                "src/Plain.vue",
                r#"<script setup lang="ts">
const props = defineProps<{
  label: string;
}>();

defineSlots<{
  default: (props: { label: string }) => unknown;
}>();
</script>

<template>
  <slot :label="props.label" />
</template>
"#,
            ),
        ],
    );

    let Some(diagnostics) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };
    assert_eq!(
        diagnostics,
        Vec::new(),
        "generic SFCs must check clean before declaration emit"
    );

    let mut checker = match BatchTypeChecker::new(&project_root) {
        Ok(checker) => checker,
        Err(_) => return,
    };
    checker.scan_project().unwrap();
    let out_dir = project_root.join("types");
    let emitted = checker
        .emit_declarations(&DeclarationEmitOptions::new(out_dir.clone()))
        .expect("declaration emit must succeed for generic SFCs");
    let snapshot: Vec<_> = emitted
        .files
        .into_iter()
        .map(|file| (relative_path(&out_dir, &file.path), file.content))
        .collect();

    insta::with_settings!({
        snapshot_path => "../../../../snapshots"
    }, {
        insta::assert_debug_snapshot!("generic_declaration_emit_outputs", snapshot);
    });

    let _ = std::fs::remove_dir_all(&project_root);
}
