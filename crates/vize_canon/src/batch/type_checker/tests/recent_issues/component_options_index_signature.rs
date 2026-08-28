//! `typeof SomeComponent` must accept arbitrary members exactly where `vue-tsc`
//! does, and reject them exactly where `vue-tsc` does (#4150).
//!
//! Vue Language Tools compiles a plain SFC to a `DefineComponent`, whose
//! `ComponentOptionsBase` extends `LegacyOptions` and therefore carries
//! `[key: string]: any`. A *generic* SFC instead compiles to a bare generic
//! function component, which carries no such index signature — which is why the
//! generated options object cannot simply always declare one.

use crate::sfc_typecheck::{SfcTypeCheckOptions, type_check_sfc};
use vize_s0::String;

const OPTIONS_TYPE_HEAD: &str = "type __VizeVueComponentOptions = {\n";
const INDEX_SIGNATURE: &str = "  [key: string]: any;\n";
const FIRST_DECLARED_MEMBER: &str = "  name?: string;\n";

const PLAIN_SFC: &str = r#"<script setup lang="ts">
function close(): void {}
defineExpose({ close });
</script>

<template>
  <div />
</template>
"#;

const GENERIC_SFC: &str = r#"<script setup lang="ts" generic="T extends { id: string }">
const props = defineProps<{ items: T[] }>();
function pick(index: number): T {
  return props.items[index];
}
defineExpose({ pick });
</script>

<template>
  <div>{{ items.length }}</div>
</template>
"#;

fn virtual_ts(source: &str) -> String {
    type_check_sfc(
        source,
        &SfcTypeCheckOptions::new("Child.vue").with_virtual_ts(),
    )
    .virtual_ts
    .expect("virtual ts should be generated")
}

/// The emitted `__VizeVueComponentOptions` opening, up to and including its
/// first declared member.
fn options_type_prologue(virtual_ts: &str) -> String {
    let start = virtual_ts
        .find(OPTIONS_TYPE_HEAD)
        .expect("generated module must declare __VizeVueComponentOptions");
    let rest = &virtual_ts[start..];
    let end = rest
        .find(FIRST_DECLARED_MEMBER)
        .expect("__VizeVueComponentOptions must declare `name`")
        + FIRST_DECLARED_MEMBER.len();
    String::from(&rest[..end])
}

#[test]
fn plain_component_options_declare_vues_string_index_signature() {
    assert_eq!(
        options_type_prologue(&virtual_ts(PLAIN_SFC)).as_str(),
        format!("{OPTIONS_TYPE_HEAD}{INDEX_SIGNATURE}{FIRST_DECLARED_MEMBER}"),
        "a non-generic SFC is a DefineComponent, so its options object owns LegacyOptions' index signature"
    );
}

#[test]
fn generic_component_options_omit_the_string_index_signature() {
    assert_eq!(
        options_type_prologue(&virtual_ts(GENERIC_SFC)).as_str(),
        format!("{OPTIONS_TYPE_HEAD}{FIRST_DECLARED_MEMBER}"),
        "a generic SFC is a function component for vue-tsc, which reports TS2339 on every options member"
    );
}
