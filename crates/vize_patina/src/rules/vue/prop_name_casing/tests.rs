//! `vue/prop-name-casing` behaviour, pinned finding by finding.
//!
//! The rule reads the component's own `defineProps` declaration, exactly as
//! `eslint-plugin-vue@10.9.2` does, so every expectation is a complete
//! diagnostic list over a whole SFC with the reported range resolved back to the
//! declaration text it must cover.

use crate::diagnostic::Severity;
use crate::linter::{LintResult, Linter};

fn lint_sfc(sfc: &str) -> LintResult {
    Linter::new()
        .with_enabled_rules(Some(vec!["vue/prop-name-casing".into()]))
        .lint_sfc(sfc, "Probe.vue")
}

fn findings(sfc: &str) -> Vec<(&'static str, Severity, u32, u32, String)> {
    lint_sfc(sfc)
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.rule_name,
                diagnostic.severity,
                diagnostic.start,
                diagnostic.end,
                diagnostic.message.to_string(),
            )
        })
        .collect()
}

/// The finding `name` must produce, anchored at its written declaration.
fn reported(
    sfc: &str,
    name: &str,
    declaration: &str,
) -> (&'static str, Severity, u32, u32, String) {
    assert_eq!(
        sfc.matches(declaration).count(),
        1,
        "declaration {declaration:?} must occur exactly once"
    );
    let start = sfc.find(declaration).expect("prop declaration") as u32;
    (
        "vue/prop-name-casing",
        Severity::Warning,
        start,
        start + declaration.len() as u32,
        format!("Prop \"{name}\" is not in camelCase"),
    )
}

fn none() -> Vec<(&'static str, Severity, u32, u32, String)> {
    Vec::new()
}

#[test]
fn a_camel_case_runtime_object_declaration_is_clean() {
    let sfc = "<script setup>\ndefineProps({ myProp: String })\n</script>\n\n<template><p /></template>\n";
    assert_eq!(findings(sfc), none());
}

#[test]
fn a_kebab_case_runtime_object_declaration_is_reported() {
    let sfc = "<script setup>\ndefineProps({ 'my-prop': String })\n</script>\n\n<template><p /></template>\n";
    assert_eq!(
        findings(sfc),
        vec![reported(sfc, "my-prop", "'my-prop': String")]
    );
}

#[test]
fn a_snake_case_declaration_is_reported_under_the_camel_case_default() {
    let sfc = "<script setup>\ndefineProps({ my_prop: String })\n</script>\n\n<template><p /></template>\n";
    assert_eq!(
        findings(sfc),
        vec![reported(sfc, "my_prop", "my_prop: String")]
    );
}

#[test]
fn a_single_lowercase_word_is_camel_case() {
    let sfc =
        "<script setup>\ndefineProps({ count: Number })\n</script>\n\n<template><p /></template>\n";
    assert_eq!(findings(sfc), none());
}

#[test]
fn a_template_attribute_written_in_camel_case_is_not_this_rule() {
    // `vue/attribute-hyphenation` owns the template side; reporting it here
    // duplicated that rule under a name upstream uses for the declaration.
    let sfc = "<script setup>\ndefineProps({ myProp: String })\n</script>\n\n<template><Child myProp=\"x\" /></template>\n";
    assert_eq!(findings(sfc), none());
}

#[test]
fn every_offending_prop_of_a_declaration_is_reported() {
    let sfc = "<script setup>\ndefineProps({ my_prop: String, ok: Number, 'other-prop': Boolean })\n</script>\n\n<template><p /></template>\n";
    assert_eq!(
        findings(sfc),
        vec![
            reported(sfc, "my_prop", "my_prop: String"),
            reported(sfc, "other-prop", "'other-prop': Boolean"),
        ]
    );
}

#[test]
fn a_component_without_props_is_clean() {
    let sfc = "<script setup>\nconst x = 1\n</script>\n\n<template><p>{{ x }}</p></template>\n";
    assert_eq!(findings(sfc), none());
}
