use super::super::fix::apply_lint_fixes;
use vize_patina::Linter;

#[test]
fn apply_lint_fixes_applies_existing_rule_fixes() {
    let source = r#"<template><button v-on:click="save">Save</button></template>"#;
    let result = Linter::new().lint_sfc(source, "App.vue");
    let fixed = apply_lint_fixes(source, &result).expect("fix should be available");

    assert_eq!(
        fixed.as_str(),
        r#"<template><button @click="save">Save</button></template>"#
    );
}
