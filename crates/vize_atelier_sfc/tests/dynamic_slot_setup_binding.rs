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

    insta::assert_snapshot!("script_setup_ref_dynamic_slot_name", result.code);
}
