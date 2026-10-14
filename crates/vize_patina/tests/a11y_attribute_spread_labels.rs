//! #8961: a `v-bind` object spread can supply `for`, `id` and `aria-labelledby`.

use vize_patina::{LintPreset, Linter};

const RULES: [&str; 2] = ["a11y/label-has-for", "a11y/form-control-has-label"];

fn reported(source: &str) -> Vec<&'static str> {
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(RULES.iter().map(|rule| (*rule).into()).collect()));
    let result = linter.lint_sfc(source, "Field.vue");
    result.diagnostics.iter().map(|d| d.rule_name).collect()
}

fn sfc(template: &str) -> String {
    format!(
        "<script setup lang=\"ts\">\nconst labelProps = {{ id: \"email-label\", for: \"email\" }};\nconst fieldProps = {{ id: \"email\", \"aria-labelledby\": \"email-label\" }};\n</script>\n\n<template>\n{template}\n</template>\n"
    )
}

#[test]
fn object_spreads_are_not_reported() {
    for template in [
        "  <label v-bind=\"labelProps\">Email</label>\n  <input type=\"email\" v-bind=\"fieldProps\" />",
    ] {
        assert_eq!(reported(&sfc(template)), Vec::<&str>::new(), "{template}");
    }
}

#[test]
fn unrelated_bindings_still_report_both_rules() {
    let template = "  <label :class=\"labelProps.id\">Email</label>\n  <input type=\"email\" :class=\"fieldProps.id\" />";
    assert_eq!(reported(&sfc(template)), RULES.to_vec());
}
