//! Duplicate IDs retain the pre-migration full packets and branch precedence.

use super::linter;
use crate::context::{LintContext, TemplateDocumentKind};
use crate::diagnostic::LintDiagnostic;
use crate::ir::TemplateSyntax;
use crate::linter::LintResult;
use crate::markup::{MarkupContext, MarkupDocument};
use crate::rules::html::id_duplication::IdDuplication;
use vize_atelier_jsx::JsxLang;
use vize_l0::{Allocator, ToCompactString};

mod original;

const TEMPLATES: &[&str] = &[
    r#"<div id="x"/><span id="x"/><b id="x"/>"#,
    r#"<div id="" id=""/><span id/><b :id="x"/>"#,
    r#"<div id="&fjlig;"/><span id="fj"/><b id="f"/>"#,
    r#"<div id="&NotEqualTilde;"/><span id="≂̸"/>"#,
    r#"<div v-if="x" id="x"/><!-- gap --> <div v-else-if="y" id="x"/><div v-else id="x"/><b id="x"/>"#,
    r#"<div v-else v-if="ok" id="x"/><div v-else id="x"/>"#,
    r#"<div v-if="ok" id="x"/><div v-if="again" v-else id="x"/><b v-else id="x"/>"#,
    r#"<div v-if="x" id="x"/>&#32;<!-- gap -->&#x9;<div v-else id="x"/>"#,
    r#"<div v-if="x" id="x"/>text<div v-else id="x"/>"#,
    r#"<div v-pre><span v-if="x" id="x"/><b v-else id="x"/></div>"#,
    r#"<div v-if="x"><span v-if="y" id="x"/><span v-else id="x"/></div><div v-else id="x"/><b id="x"/>"#,
    r#"<template v-for="item in items"><div id="x"/></template><span id="x"/>"#,
    r#"<svg><g id="x"/></svg><math><mtext id="x"/></math>"#,
    "<!-- eslint-disable-next-line html/id-duplication -->\r\n<div id='x' id='x'/><b id='x'/>",
];

const JSX: &[&str] = &[
    r#"const A = () => <><div id="x"/><span id="x"/></>;"#,
    r#"const A = () => <><div id="&fjlig;"/><span id="fj"/><b id="f"/></>;"#,
    r#"const A = () => <><div id="" id=""/><span id={value}/></>;"#,
    r#"const A = () => <>{ok ? <div id="x"/> : <span id="x"/>}<b id="x"/></>;"#,
    r#"const A = () => <>{ok && <div id="x"/>}<span id="x"/>{items.map(item => <b id="x"/>)}</>;"#,
    r#"const A = () => <Comp v-slots={{ head: () => <><div id="x"/><span id="x"/></> }}/>;"#,
    r#"const A = () => <Comp render={<span id="x"/>}><div id="x"/><span id="x"/></Comp>;"#,
    r#"const A = () => <><div v-else v-if={ok} id="x"/><span v-else id="x"/></>;"#,
];

#[test]
fn complete_vue_sfc_jsx_and_tsx_packets_match_the_frozen_original_rule() {
    let actual = linter(Box::new(IdDuplication));
    let baseline = linter(Box::new(original::IdDuplication));
    for source in TEMPLATES {
        assert_eq!(
            format!("{:?}", actual.lint_template(source, "Ids.vue")),
            format!("{:?}", baseline.lint_template(source, "Ids.vue")),
            "{source}"
        );
        let sfc = format!(
            "<script setup>const note='日本語😀'</script>\r\n<template>{source}</template>"
        );
        assert_eq!(
            format!("{:?}", actual.lint_sfc(&sfc, "Ids.vue")),
            format!("{:?}", baseline.lint_sfc(&sfc, "Ids.vue")),
            "{sfc}"
        );
    }
    for lang in [JsxLang::Jsx, JsxLang::Tsx] {
        for source in JSX {
            assert_eq!(
                format!("{:?}", actual.lint_jsx(source, "Ids.tsx", lang)),
                format!("{:?}", baseline.lint_jsx(source, "Ids.tsx", lang)),
                "{source}"
            );
        }
    }
    for (source, lang, filename) in super::jsx_corpus() {
        assert_eq!(
            format!("{:?}", actual.lint_jsx(&source, filename, lang)),
            format!("{:?}", baseline.lint_jsx(&source, filename, lang)),
            "{filename}: {source}"
        );
    }
}

