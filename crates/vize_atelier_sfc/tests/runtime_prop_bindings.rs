//! Runtime `defineProps` keys come from the call AST, not option text (#12510).

use vize_atelier_sfc::{BindingType, SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};

#[test]
fn nested_runtime_prop_options_never_become_compile_result_bindings() {
    let source = r#"<script setup>
defineProps({
  /** The title. */
  title: {
    type: String,
    required: true,
    default: () => ({ nested: true }),
    validator: value => value.length > 0,
  },
  count: Number,
})
</script>
<template>{{ title }} {{ count }}</template>
"#;
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse SFC");
    let result = compile_sfc(&descriptor, SfcCompileOptions::default()).expect("compile SFC");
    let bindings = &result
        .bindings
        .expect("script setup binding metadata")
        .bindings;

    assert_eq!(bindings.get("title"), Some(&BindingType::Props));
    assert_eq!(bindings.get("count"), Some(&BindingType::Props));
    for nested in ["type", "required", "default", "validator", "nested"] {
        assert_ne!(
            bindings.get(nested),
            Some(&BindingType::Props),
            "nested option key {nested:?} must not become a prop"
        );
    }
}
