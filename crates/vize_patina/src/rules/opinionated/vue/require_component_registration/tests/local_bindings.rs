use super::create_linter;

#[test]
fn test_allows_locally_declared_functional_components() {
    let linter = create_linter();
    let sfc = r#"<script setup lang="ts">
import { createTextVNode, type FunctionalComponent } from 'vue'
const TextSpace: FunctionalComponent = () => createTextVNode(' ')
function TextGap() { return createTextVNode(' ') }
</script>
<template><TextSpace /><text-gap /><MissingPart /></template>"#;
    let result = linter.lint_sfc(sfc, "LocalParts.vue");
    assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
}
