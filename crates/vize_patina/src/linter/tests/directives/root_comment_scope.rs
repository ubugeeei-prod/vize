//! `<!-- eslint-disable rule -->` written between the blocks applies to the
//! top-level element after it, as eslint-plugin-vue's comment directive does.

use super::Linter;

const RULE: &str = "vue/no-multiple-template-root";

fn multi_root_reports(source: &str) -> usize {
    Linter::new()
        .with_enabled_rules(Some(vec![RULE.into()]))
        .lint_sfc(source, "Popover.vue")
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE)
        .count()
}

#[test]
fn disable_above_the_template_reaches_it() {
    let source = r#"<script setup lang="ts">
const a = 1;
</script>

<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <p>{{ a }}</p>
  <p>second root</p>
</template>
"#;
    assert_eq!(multi_root_reports(source), 0);
}

#[test]
fn disable_at_the_top_of_a_template_first_file_reaches_it() {
    let source = r#"<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <p>first root</p>
  <p>second root</p>
</template>
"#;
    assert_eq!(multi_root_reports(source), 0);
}

#[test]
fn disable_is_cleared_at_the_end_of_the_next_block() {
    let source = r#"<!-- eslint-disable vue/no-multiple-template-root -->
<script setup lang="ts">
const a = 1;
</script>

<template>
  <p>{{ a }}</p>
  <p>second root</p>
</template>
"#;
    assert_eq!(multi_root_reports(source), 1);
}

#[test]
fn enable_before_the_template_closes_the_region() {
    let source = r#"<!-- eslint-disable vue/no-multiple-template-root -->
<!-- eslint-enable vue/no-multiple-template-root -->
<template>
  <p>first root</p>
  <p>second root</p>
</template>
"#;
    assert_eq!(multi_root_reports(source), 1);
}

#[test]
fn disable_of_another_rule_does_not_reach_the_template() {
    let source = r#"<!-- eslint-disable vue/no-v-html -->
<template>
  <p>first root</p>
  <p>second root</p>
</template>
"#;
    assert_eq!(multi_root_reports(source), 1);
}

#[test]
fn next_line_pragma_only_covers_the_line_after_the_comment() {
    let source = r#"<!-- eslint-disable-next-line vue/no-multiple-template-root -->
<template>
  <p>first root</p>
  <p>second root</p>
</template>
"#;
    assert_eq!(multi_root_reports(source), 1);
}

#[test]
fn comment_text_inside_a_script_string_is_not_a_root_comment() {
    let source = r#"<script setup lang="ts">
const note = "<!-- eslint-disable vue/no-multiple-template-root -->";
</script>

<template>
  <p>{{ note }}</p>
  <p>second root</p>
</template>
"#;
    assert_eq!(multi_root_reports(source), 1);
}
