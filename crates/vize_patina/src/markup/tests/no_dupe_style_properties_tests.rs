use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::opinionated::html::NoDupeStyleProperties;
use vize_atelier_jsx::JsxLang;
use vize_s0::Allocator;

fn run_over_template<R: MarkupRule>(rule: &R, source: &str) -> usize {
    let allocator = Allocator::with_capacity(source.len() * 4 + 1024);
    let parser = vize_armature::Parser::new(&allocator, source);
    let (root, _errors) = parser.parse();
    let document = MarkupDocument::new(&root, TemplateSyntax::Vue);

    let mut lint = LintContext::new(&allocator, source, "test.vue");
    let mut ctx = MarkupContext::new(&mut lint, &document);
    document.visit_with(rule, &mut ctx);
    lint.diagnostics().len()
}

fn run_over_jsx_lowered<R: MarkupRule>(rule: &R, source: &str) -> usize {
    let allocator = Allocator::with_capacity(source.len() * 4 + 1024);
    let lowered =
        vize_atelier_jsx::lower_source(&allocator, allocator.as_oxc(), source, JsxLang::Jsx);

    let mut total = 0;
    for lowered_root in &lowered.roots {
        let document = MarkupDocument::new(&lowered_root.root, TemplateSyntax::Vue);
        let mut lint = LintContext::new(&allocator, source, "test.jsx");
        let mut ctx = MarkupContext::new(&mut lint, &document);
        document.visit_with(rule, &mut ctx);
        total += lint.diagnostics().len();
    }
    total
}

fn run_over_jsx_oxc<R: MarkupRule>(rule: &R, source: &str) -> usize {
    let oxc_allocator = oxc_allocator::Allocator::default();
    let parsed = vize_atelier_jsx::parse_module(&oxc_allocator, source, JsxLang::Jsx);
    let document = MarkupDocument::from_jsx(&parsed.program, TemplateSyntax::Vue, 0);

    let lint_allocator = Allocator::with_capacity(source.len() * 4 + 1024);
    let mut lint = LintContext::new(&lint_allocator, source, "test.jsx");
    let mut ctx = MarkupContext::new(&mut lint, &document);
    document.visit_with(rule, &mut ctx);
    lint.diagnostics().len()
}

#[test]
fn no_dupe_style_properties_template() {
    let rule = NoDupeStyleProperties;
    for (source, expected, label) in [
        (
            r#"<div style="color: red; color: blue">x</div>"#,
            1,
            "one duplicate declaration",
        ),
        (
            r#"<div style="color: red; color: blue; color: green">x</div>"#,
            2,
            "triple duplicate reports each repeated declaration",
        ),
        (
            r#"<div style="color: red; color: blue; margin: 0; margin: 1px">x</div>"#,
            2,
            "two distinct repeated properties",
        ),
        (
            r#"<div style="margin: 0; MARGIN: 1px">x</div>"#,
            1,
            "property names are case-insensitive",
        ),
        (
            r#"<div style="  color :red ;  color : blue ">x</div>"#,
            1,
            "property names are trimmed",
        ),
        (
            r#"<div style="background: url(http://example.com/a); background: blue">x</div>"#,
            1,
            "colons inside values do not affect property parsing",
        ),
        (
            r#"<div style="color; color: blue">x</div>"#,
            0,
            "declarations without a colon are ignored",
        ),
        (
            r#"<div style="color: red" style="color: blue">x</div>"#,
            0,
            "duplicate properties across attributes are not combined",
        ),
        (
            r#"<div style="color: red; color: blue" style="margin: 0; margin: 1px">x</div>"#,
            2,
            "each static style attribute is checked independently",
        ),
        (r#"<div style>x</div>"#, 0, "valueless style is ignored"),
        (r#"<div style="">x</div>"#, 0, "empty style is clean"),
        (
            r#"<div :style="{ color: a, color: b }">x</div>"#,
            0,
            "dynamic style binding is ignored",
        ),
        (
            r#"<div v-bind:style="'color:red;color:blue'">x</div>"#,
            0,
            "long-form dynamic style binding is ignored",
        ),
        (
            r#"<div STYLE="color: red; color: blue">x</div>"#,
            0,
            "attribute names are case-sensitive",
        ),
        (
            r#"<MyWidget style="color: red; color: blue">x</MyWidget>"#,
            0,
            "components stay skipped",
        ),
        (
            r#"<slot style="color: red; color: blue"></slot>"#,
            1,
            "slot elements are not components and stay inspected",
        ),
        (
            r#"<template style="color: red; color: blue"><div /></template>"#,
            1,
            "template elements are not components and stay inspected",
        ),
    ] {
        assert_eq!(
            run_over_template(&rule, source),
            expected,
            "template case failed: {label}"
        );
    }
}

#[test]
fn no_dupe_style_properties_jsx_direct_matches_lowered_static_boundaries() {
    let rule = NoDupeStyleProperties;
    for (source, expected, label) in [
        (
            r#"const A = () => <div style="color: red; color: blue" />;"#,
            1,
            "one duplicate declaration",
        ),
        (
            r#"const A = () => <div style="color: red; color: blue; color: green" />;"#,
            2,
            "triple duplicate reports each repeated declaration",
        ),
        (
            r#"const A = () => <div style="margin: 0; MARGIN: 1px" />;"#,
            1,
            "property names are case-insensitive",
        ),
        (
            r#"const A = () => <div style="background: url(http://example.com/a); background: blue" />;"#,
            1,
            "colons inside values do not affect property parsing",
        ),
        (
            r#"const A = () => <div style="color: red" style="color: blue" />;"#,
            0,
            "duplicate properties across attributes are not combined",
        ),
        (
            r#"const A = () => <div style />;"#,
            0,
            "valueless style is ignored",
        ),
        (
            r#"const A = () => <div style={{ color: a, color: b }} />;"#,
            0,
            "expression-valued style is dynamic and ignored",
        ),
        (
            r#"const A = () => <div style={'color:red;color:blue'} />;"#,
            0,
            "string expression style is dynamic and ignored",
        ),
        (
            r#"const A = () => <div STYLE="color: red; color: blue" />;"#,
            0,
            "attribute names are case-sensitive",
        ),
        (
            r#"const A = () => <div html:style="color: red; color: blue" />;"#,
            0,
            "namespaced style attributes are ignored",
        ),
        (
            r#"const A = () => <Component style="color: red; color: blue" />;"#,
            0,
            "components stay skipped",
        ),
        (
            r#"const A = () => <Icons.Div style="color: red; color: blue" />;"#,
            0,
            "member components stay skipped",
        ),
        (
            r#"const A = () => <svg:div style="color: red; color: blue" />;"#,
            1,
            "lowercase namespaced intrinsic tags stay inspected",
        ),
    ] {
        let direct = run_over_jsx_oxc(&rule, source);
        assert_eq!(direct, expected, "JSX direct case failed: {label}");
        assert_eq!(
            direct,
            run_over_jsx_lowered(&rule, source),
            "JSX direct and lowered fallback diverged for {label}"
        );
    }
}
