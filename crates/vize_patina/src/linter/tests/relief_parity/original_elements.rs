//! Source ranges and element kinds retain independent original rule packets.

use super::linter;
use crate::rule::Rule;
use crate::rules::vue::configured::{MaxTemplateDepth, NoRestrictedHtmlElements};
use vize_atelier_jsx::JsxLang;
use vize_l0::config::ConfigLintRuleOptions;

mod custody;
mod original;

fn family(original: bool) -> Vec<Box<dyn Rule>> {
    let parsed: ConfigLintRuleOptions = serde_json::from_value(serde_json::json!({
        "vue/max-template-depth": { "maxDepth": 1 },
        "vue/no-restricted-html-elements": [
            "div", "component",
            { "element": ["span", "Comp", "slot", "template"], "message": "Use a supported element." },
            { "element": "div", "message": "Never reached." }
        ],
    })).unwrap();
    let options = parsed.configured_vue();
    if original {
        vec![
            Box::new(original::MaxTemplateDepth::from_options(
                options.max_template_depth.as_ref().unwrap(),
            )),
            Box::new(original::NoRestrictedHtmlElements::from_options(
                options.no_restricted_html_elements.as_ref().unwrap(),
            )),
        ]
    } else {
        vec![
            Box::new(MaxTemplateDepth::from_options(
                options.max_template_depth.as_ref().unwrap(),
            )),
            Box::new(NoRestrictedHtmlElements::from_options(
                options.no_restricted_html_elements.as_ref().unwrap(),
            )),
        ]
    }
}

const TEMPLATES: &[&str] = &[
    r#"<div><span/><div><span/></div></div>"#,
    r#"<Comp><slot/><template v-if="x"><span/></template></Comp>"#,
    r#"<Comp v-pre/><slot v-pre/><template v-if="x" v-pre/>"#,
    r#"<div v-pre><Comp/><slot/><template v-if="x"/></div>"#,
    r#"<svg><Comp/><foreignObject><Comp/><span/></foreignObject></svg><math><mtext><Comp/><span/></mtext></math>"#,
    r#"<div><template v-for="item in items"><span/></template><component :is="tag"/></div>"#,
    "<!-- eslint-disable-next-line vue/no-restricted-html-elements, vue/max-template-depth -->\r\n<div><span/></div>",
];

const JSX: &[&str] = &[
    r#"const A = () => <div><span/><div><span/></div></div>;"#,
    r#"const A = () => <Comp v-slots={{ head: () => <div><span/></div> }} />;"#,
    r#"const A = () => <><component is={tag}/><component is="literal"/><Bound/><Lib.Component/></>;"#,
    r#"const Bound = () => null; const A = () => <Bound><div><span/></div></Bound>;"#,
    r#"const A = () => <div>{ok ? <><span/><div/></> : <span/>}{items.map(item => <><div/><span/></>)}</div>;"#,
    r#"const A = () => <Comp render={<div><span/></div>}><span/></Comp>;"#,
    r#"const A = () => <other:div><span/></other:div>;"#,
];

#[test]
fn whole_vue_sfc_jsx_and_tsx_results_match_the_frozen_original_rules() {
    for (rule, baseline) in family(false).into_iter().zip(family(true)) {
        let name = rule.meta().name;
        let actual = linter(rule);
        let original = linter(baseline);
        for source in TEMPLATES {
            assert_eq!(
                format!("{:?}", actual.lint_template(source, "Elements.vue")),
                format!("{:?}", original.lint_template(source, "Elements.vue")),
                "{name}: {source}"
            );
            let sfc = format!(
                "<script setup>const note='日本語😀'</script>\r\n<template>{source}</template>"
            );
            assert_eq!(
                format!("{:?}", actual.lint_sfc(&sfc, "Elements.vue")),
                format!("{:?}", original.lint_sfc(&sfc, "Elements.vue")),
                "{name}: {sfc}"
            );
        }
        for lang in [JsxLang::Jsx, JsxLang::Tsx] {
            for source in JSX {
                assert_eq!(
                    format!("{:?}", actual.lint_jsx(source, "Elements.tsx", lang)),
                    format!("{:?}", original.lint_jsx(source, "Elements.tsx", lang)),
                    "{name}: {source}"
                );
            }
        }
        for (source, lang, filename) in super::jsx_corpus() {
            assert_eq!(
                format!("{:?}", actual.lint_jsx(&source, filename, lang)),
                format!("{:?}", original.lint_jsx(&source, filename, lang)),
                "{name}: {filename}: {source}"
            );
        }
        for source in [
            "<form><div><form><span/></form></div></form>",
            "<table><tr><td><span/></td></tr></table>",
            "<p>text<div><span/></div>after</p>",
        ] {
            assert_eq!(
                format!("{:?}", actual.lint_standalone_html(source, "Elements.html")),
                format!(
                    "{:?}",
                    original.lint_standalone_html(source, "Elements.html")
                ),
                "{name}: {source}"
            );
        }
    }
}
