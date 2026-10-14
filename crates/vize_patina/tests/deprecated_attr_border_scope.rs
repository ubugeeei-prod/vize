//! `html/deprecated-attr`: `border` is obsolete only on `<img>` and `<object>`
//! (and `<table>` keeps its conforming exception); Vue's `<component>` is never
//! an HTML element.

use vize_patina::{LintPreset, Linter};

fn warnings(template: &str) -> usize {
    Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec!["html/deprecated-attr".into()]))
        .lint_template(template, "test.vue")
        .warning_count
}

#[test]
fn border_is_reported_on_img_and_object_only() {
    assert_eq!(warnings(r#"<img border="0" src="a.png">"#), 1);
    assert_eq!(warnings(r#"<object border="1" data="a.svg"></object>"#), 1);
    assert_eq!(warnings(r#"<table border="1"></table>"#), 0);
    assert_eq!(
        warnings(r#"<div border="b base"><span border="t" /></div>"#),
        0
    );
}

#[test]
fn dynamic_component_is_not_an_html_element() {
    assert_eq!(
        warnings(r#"<component :is="tag" border="t" align="center" />"#),
        0
    );
    assert_eq!(warnings(r#"<div align="center" />"#), 1);
}
