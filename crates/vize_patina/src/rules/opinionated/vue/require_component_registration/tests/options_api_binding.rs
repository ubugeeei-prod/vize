use super::create_linter;

#[test]
fn options_api_name_resolves_an_exported_component_binding() {
    let result = create_linter().lint_sfc(
        r#"<template><b-field /></template>
<script lang="ts">
import { defineComponent } from "vue";
const Field = defineComponent({ name: "BField" });
export default Field;
</script>"#,
        "Field.vue",
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn options_api_name_does_not_resolve_an_arbitrary_exported_binding() {
    let result = create_linter().lint_sfc(
        r#"<template><b-field /></template>
<script lang="ts">
const Field = wrapComponent({ name: "BField" });
export default Field;
</script>"#,
        "OtherName.vue",
    );
    assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
}
