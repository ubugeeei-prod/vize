#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "test lines are rendered with std formatting"
)]

use super::{Finding, nuxt_template_findings};

const FIXTURE: &str = include_str!(
    "../../../../../tests/_fixtures/differential/nuxt/template-diagnostics/App.vue.txt"
);
/// Nuxt 4.7.0's own transform on `App.vue.txt` through `@vue/compiler-sfc` 3.5,
/// with no page route matching and `ServerCard` as a server component.
const NUXT_REFERENCE: &str = include_str!(
    "../../../../../tests/_fixtures/differential/nuxt/template-diagnostics/nuxt-4.7.expected.txt"
);
const SERVER_TAGS: [&str; 2] = ["ServerCard", "server-card"];

fn render(findings: &[Finding]) -> String {
    let mut lines = String::new();
    for finding in findings {
        lines.push_str(&format!(
            "{} <{}> {}:{} @{}",
            finding.kind.as_str(),
            finding.tag,
            finding.line,
            finding.column,
            finding.offset
        ));
        let fields = [
            ("attribute", finding.attribute.as_deref()),
            ("to", finding.to.as_deref()),
            ("directive", finding.directive.as_deref()),
            ("prop", finding.prop.as_deref()),
            ("value", finding.value),
            ("name", finding.island_name.as_deref()),
            ("bind", finding.island_prop.as_deref()),
        ];
        for (key, value) in fields {
            if let Some(value) = value {
                lines.push_str(&format!(" {key}={value:?}"));
            }
        }
        lines.push('\n');
    }
    lines
}

fn findings(template: &str, script: &str) -> String {
    let source = format!(
        "<script setup lang=\"ts\">\n{script}\n</script>\n<template>{template}</template>\n"
    );
    render(&nuxt_template_findings(&source, &SERVER_TAGS))
}

/// A finding as Nuxt reports it once no route matches and every candidate
/// tag is a server component: `code line:column @offset params`.
fn render_as_nuxt(findings: &[Finding]) -> String {
    let quote = |text: &str| serde_json::to_string(text).unwrap();
    let mut lines = String::new();
    for finding in findings {
        let field = |value: Option<&str>| quote(value.unwrap_or_default());
        let (code, params) = match finding.kind.as_str() {
            "link" => (
                "NUXT_B4025",
                format!(
                    "\"tag\":{},\"attribute\":{},\"to\":{}",
                    quote(&finding.tag),
                    field(finding.attribute.as_deref()),
                    field(finding.to.as_deref())
                ),
            ),
            "event" => (
                "NUXT_B3014",
                format!(
                    "\"component\":{},\"directive\":{}",
                    quote(&finding.tag),
                    field(finding.directive.as_deref())
                ),
            ),
            kind => {
                let component = match (kind, finding.island_name.as_deref()) {
                    ("island", Some(name)) => format!("{} name=\"{name}\"", finding.tag),
                    _ => finding.tag.to_string(),
                };
                (
                    "NUXT_B3015",
                    format!(
                        "\"component\":{},\"prop\":{},\"value\":{}",
                        quote(&component),
                        field(finding.prop.as_deref()),
                        field(finding.value)
                    ),
                )
            }
        };
        lines.push_str(&format!(
            "{code} {}:{} @{} {{{params}}}\n",
            finding.line, finding.column, finding.offset
        ));
    }
    lines
}

#[test]
fn fixture_findings_match_nuxt_template_diagnostics() {
    assert_eq!(
        render_as_nuxt(&nuxt_template_findings(FIXTURE, &SERVER_TAGS)),
        NUXT_REFERENCE
    );
}

#[test]
fn server_checks_need_the_tag_to_be_a_server_component() {
    let source = FIXTURE.replace("ServerCard", "ClientCard");
    let findings = nuxt_template_findings(&source, &SERVER_TAGS);
    let tagged = |tag: &str| findings.iter().filter(|finding| finding.tag == tag).count();
    assert_eq!((tagged("ClientCard"), tagged("server-card")), (0, 3));
}

#[test]
fn a_tag_the_script_binds_is_not_checked() {
    let rendered = findings(
        "<NuxtLink to=\"/a\" /><ServerCard @click=\"go\" />",
        "import { NuxtLink } from \"#components\";\nimport ServerCard from \"./ServerCard.vue\";\nconst go = () => {};",
    );
    assert_eq!(rendered, "");
}

#[test]
fn names_only_follow_constant_setup_declarations() {
    let rendered = findings(
        "<ServerCard :a=\"later\" :b=\"wrapped\" :c=\"imported\" :d=\"declared\" :e=\"maker\" />",
        "import { imported } from \"./x\";\nlet later = () => {};\nconst wrapped = (() => {});\ndeclare function declared(): void;\nconst maker = function () {};",
    );
    assert_eq!(
        rendered,
        "prop <ServerCard> 8:75 @256 prop=\"e\" value=\"a function\"\n"
    );
}

#[test]
fn same_name_shorthand_and_empty_values_are_not_described() {
    let rendered = findings(
        "<ServerCard :maker :b=\"\" :[dynamic]=\"maker\" v-bind=\"{ maker }\" />",
        "const maker = () => {};\nconst dynamic = 'x';",
    );
    assert_eq!(rendered, "");
}

#[test]
fn unparsed_sfcs_and_preprocessed_templates_yield_nothing() {
    assert!(
        nuxt_template_findings("<template><NuxtLink to=\"/a\"></template", &SERVER_TAGS).is_empty()
    );
    assert!(
        nuxt_template_findings(
            "<template lang=\"pug\">NuxtLink(to=\"/a\")</template>",
            &SERVER_TAGS
        )
        .is_empty()
    );
    assert!(
        nuxt_template_findings("<script setup>\nconst a = 1\n</script>", &SERVER_TAGS).is_empty()
    );
}
