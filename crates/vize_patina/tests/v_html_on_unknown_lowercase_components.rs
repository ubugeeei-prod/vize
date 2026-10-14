//! `vue/no-v-text-v-html-on-component` classifies tags like Vue's compiler and
//! the reference rule: every name outside the known HTML, SVG and MathML tags
//! of its namespace is a component.

use vize_patina::{LintPreset, Linter};

fn errors(template: &str) -> usize {
    Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec!["vue/no-v-text-v-html-on-component".into()]))
        .lint_template(template, "test.vue")
        .error_count
}

#[test]
fn unknown_lowercase_tags_are_components() {
    for source in [
        r#"<view class="bar" v-html="title"></view>"#,
        r#"<text v-text="label" />"#,
        r#"<my-element v-html="content" />"#,
        r#"<svg><foreignObject><view v-html="content" /></foreignObject></svg>"#,
    ] {
        assert_eq!(errors(source), 1, "{source}");
    }
}

#[test]
fn native_html_svg_and_math_tags_are_not_components() {
    for source in [
        r#"<section v-html="content"></section>"#,
        r#"<svg><text v-text="label" /></svg>"#,
        r#"<svg><g><view v-html="content" /></g></svg>"#,
        r#"<math><mi v-text="x" /></math>"#,
        r#"<template v-if="ok"><p v-html="content" /></template>"#,
    ] {
        assert_eq!(errors(source), 0, "{source}");
    }
}
