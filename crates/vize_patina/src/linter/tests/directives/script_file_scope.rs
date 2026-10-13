//! `/* eslint-disable rule */` in `<script>` is a file-level ESLint directive:
//! left open, it silences that rule in everything after it, including the
//! template. eslint-plugin-vue users rely on this to exempt a whole component.

use super::Linter;

const RULE: &str = "vue/no-multiple-template-root";

fn multi_root_reports(source: &str) -> usize {
    Linter::new()
        .with_enabled_rules(Some(vec![RULE.into()]))
        .lint_sfc(source, "Edge.vue")
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE)
        .count()
}

#[test]
fn reports_without_a_directive() {
    let source = r#"<script setup lang="ts">
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
fn open_script_disable_reaches_a_later_template() {
    let source = r#"<script setup lang="ts">
/* eslint-disable vue/no-multiple-template-root */
const a = 1;
</script>

<template>
  <p>{{ a }}</p>
  <p>second root</p>
</template>
"#;
    assert_eq!(multi_root_reports(source), 0);
}

#[test]
fn open_script_disable_of_every_rule_reaches_a_later_template() {
    let source = r#"<script setup lang="ts">
/* eslint-disable */
const a = 1;
</script>

<template>
  <p>{{ a }}</p>
  <p>second root</p>
</template>
"#;
    assert_eq!(multi_root_reports(source), 0);
}

#[test]
fn script_disable_does_not_reach_an_earlier_template() {
    let source = r#"<template>
  <p>{{ a }}</p>
  <p>second root</p>
</template>

<script setup lang="ts">
/* eslint-disable vue/no-multiple-template-root */
const a = 1;
</script>
"#;
    assert_eq!(multi_root_reports(source), 1);
}

#[test]
fn script_enable_closes_the_region_before_the_template() {
    let source = r#"<script setup lang="ts">
/* eslint-disable vue/no-multiple-template-root */
const a = 1;
/* eslint-enable vue/no-multiple-template-root */
</script>

<template>
  <p>{{ a }}</p>
  <p>second root</p>
</template>
"#;
    assert_eq!(multi_root_reports(source), 1);
}

#[test]
fn script_disable_of_another_rule_does_not_reach_the_template() {
    let source = r#"<script setup lang="ts">
/* eslint-disable vue/no-v-html */
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
fn single_line_script_pragmas_do_not_reach_the_template() {
    let source = r#"<script setup lang="ts">
const a = 1; // eslint-disable-line vue/no-multiple-template-root
// eslint-disable-next-line vue/no-multiple-template-root
</script>

<template>
  <p>{{ a }}</p>
  <p>second root</p>
</template>
"#;
    assert_eq!(multi_root_reports(source), 1);
}

#[test]
fn style_disable_does_not_reach_a_later_template() {
    let source = r#"<style>
/* eslint-disable vue/no-multiple-template-root */
.a { color: red; }
</style>

<template>
  <p class="a">first root</p>
  <p>second root</p>
</template>
"#;
    assert_eq!(multi_root_reports(source), 1);
}

/// ESLint accepts region pragmas only in block comments.
#[test]
fn line_comment_disable_does_not_reach_the_template() {
    let source = r#"<script setup lang="ts">
// eslint-disable vue/no-multiple-template-root
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
fn pragma_text_inside_a_script_string_is_not_a_directive() {
    let source = r#"<script setup lang="ts">
const note = "/* eslint-disable vue/no-multiple-template-root */";
</script>

<template>
  <p>{{ note }}</p>
  <p>second root</p>
</template>
"#;
    assert_eq!(multi_root_reports(source), 1);
}

/// Another script block keeps its own pragmas, as the reserved-key
/// suppression corpus records.
#[test]
fn open_script_disable_does_not_reach_another_script_block() {
    let source = r#"<script setup>/* eslint-disable script/no-reserved-keys */ const marker = 1</script><script>export default { data() { return { _private: 0 } } }</script>"#;
    let reports = Linter::new()
        .with_enabled_rules(Some(vec!["script/no-reserved-keys".into()]))
        .lint_sfc(source, "Control.vue")
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "script/no-reserved-keys")
        .count();
    assert_eq!(reports, 1);
}
