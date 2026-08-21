//! SFC parsing: blocks, `<script setup>`, scoped styles.
//!
//! Split out of `lib.rs` so that module stays inside the per-file
//! source-length budget.

use super::{SfcCompileOptions, parse_sfc};
use vize_carton::config::VueVersion;

#[test]
fn template_compile_options_default_dialect_is_vue3() {
    // The default per-file dialect must be modern Vue 3 — the zero-cost path.
    let options = SfcCompileOptions::default();
    assert_eq!(options.template.dialect, VueVersion::V3);
}

#[cfg(feature = "compile")]
#[test]
fn dialect_threads_through_compile_without_changing_vue3_output() {
    // PR2 is plumbing only: a non-Vue-3 dialect must reach the compile
    // options (and therefore ParserOptions/TransformOptions), but no
    // dialect-specific behavior is wired yet, so a plain Vue 3 template
    // compiles byte-identically regardless of the selected dialect.
    let source = r#"
<template>
  <div :class="cls" @click="onClick">{{ msg }}</div>
</template>
"#;
    use super::compile_sfc;

    let descriptor = parse_sfc(source, Default::default()).unwrap();

    let mut v2_options = SfcCompileOptions::default();
    v2_options.template.dialect = VueVersion::V2;
    assert_eq!(v2_options.template.dialect, VueVersion::V2);

    let v3 = compile_sfc(&descriptor, SfcCompileOptions::default()).unwrap();
    let v2 = compile_sfc(&descriptor, v2_options).unwrap();

    assert_eq!(
        v3.code, v2.code,
        "dialect plumbing must not change Vue 3 codegen output"
    );
}

#[test]
fn test_parse_simple_sfc() {
    let source = r#"
<template>
  <div>Hello World</div>
</template>

<script>
export default {
  name: 'HelloWorld'
}
</script>

<style>
.hello { color: red; }
</style>
"#;
    let descriptor = parse_sfc(source, Default::default()).unwrap();

    assert!(descriptor.template.is_some());
    assert!(descriptor.script.is_some());
    assert_eq!(descriptor.styles.len(), 1);
}

#[test]
fn test_parse_script_setup() {
    let source = r#"
<template>
  <div>{{ msg }}</div>
</template>

<script setup>
import { ref } from 'vue'
const msg = ref('Hello')
</script>
"#;
    let descriptor = parse_sfc(source, Default::default()).unwrap();

    assert!(descriptor.template.is_some());
    assert!(descriptor.script_setup.is_some());
}

#[test]
fn test_parse_scoped_style() {
    let source = r#"
<template>
  <div class="container">Scoped</div>
</template>

<style scoped>
.container { background: blue; }
</style>
"#;
    let descriptor = parse_sfc(source, Default::default()).unwrap();

    assert_eq!(descriptor.styles.len(), 1);
    assert!(descriptor.styles[0].scoped);
}
