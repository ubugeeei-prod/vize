#![allow(clippy::disallowed_macros)] // `insta::assert_snapshot!` expands to `format!`.

use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};

fn compile(source: &str) -> vize_carton::String {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse SFC");
    compile_sfc(&descriptor, SfcCompileOptions::default())
        .expect("compile SFC")
        .code
}

#[test]
fn custom_directive_with_children_keeps_need_patch() {
    let code = compile(
        r#"<script setup>
import { ref } from 'vue'
const value = ref('first')
</script>

<template>
  <div v-track="value">content</div>
</template>"#,
    );

    insta::assert_snapshot!("custom_directive_with_children_need_patch", code);
}

#[test]
fn built_in_v_show_with_children_keeps_its_runtime_path() {
    let code = compile(
        r#"<script setup>
import { ref } from 'vue'
const visible = ref(true)
</script>

<template>
  <div v-show="visible">content</div>
</template>"#,
    );

    insta::assert_snapshot!("built_in_v_show_with_children_runtime_path", code);
}
