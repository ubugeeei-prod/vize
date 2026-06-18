use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};

#[test]
fn script_setup_ref_component_tag_uses_unref() {
    let source = r#"<template>
  <Menu>hello</Menu>
</template>

<script setup>
import { computed, h } from 'vue'
const Menu = computed(() => ({ render: () => h('div', 'x') }))
</script>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let result = compile_sfc(&descriptor, SfcCompileOptions::default()).expect("compile");

    assert!(
        result.code.contains("unref as _unref"),
        "computed component tags should import unref:\n{}",
        result.code
    );
    assert!(
        result.code.contains("_createBlock(_unref(Menu)")
            || result.code.contains("_createVNode(_unref(Menu)"),
        "computed component tag must be unref'd:\n{}",
        result.code
    );
    assert!(
        !result.code.contains("_createBlock(Menu") && !result.code.contains("_createVNode(Menu"),
        "raw computed ref must not be emitted as the component type:\n{}",
        result.code
    );
}
