//! eslint-plugin-vue's `vue/no-dupe-keys` pragma suppresses `script/no-dupe-keys`.

use vize_patina::{LintPreset, Linter};

const RULE: &str = "script/no-dupe-keys";

fn count(source: &str) -> usize {
    Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![RULE.into()]))
        .lint_sfc(source, "Dupe.vue")
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE)
        .count()
}

fn setup(pragma: &str) -> String {
    format!(
        "<script setup lang=\"ts\">\ndefineProps<{{ open: boolean }}>()\n{pragma}const open = ref(false)\n</script>\n<template><div>{{{{ open }}}}</div></template>\n"
    )
}

fn options(pragma: &str) -> String {
    format!(
        "<script>\nexport default {{\n  props: ['open'],\n  computed: {{\n    {pragma}open() {{ return true }},\n  }},\n}}\n</script>\n"
    )
}

#[test]
fn the_unsuppressed_collisions_are_reported() {
    assert_eq!(count(&setup("")), 1);
    assert_eq!(count(&options("")), 1);
}

#[test]
fn eslint_plugin_vue_and_patina_names_both_suppress() {
    for name in [
        "vue/no-dupe-keys",
        "script/no-dupe-keys",
        "vize/script/no-dupe-keys",
    ] {
        assert_eq!(
            count(&setup(&format!("// eslint-disable-next-line {name}\n"))),
            0,
            "{name}"
        );
        assert_eq!(
            count(&options(&format!(
                "// eslint-disable-next-line {name}\n    "
            ))),
            0,
            "{name}"
        );
    }
}

#[test]
fn unrelated_vue_rule_names_do_not_suppress() {
    assert_eq!(
        count(&setup(
            "// eslint-disable-next-line vue/no-dupe-v-else-if\n"
        )),
        1
    );
}
