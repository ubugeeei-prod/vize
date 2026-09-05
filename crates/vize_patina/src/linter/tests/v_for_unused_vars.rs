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

#[test]
fn test_lint_template_marks_v_for_alias_used_by_spread_expression() {
    let linter =
        Linter::new().with_enabled_rules(Some(vec!["vue/no-unused-vars".to_compact_string()]));
    let source = r#"<template>
  <el-cascader-menu v-for="(menu, index) in menus" :key="index" :nodes="[...menu]" />
</template>"#;
    let result = linter.lint_sfc(source, "test.vue");

    assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
}

#[test]
fn test_lint_template_allows_leading_v_for_alias_when_index_is_used() {
    let linter =
        Linter::new().with_enabled_rules(Some(vec!["vue/no-unused-vars".to_compact_string()]));
    let source = r#"<template>
  <HoverCardRoot v-for="(entry, index) in 20" :key="index">
    {{ index }}
  </HoverCardRoot>
</template>"#;
    let result = linter.lint_sfc(source, "HoverMultiCard.story.vue");

    assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
}

#[test]
fn test_lint_template_allows_value_and_key_aliases_when_index_is_used() {
    let linter =
        Linter::new().with_enabled_rules(Some(vec!["vue/no-unused-vars".to_compact_string()]));
    let source = r#"<template>
  <div v-for="(entry, key, index) in items" :key="index">{{ index }}</div>
</template>"#;
    let result = linter.lint_sfc(source, "test.vue");

    assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
}

#[test]
fn test_lint_template_reports_unused_destructured_value_before_used_index() {
    let linter =
        Linter::new().with_enabled_rules(Some(vec!["vue/no-unused-vars".to_compact_string()]));
    let source = r#"<template>
  <div v-for="({ id, label }, index) in items" :key="index">{{ index }}</div>
</template>"#;
    let result = linter.lint_sfc(source, "test.vue");

    assert_eq!(result.warning_count, 2, "{:?}", result.diagnostics);
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("'id'"))
    );
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("'label'"))
    );
}
