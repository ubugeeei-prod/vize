use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};

#[test]
fn script_setup_ref_component_tag_uses_unref() {
    let source = r#"<template>
  <Menu>hello</Menu>
  <Menu v-if="show" />
  <Menu v-for="item in items" :key="item" />
  <Menu v-once />
  <Menu.Item />
</template>

<script setup>
import { computed, h, ref } from 'vue'
const show = ref(true)
const items = ref(['a'])
const Menu = computed(() => Object.assign({ render: () => h('div', 'x') }, {
  Item: { render: () => h('span', 'item') },
}))
</script>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("Failed to parse SFC");
    let result = compile_sfc(&descriptor, SfcCompileOptions::default()).expect("compile");

    assert!(
        result.code.contains("unref as _unref"),
        "computed component tags should import unref:\n{}",
        result.code
    );
    assert!(
        result.code.matches("_unref(Menu").count() >= 5,
        "all computed component tag paths should be unref'd:\n{}",
        result.code
    );
    assert!(
        result.code.contains("_createBlock(_unref(Menu)")
            || result.code.contains("_createVNode(_unref(Menu)"),
        "computed component tag must be unref'd:\n{}",
        result.code
    );
    assert!(
        result.code.contains("_createVNode(_unref(Menu))"),
        "v-once computed component tags must be unref'd:\n{}",
        result.code
    );
    assert!(
        result.code.contains("_unref(Menu).Item"),
        "dotted computed component tags must unref the base binding:\n{}",
        result.code
    );
    assert!(
        !result.code.contains("_createBlock(Menu") && !result.code.contains("_createVNode(Menu"),
        "raw computed ref must not be emitted as the component type:\n{}",
        result.code
    );
}
