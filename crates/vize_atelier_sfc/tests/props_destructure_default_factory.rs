use vize_atelier_sfc::{SfcCompileOptions, compile_sfc, parse_sfc};

#[test]
fn type_based_expression_defaults_are_evaluated_per_instance() {
    let source = r#"<script setup lang="ts">
const { start = new Date().toISOString(), id = crypto.randomUUID(), items = [] } = defineProps<{
  start?: string;
  id?: string;
  items?: string[];
}>();
</script>
<template><p>{{ start }} {{ id }} {{ items.length }}</p></template>"#;
    let descriptor = parse_sfc(source, Default::default()).unwrap();

    for inline_template in [false, true] {
        let mut options = SfcCompileOptions::default();
        options.script.inline_template = inline_template;
        let result = compile_sfc(&descriptor, options).unwrap();
        assert!(
            result
                .code
                .contains("default: () => new Date().toISOString()"),
            "{}",
            result.code
        );
        assert!(
            result.code.contains("default: () => crypto.randomUUID()"),
            "{}",
            result.code
        );
        assert!(result.code.contains("default: () => []"), "{}", result.code);
    }
}
