//! `vue/v-slot-style` behaviour, pinned finding by finding.
//!
//! The expectations are complete diagnostic lists, and the positions and
//! decisions are the ones `eslint-plugin-vue@10.9.2` reaches on the same input
//! under its own defaults: `v-slot` at a component, `#default` on a
//! `<template>`, `#name` for a named slot.

use super::{VSlotStyle, VSlotStyleOption};
use crate::diagnostic::Severity;
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn run(rule: VSlotStyle, source: &str) -> Vec<(Severity, u32, u32, &str, String)> {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(rule));
    Linter::with_registry(registry)
        .lint_template(source, "test.vue")
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.severity,
                diagnostic.start,
                diagnostic.end,
                &source[diagnostic.start as usize..diagnostic.end as usize],
                diagnostic.message.to_string(),
            )
        })
        .collect()
}

fn findings(source: &str) -> Vec<(Severity, u32, u32, &str, String)> {
    run(VSlotStyle::default(), source)
}

fn only_message(source: &str) -> Vec<String> {
    findings(source)
        .into_iter()
        .map(|finding| finding.4)
        .collect()
}

#[test]
fn a_bare_v_slot_on_a_component_is_the_expected_style() {
    assert_eq!(
        findings(r#"<MyComponent v-slot="{ item }"><p /></MyComponent>"#),
        vec![]
    );
}

#[test]
fn a_shorthand_default_slot_on_a_component_is_reported() {
    let source = r#"<MyComponent #default="{ item }"><p /></MyComponent>"#;
    assert_eq!(
        findings(source),
        vec![(
            Severity::Warning,
            13,
            32,
            r#"#default="{ item }""#,
            "Expected 'v-slot' instead of '#default'".to_string(),
        )]
    );
}

#[test]
fn a_longform_default_slot_on_a_component_is_reported() {
    assert_eq!(
        only_message(r#"<MyComponent v-slot:default="x"><p /></MyComponent>"#),
        vec!["Expected 'v-slot' instead of 'v-slot:default'".to_string()]
    );
}

#[test]
fn a_shorthand_named_slot_is_clean_anywhere() {
    assert_eq!(
        findings(r#"<MyComponent><template #header>H</template></MyComponent>"#),
        vec![]
    );
    assert_eq!(findings(r#"<MyComponent #header>H</MyComponent>"#), vec![]);
}

#[test]
fn a_longform_named_slot_is_reported() {
    assert_eq!(
        only_message(r#"<MyComponent><template v-slot:header>H</template></MyComponent>"#),
        vec!["Expected '#header' instead of 'v-slot:header'".to_string()]
    );
}

#[test]
fn a_bare_v_slot_on_a_template_is_reported() {
    assert_eq!(
        only_message(r#"<MyComponent><template v-slot>H</template></MyComponent>"#),
        vec!["Expected '#default' instead of 'v-slot'".to_string()]
    );
}

#[test]
fn a_shorthand_default_slot_on_a_template_is_clean() {
    assert_eq!(
        findings(r#"<MyComponent><template #default="x">H</template></MyComponent>"#),
        vec![]
    );
}

#[test]
fn a_dynamic_argument_is_a_named_slot() {
    assert_eq!(
        findings(r#"<MyComponent #[name]="x"><p /></MyComponent>"#),
        vec![]
    );
    assert_eq!(
        only_message(r#"<MyComponent v-slot:[name]="x"><p /></MyComponent>"#),
        vec!["Expected '#[name]' instead of 'v-slot:[name]'".to_string()]
    );
}

#[test]
fn a_uniform_longform_option_reports_every_shorthand() {
    let messages = run(
        VSlotStyle::uniform(VSlotStyleOption::Longform),
        r#"<MyComponent><template #header>H</template></MyComponent>"#,
    )
    .into_iter()
    .map(|finding| finding.4)
    .collect::<Vec<_>>();
    assert_eq!(
        messages,
        vec!["Expected 'v-slot:header' instead of '#header'".to_string()]
    );
}

#[test]
fn a_uniform_shorthand_option_reports_a_bare_v_slot_on_a_component() {
    let messages = run(
        VSlotStyle::uniform(VSlotStyleOption::Shorthand),
        r#"<MyComponent v-slot="x"><p /></MyComponent>"#,
    )
    .into_iter()
    .map(|finding| finding.4)
    .collect::<Vec<_>>();
    assert_eq!(
        messages,
        vec!["Expected '#default' instead of 'v-slot'".to_string()]
    );
}

#[test]
fn non_slot_directives_are_ignored() {
    assert_eq!(findings(r#"<div v-if="ok" :class="cls"></div>"#), vec![]);
}
