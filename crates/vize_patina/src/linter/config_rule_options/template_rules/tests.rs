//! Configured template rules reached through `linter.ruleOptions`.

use crate::{LintPreset, LintResult, Linter};
use vize_l0::String;
use vize_l0::config::ConfigLintRuleOptions;

fn options(value: serde_json::Value) -> ConfigLintRuleOptions {
    serde_json::from_value(value).unwrap()
}

fn linter(rule: &str, value: serde_json::Value) -> Linter {
    Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![String::from(rule)]))
        .with_config_rule_options(&options(value))
}

fn findings(result: &LintResult) -> Vec<(&'static str, String, u32, u32)> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.rule_name,
                diagnostic.message.clone(),
                diagnostic.start,
                diagnostic.end,
            )
        })
        .collect()
}

/// Ranges of the rule findings; parser diagnostics are not part of the oracle.
fn spans(result: &LintResult) -> Vec<(u32, u32)> {
    result
        .diagnostics
        .iter()
        .filter(|diagnostic| !diagnostic.rule_name.starts_with("parser/"))
        .map(|diagnostic| (diagnostic.start, diagnostic.end))
        .collect()
}

#[test]
fn options_alone_do_not_enable_a_template_rule() {
    let linter = Linter::with_preset(LintPreset::Incremental).with_config_rule_options(&options(
        serde_json::json!({
            "vue/no-duplicate-attributes": { "allowCoexistClass": false },
            "vue/no-multiple-template-root": { "disallowComments": true },
            "vue/no-lone-template": { "ignoreAccessible": true },
            "vue/no-static-inline-styles": { "allowBinding": true },
            "vue/no-unused-components": { "ignoreWhenBindingPresent": false },
            "vue/no-v-html": { "ignorePattern": "^safe" }
        }),
    ));
    let result = linter.lint_template(
        r#"<!-- a --><div class="a" :class="b" style="c" v-html="html"><div><template id="x" /></div></div>"#,
        "Probe.vue",
    );
    assert_eq!(findings(&result), Vec::new());
}

#[test]
fn duplicate_attributes_coexistence_follows_each_flag() {
    let source = r#"<div class="a" :class="b" style="c" :style="d" />"#;
    let default = linter("vue/no-duplicate-attributes", serde_json::json!({}));
    assert_eq!(
        spans(&default.lint_template(source, "Probe.vue")),
        Vec::new()
    );

    let class_only = linter(
        "vue/no-duplicate-attributes",
        serde_json::json!({ "vue/no-duplicate-attributes": { "allowCoexistClass": false } }),
    );
    assert_eq!(
        findings(&class_only.lint_template(source, "Probe.vue")),
        vec![(
            "vue/no-duplicate-attributes",
            String::from("Duplicate attribute 'v-bind:class'"),
            15,
            25,
        )]
    );

    let neither = linter(
        "vue/no-duplicate-attributes",
        serde_json::json!({
            "vue/no-duplicate-attributes": { "allowCoexistClass": false, "allowCoexistStyle": false }
        }),
    );
    assert_eq!(
        spans(&neither.lint_template(source, "Probe.vue")),
        vec![(15, 25), (36, 46)]
    );

    // Two static spellings stay duplicates whatever the flags say.
    let allowed = linter(
        "vue/no-duplicate-attributes",
        serde_json::json!({
            "vue/no-duplicate-attributes": { "allowCoexistClass": true, "allowCoexistStyle": true }
        }),
    );
    assert_eq!(
        spans(&allowed.lint_template(r#"<div class="a" class="b" />"#, "Probe.vue")),
        vec![(15, 24)]
    );
}

#[test]
fn template_root_comments_are_reported_only_when_disallowed() {
    let source = "<!-- lead --><div><!-- inner --></div>\n<!-- tail -->";
    let default = linter(
        "vue/no-multiple-template-root",
        serde_json::json!({ "vue/no-multiple-template-root": {} }),
    );
    assert_eq!(
        spans(&default.lint_template(source, "Probe.vue")),
        Vec::new()
    );

    let configured = linter(
        "vue/no-multiple-template-root",
        serde_json::json!({ "vue/no-multiple-template-root": { "disallowComments": true } }),
    );
    let result = configured.lint_template(source, "Probe.vue");
    assert_eq!(
        findings(&result),
        vec![
            (
                "vue/no-multiple-template-root",
                String::from("The template root disallows comments."),
                0,
                13,
            ),
            (
                "vue/no-multiple-template-root",
                String::from("The template root disallows comments."),
                39,
                52,
            ),
        ]
    );
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.fix.is_none())
    );

    // Root comments are reported alongside the root-shape finding.
    let result = configured.lint_template("<!-- a --><div /><div />", "Probe.vue");
    assert_eq!(spans(&result), vec![(0, 10), (17, 24)]);
}

