use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};

#[test]
fn script_setup_ref_drives_a_dynamic_slot_name() {
    let source = r#"<script setup>
import { ref } from 'vue'
import Child from './Child.vue'

const slotName = ref('header')
</script>

<template>
  <Child>
    <template #[slotName]>content</template>
  </Child>
</template>"#;

    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse SFC");
    let result = compile_sfc(&descriptor, SfcCompileOptions::default()).expect("compile SFC");

    assert!(
        result.code.contains("[slotName.value]: _withCtx"),
        "dynamic slot names must resolve script-setup refs like template expressions:\n{}",
        result.code
    );
    assert!(
        !result.code.contains("[_ctx.slotName]"),
        "script-setup bindings must not be resolved through the public instance:\n{}",
        result.code
    );
}
