//! Static-attribute sharing keeps normal L1 spellings and callback phase.

use super::{ReliefOnly, linter};
use crate::context::{LintContext, TemplateDocumentKind};
use crate::diagnostic::LintDiagnostic;
use crate::ir::TemplateSyntax;
use crate::linter::{LintResult, Linter};
use crate::markup::{MarkupContext, MarkupDocument};
use crate::rule::{Rule, RuleRegistry};
use crate::rules::vue::configured::{
    MaxTemplateDepth, NoRestrictedHtmlElements, NoRestrictedStaticAttribute,
};
use vize_atelier_jsx::JsxLang;
use vize_l0::{Allocator, config::ConfigLintRuleOptions};

fn family() -> Vec<Box<dyn Rule>> {
    let parsed: ConfigLintRuleOptions = serde_json::from_value(serde_json::json!({
        "vue/max-template-depth": { "maxDepth": 1 },
        "vue/no-restricted-html-elements": [
            "div",
            { "element": ["span", "Comp", "slot", "template"], "message": "Use a supported element." },
            { "element": "div", "message": "Never reached." }
        ],
        "vue/no-restricted-static-attribute": ["legacy", ":legacy", "v-bind:legacy", "v-pre", "other:legacy"]
    })).unwrap();
    let options = parsed.configured_vue();
    vec![Box::new(NoRestrictedStaticAttribute::from_options(
        options.no_restricted_static_attribute.as_ref().unwrap(),
    ))]
}

const TEMPLATES: &[&str] = &[
    r#"<div legacy legacy="two"><span legacy/><span :legacy="value"/></div>"#,
    r#"<Comp v-pre legacy/><slot v-pre legacy/><template v-if="x" v-pre legacy/>"#,
    r#"<div v-pre><span v-pre :legacy="value" v-bind:legacy="value"/></div>"#,
    r#"<div><Comp/><slot legacy/><template v-if="x"><span legacy/></template><template v-for="item in items"><div legacy/></template></div>"#,
    r#"<svg><Comp legacy/><foreignObject><Comp legacy/><span legacy/></foreignObject></svg><math><mtext><Comp legacy/><span legacy/></mtext></math>"#,
    r#"<div legacy=><span legacy='日本語😀'/><span v-bind:legacy='value'/><div :[legacy]='value'/></div>"#,
    "<!-- eslint-disable-next-line vue/no-restricted-static-attribute -->\r\n<div legacy><span legacy/></div>",
];

const JSX: &[&str] = &[
    r#"const A = () => <div legacy legacy="two"><span legacy/><span legacy={value}/></div>;"#,
    r#"const A = () => <><Comp legacy/><slot legacy/><template legacy><span legacy/></template></>;"#,
    r#"const A = () => <div>{ok ? <span legacy/> : <div legacy/>}{items.map(item => <span legacy key={item}/>)}</div>;"#,
    r#"const A = () => <Component render={<div legacy/>}><span other:legacy="x"/><span legacy/></Component>;"#,
    r#"const A = () => <other:div legacy/>;"#,
];

#[test]
fn configured_header_rules_preserve_complete_template_sfc_and_jsx_family_packets() {
    for (rule, baseline) in family().into_iter().zip(family()) {
        let name = rule.meta().name;
        let actual = linter(rule);
        let legacy = linter(Box::new(ReliefOnly(baseline)));
        for source in TEMPLATES {
            assert_eq!(
                format!("{:?}", actual.lint_template(source, "Headers.vue")),
                format!("{:?}", legacy.lint_template(source, "Headers.vue")),
                "{name}: {source}"
            );
            let source = format!(
                "<script setup>const note='日本語😀'</script>\r\n<template>{source}</template>"
            );
            assert_eq!(
                format!("{:?}", actual.lint_sfc(&source, "Headers.vue")),
                format!("{:?}", legacy.lint_sfc(&source, "Headers.vue")),
                "{name}: {source}"
            );
        }
        for lang in [JsxLang::Jsx, JsxLang::Tsx] {
            for source in JSX {
                assert_eq!(
                    format!("{:?}", actual.lint_jsx(source, "Headers.tsx", lang)),
                    format!("{:?}", legacy.lint_jsx(source, "Headers.tsx", lang)),
                    "{name}: {source}"
                );
            }
        }
    }
}