#[test]
fn normal_html_surface_keeps_ignored_headers_and_branch_successors() {
    let baseline = linter(Box::new(original::IdDuplication));
    for source in [
        r#"<form id="x"><form id="x"></form></form>"#,
        r#"<form><div v-if="ok" id="x"></div><form id="x"><span v-else id="x"></span></form></form>"#,
        r#"<p id="x">text<div id="x"></div>after</p>"#,
        r#"<div v-if="ok" id="x"></div><? ignored ?><b v-else id="x"></b>"#,
        r#"<div v-pre><span v-if="ok" id="x"></span><b v-else id="x"></b></div>"#,
    ] {
        let allocator = Allocator::new();
        let input = crate::markup::LintInput::lower_for(
            &allocator,
            source,
            TemplateDocumentKind::StandaloneHtml,
        );
        let markup = input.markup();
        let document = MarkupDocument::from_l2(&markup, TemplateSyntax::Vue);
        let mut lint = LintContext::new(&allocator, source, "Ids.html");
        document.visit_with(
            &IdDuplication,
            &mut MarkupContext::new(&mut lint, &document),
        );
        let actual = LintResult {
            filename: "Ids.html".into(),
            error_count: lint.error_count(),
            warning_count: lint.warning_count(),
            diagnostics: lint.into_diagnostics(),
        };
        let expected = baseline.lint_standalone_html(source, "Ids.html");
        assert_eq!(format!("{actual:?}"), format!("{expected:?}"), "{source}");
    }
}

fn duplicate(source: &str, value: &str, first: &str, later: &str) -> LintDiagnostic {
    let first_start = source.find(first).unwrap() as u32;
    let start = source.rfind(later).unwrap() as u32;
    LintDiagnostic::error(
        "html/id-duplication",
        format!("Duplicate id=\"{value}\" found in template"),
        start,
        start + later.len() as u32,
    )
    .with_help("Element IDs must be unique within a document. Duplicate IDs break label associations, ARIA references, and getElementById().".to_compact_string())
    .with_label("first defined here".to_compact_string(), first_start, first_start + first.len() as u32)
}

#[test]
fn full_entity_expansion_and_first_coexisting_label_keep_fixed_packets() {
    let source = r#"<div id="&fjlig;"/><span id="f"/><b id="fj"/>"#;
    let expected = LintResult {
        filename: "Ids.vue".into(),
        diagnostics: vec![duplicate(source, "fj", r#"id="&fjlig;""#, r#"id="fj""#)],
        error_count: 1,
        warning_count: 0,
    };
    let actual = linter(Box::new(IdDuplication)).lint_template(source, "Ids.vue");
    assert_eq!(format!("{actual:?}"), format!("{expected:?}"));

    let source = r#"<div v-if="ok" id="x"/><span v-else id="x"/><b id="x"/>"#;
    let expected = LintResult {
        filename: "Ids.vue".into(),
        diagnostics: vec![duplicate(source, "x", r#"id="x""#, r#"id="x""#)],
        error_count: 1,
        warning_count: 0,
    };
    let actual = linter(Box::new(IdDuplication)).lint_template(source, "Ids.vue");
    assert_eq!(format!("{actual:?}"), format!("{expected:?}"));

    // The earlier v-if ID cannot coexist with either v-else child. The
    // diagnostic must skip it and label the first child of the same branch.
    let source = r#"<div v-if="ok" id="x"/><section v-else><b id = "x"/><i id="x"/></section>"#;
    let expected = LintResult {
        filename: "Ids.vue".into(),
        diagnostics: vec![duplicate(source, "x", r#"id = "x""#, r#"id="x""#)],
        error_count: 1,
        warning_count: 0,
    };
    let actual = linter(Box::new(IdDuplication)).lint_template(source, "Ids.vue");
    assert_eq!(format!("{actual:?}"), format!("{expected:?}"));
}

#[test]
fn tsrx_runs_shared_id_semantics_with_the_complete_packet() {
    let source = "export function Page() @{\n  <div id=\"x\"/><span id=\"x\"/>\n}\n";
    let expected = LintResult {
        filename: "Ids.tsrx".into(),
        diagnostics: vec![duplicate(source, "x", r#"id="x""#, r#"id="x""#)],
        error_count: 1,
        warning_count: 0,
    };
    let actual = linter(Box::new(IdDuplication)).lint_tsrx(source, "Ids.tsrx");
    assert_eq!(format!("{actual:?}"), format!("{expected:?}"));
}
