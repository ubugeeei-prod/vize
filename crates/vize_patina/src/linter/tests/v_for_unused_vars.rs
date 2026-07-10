use super::{Linter, ToCompactString};

#[test]
fn test_lint_template_marks_v_for_alias_used_by_nested_v_for_source() {
    let linter =
        Linter::new().with_enabled_rules(Some(vec!["vue/no-unused-vars".to_compact_string()]));
    let source = r#"<script setup>
const groups = new Map()
</script>
<template>
  <template v-for="[key, values] in groups" :key="key">
    <span v-for="value in values" :key="value">{{ value }}</span>
  </template>
</template>"#;
    let result = linter.lint_sfc(source, "test.vue");

    assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
}

#[test]
fn test_lint_template_marks_v_for_alias_used_without_script_block() {
    let linter =
        Linter::new().with_enabled_rules(Some(vec!["vue/no-unused-vars".to_compact_string()]));
    let source = r#"<template><div v-for="value in values">{{ value }}</div></template>"#;
    let result = linter.lint_sfc(source, "test.vue");

    assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
}
