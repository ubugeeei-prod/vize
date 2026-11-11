use super::Linter;

const SOURCE: &str = r#"<script setup lang="ts">
const count = 1;
</script>

<template>
  <!-- eslint-disable-next-line vue/eqeqeq -->
  <p v-if="count == 1">one</p>
  <p v-if="count == 2">two</p>
</template>
"#;

fn lines(linter: Linter) -> Vec<u32> {
    let source = SOURCE;
    linter
        .with_enabled_rules(Some(vec!["vue/eqeqeq".into()]))
        .lint_sfc(source, "Directive.vue")
        .diagnostics
        .into_iter()
        .filter(|diagnostic| diagnostic.rule_name == "vue/eqeqeq")
        .map(|diagnostic| {
            source[..diagnostic.start as usize]
                .bytes()
                .filter(|byte| *byte == b'\n')
                .count() as u32
                + 1
        })
        .collect()
}

#[test]
fn inline_disable_hides_the_next_line_by_default() {
    assert_eq!(lines(Linter::new()), vec![8]);
}

#[test]
fn no_inline_config_reports_both_equality_lines() {
    assert_eq!(lines(Linter::new().with_no_inline_config(true)), vec![7, 8]);
}