#[test]
fn normal_in_dom_headers_keep_repaired_ancestry_and_ignored_start_tags() {
    for (rule, baseline) in family().into_iter().zip(family()) {
        let name = rule.meta().name;
        let legacy = linter(Box::new(ReliefOnly(baseline)));
        for source in [
            r#"<!DOCTYPE html><html><body><div legacy><span legacy/></div></body></html>"#,
            r#"<form><div legacy><form><span legacy/></form></div></form>"#,
            r#"<table><tr legacy><td><span legacy/></td></tr></table>"#,
            r#"<p legacy>text<div legacy><span legacy/></div>after</p>"#,
            r#"<div v-pre><span v-pre :legacy="x"/></div>"#,
        ] {
            let allocator = Allocator::new();
            let lowered = crate::markup::LintInput::lower_for(
                &allocator,
                source,
                TemplateDocumentKind::StandaloneHtml,
            );
            let markup = lowered.markup();
            let document = MarkupDocument::from_l2(&markup, TemplateSyntax::Vue);
            let mut lint = LintContext::new(&allocator, source, "Headers.html");
            document.visit_with(
                rule.as_markup_rule().unwrap(),
                &mut MarkupContext::new(&mut lint, &document),
            );
            let actual = LintResult {
                filename: "Headers.html".into(),
                error_count: lint.error_count(),
                warning_count: lint.warning_count(),
                diagnostics: lint.into_diagnostics(),
            };
            let expected = legacy.lint_standalone_html(source, "Headers.html");
            assert_eq!(
                format!("{actual:?}"),
                format!("{expected:?}"),
                "{name}: {source}"
            );
        }
    }
}

fn combined() -> Linter {
    let mut registry = RuleRegistry::new();
    for rule in family() {
        registry.register(rule);
    }
    Linter::with_registry(registry)
}

#[test]
fn tsrx_runs_the_actual_configured_instances_with_complete_name_only_attribute_packets() {
    let source = "export function Page() @{\n  <div legacy><span legacy/></div>\n}\n";
    let first_attribute = source.find("legacy").unwrap() as u32;
    let second_attribute = source.rfind("legacy").unwrap() as u32;
    let mut expected = LintResult {
        filename: "Headers.tsrx".into(),
        diagnostics: vec![
            LintDiagnostic::warn(
                "vue/no-restricted-static-attribute",
                "Using `legacy` is not allowed.",
                first_attribute,
                first_attribute + 6,
            ),
            LintDiagnostic::warn(
                "vue/no-restricted-static-attribute",
                "Using `legacy` is not allowed.",
                second_attribute,
                second_attribute + 6,
            ),
        ],
        error_count: 0,
        warning_count: 2,
    };
    expected
        .diagnostics
        .sort_by_key(|diagnostic| (diagnostic.start, diagnostic.end, diagnostic.rule_name));
    let actual = combined().lint_tsrx(source, "Headers.tsrx");
    assert_eq!(format!("{actual:?}"), format!("{expected:?}"));
}

#[test]
fn unmigrated_max_depth_keeps_the_original_synthesized_slot_function_packet() {
    let source = r#"const A = () => <Comp v-slots={{ head: () => <div><span/></div> }} />;"#;
    let rule = MaxTemplateDepth::from_options(&vize_l0::config::MaxTemplateDepthOptions {
        max_depth: Some(1),
    });
    let function = source.find("() => <div>").unwrap() as u32;
    let div = source.find("<div>").unwrap() as u32;
    let end = (source.find("</div>").unwrap() + "</div>".len()) as u32;
    let span = source.find("<span/>").unwrap() as u32;
    let expected = LintResult {
        filename: "Slot.tsx".into(),
        diagnostics: vec![
            LintDiagnostic::warn(
                "vue/max-template-depth",
                "Element is nested too deeply (depth of 2, maximum allowed is 1).",
                function,
                end,
            ),
            LintDiagnostic::warn(
                "vue/max-template-depth",
                "Element is nested too deeply (depth of 3, maximum allowed is 1).",
                div,
                end,
            ),
            LintDiagnostic::warn(
                "vue/max-template-depth",
                "Element is nested too deeply (depth of 4, maximum allowed is 1).",
                span,
                span + 7,
            ),
        ],
        error_count: 0,
        warning_count: 3,
    };
    let actual = linter(Box::new(rule)).lint_jsx(source, "Slot.tsx", JsxLang::Tsx);
    assert_eq!(format!("{actual:?}"), format!("{expected:?}"));
}

#[test]
fn unmigrated_restricted_elements_keep_the_original_lowercase_component_packet() {
    let source = r#"const A = () => <component is={tag} />;"#;
    let parsed: ConfigLintRuleOptions = serde_json::from_value(serde_json::json!({
        "vue/no-restricted-html-elements": ["component"]
    }))
    .unwrap();
    let rule = NoRestrictedHtmlElements::from_options(
        parsed
            .configured_vue()
            .no_restricted_html_elements
            .as_ref()
            .unwrap(),
    );
    let start = source.find("<component").unwrap() as u32;
    let end = (source.find("/>").unwrap() + 2) as u32;
    let expected = LintResult {
        filename: "Component.tsx".into(),
        diagnostics: vec![LintDiagnostic::warn(
            "vue/no-restricted-html-elements",
            "Unexpected use of forbidden element component.",
            start,
            end,
        )],
        error_count: 0,
        warning_count: 1,
    };
    let actual = linter(Box::new(rule)).lint_jsx(source, "Component.tsx", JsxLang::Tsx);
    assert_eq!(format!("{actual:?}"), format!("{expected:?}"));
}