#[test]
fn lone_template_ignore_accessible_reaches_the_rule() {
    let source =
        r#"<div><template ref="row"><span /></template><template><span /></template></div>"#;
    let default = linter("vue/no-lone-template", serde_json::json!({}));
    assert_eq!(spans(&default.lint_template(source, "Probe.vue")).len(), 2);

    let configured = linter(
        "vue/no-lone-template",
        serde_json::json!({ "vue/no-lone-template": { "ignoreAccessible": true } }),
    );
    assert_eq!(
        spans(&configured.lint_template(source, "Probe.vue")).len(),
        1
    );
}

#[test]
fn static_inline_styles_allow_binding_keeps_the_attribute_report() {
    let source = r#"<div style="color: red" :style="{ width: '1px' }" />"#;
    let default = linter("vue/no-static-inline-styles", serde_json::json!({}));
    assert_eq!(spans(&default.lint_template(source, "Probe.vue")).len(), 2);

    let configured = linter(
        "vue/no-static-inline-styles",
        serde_json::json!({ "vue/no-static-inline-styles": { "allowBinding": true } }),
    );
    assert_eq!(
        spans(&configured.lint_template(source, "Probe.vue")),
        vec![(5, 23)]
    );
}

#[test]
fn unused_components_can_report_despite_a_dynamic_is_binding() {
    let sfc = "<script>\nimport Used from './Used.vue'\nimport Unused from './Unused.vue'\nexport default { components: { Used, Unused }, data: () => ({ current: 'Used' }) }\n</script>\n<template><component :is=\"current\" /><Used /></template>\n";
    let lint = |value| {
        linter("vue/no-unused-components", value)
            .lint_sfc(sfc, "Probe.vue")
            .diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.message)
            .collect::<Vec<_>>()
    };
    assert_eq!(lint(serde_json::json!({})), Vec::<String>::new());
    assert_eq!(
        lint(
            serde_json::json!({ "vue/no-unused-components": { "ignoreWhenBindingPresent": true } })
        ),
        Vec::<String>::new()
    );
    assert_eq!(
        lint(
            serde_json::json!({ "vue/no-unused-components": { "ignoreWhenBindingPresent": false } })
        ),
        vec![String::from(
            "Component 'Unused' is registered but never used in template"
        )]
    );
}

#[test]
fn v_html_ignore_pattern_matches_names_and_expression_text() {
    let source = r#"<div><p v-html="safeHtml" /><p v-html="(safeHtml)" /><p v-html="render(safe)" /><p v-html="safe.html" /><p v-html="raw" /></div>"#;
    let default = linter("vue/no-v-html", serde_json::json!({}));
    assert_eq!(spans(&default.lint_template(source, "Probe.vue")).len(), 5);

    let configured = linter(
        "vue/no-v-html",
        serde_json::json!({ "vue/no-v-html": { "ignorePattern": "^safe" } }),
    );
    let raw = source.find(r#"v-html="raw""#).unwrap() as u32;
    let call = source.find(r#"v-html="render(safe)""#).unwrap() as u32;
    assert_eq!(
        spans(&configured.lint_template(source, "Probe.vue")),
        vec![(call, call + 21), (raw, raw + 12)]
    );

    // An empty pattern ignores nothing, as upstream treats it as absent.
    let empty = linter(
        "vue/no-v-html",
        serde_json::json!({ "vue/no-v-html": { "ignorePattern": "" } }),
    );
    assert_eq!(spans(&empty.lint_template(source, "Probe.vue")).len(), 5);
}
