use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::html::DeprecatedAttr;
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
fn deprecated_attr_template() {
    let rule = DeprecatedAttr;
    assert_eq!(
        run_over_template(&rule, r#"<div align="center">text</div>"#),
        1,
        "template static deprecated attrs must warn through markup IR"
    );
    assert_eq!(
        run_over_template(&rule, r#"<div align>text</div>"#),
        1,
        "valueless deprecated attrs remain static attrs"
    );
    assert_eq!(
        run_over_template(&rule, r##"<table bgcolor="#fff" cellpadding="5"></table>"##),
        2,
        "every deprecated table attr reports"
    );
    assert_eq!(
        run_over_template(&rule, r#"<table border="1"></table>"#),
        0,
        "table border keeps its legacy exception"
    );
    assert_eq!(
        run_over_template(&rule, r#"<div :align="side">text</div>"#),
        0,
        "bound attributes stay outside the legacy rule"
    );
    assert_eq!(
        run_over_template(&rule, r#"<div v-bind:align="side">text</div>"#),
        0,
        "long-form v-bind attributes stay outside the legacy rule"
    );
    assert_eq!(
        run_over_template(&rule, r#"<div ALIGN="center">text</div>"#),
        0,
        "attribute names remain exact and case-sensitive"
    );
    assert_eq!(
        run_over_template(&rule, r#"<MyTable align="center">text</MyTable>"#),
        0,
        "components stay skipped"
    );
}

#[test]
fn deprecated_attr_jsx_direct_matches_lowered_static_boundaries() {
    let rule = DeprecatedAttr;
    for (source, expected, label) in [
        (
            r#"const A = () => <div align="center" />;"#,
            1,
            "static global deprecated attr",
        ),
        (
            r#"const A = () => <div align />;"#,
            1,
            "valueless deprecated attr",
        ),
        (
            r#"const A = () => <table border="1" />;"#,
            0,
            "table border exception",
        ),
        (
            r#"const A = () => <table cellpadding="5" />;"#,
            1,
            "lowercase table cellpadding",
        ),
        (
            r#"const A = () => <table cellPadding="5" />;"#,
            0,
            "JSX camelCase attrs are not legacy lowercase HTML attrs",
        ),
        (
            r#"const A = () => <table cellpadding={5} />;"#,
            0,
            "dynamic JSX attrs stay outside the legacy rule",
        ),
        (
            r#"const A = () => <div ALIGN="center" />;"#,
            0,
            "case-sensitive attr name",
        ),
        (
            r#"const A = () => <Table align="center" />;"#,
            0,
            "component",
        ),
        (
            r#"const A = () => <svg:table border="1" />;"#,
            1,
            "namespaced table does not receive the unqualified table exception",
        ),
        (
            r#"const A = () => <div html:align="center" />;"#,
            0,
            "namespaced JSX attributes stay ignored",
        ),
    ] {
        assert_eq!(
            run_over_jsx_lowered(&rule, source),
            expected,
            "lowered JSX boundary changed for {label}"
        );
        assert_eq!(
            run_over_jsx_oxc(&rule, source),
            expected,
            "direct JSX IR must match the lowered boundary for {label}"
        );
    }
}
