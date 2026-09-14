use super::{LintPreset, Linter};

#[test]
fn test_lint_sfc_opinionated_allows_dynamic_component_tag_name_v_html() {
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let sfc = r#"<script setup lang="ts">
const { tagName = "div", html = "" } = defineProps<{
  tagName?: string
  html?: string
}>()
</script>

<template>
  <component :is="tagName" v-html="html" />
</template>
"#;
    let result = linter.lint_sfc(sfc, "PreviewCard.vue");
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name != "vue/no-v-text-v-html-on-component"),
        "got: {:?}",
        result.diagnostics
    );
}
