use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::a11y::NoRedundantRoles;
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
fn no_redundant_roles_template() {
    let rule = NoRedundantRoles;
    assert_eq!(
        run_over_template(&rule, r#"<nav role="navigation">Navigation</nav>"#),
        1,
        "template nav/navigation must warn through markup IR"
    );
    assert_eq!(
        run_over_template(&rule, r#"<div role="navigation">Navigation</div>"#),
        0,
        "non-implicit roles stay clean"
    );
    assert_eq!(
        run_over_template(&rule, r#"<a href="/" role="link">Home</a>"#),
        1,
        "static href gives anchors their implicit link role"
    );
    assert_eq!(
        run_over_template(&rule, r#"<a :href="url" role="link">Home</a>"#),
        0,
        "bound href stays outside the legacy implicit-role probe"
    );
    assert_eq!(
        run_over_template(&rule, r#"<img alt="" role="presentation" />"#),
        1,
        "empty static alt maps img to presentation"
    );
    assert_eq!(
        run_over_template(&rule, r#"<input type="checkbox" role="checkbox" />"#),
        1,
        "static input type controls the implicit input role"
    );
    assert_eq!(
        run_over_template(&rule, r#"<nav :role="'navigation'">Navigation</nav>"#),
        0,
        "bound role values remain ignored"
    );
    assert_eq!(
        run_over_template(&rule, r#"<button ROLE="button">Click</button>"#),
        0,
        "role attribute names remain exact and case-sensitive"
    );
    assert_eq!(
        run_over_template(&rule, r#"<button role="Button">Click</button>"#),
        0,
        "role values remain exact and case-sensitive"
    );
    assert_eq!(
        run_over_template(&rule, r#"<button role="button ">Click</button>"#),
        0,
        "role values are not trimmed"
    );
    assert_eq!(
        run_over_template(&rule, r#"<button role="button checkbox">Click</button>"#),
        0,
        "role values are not tokenized"
    );
    assert_eq!(
        run_over_template(&rule, r#"<MyNav role="navigation">Navigation</MyNav>"#),
        0,
        "components stay skipped"
    );
    assert_eq!(
        run_over_template(
            &rule,
            r#"<nav role class="x" role="navigation">Navigation</nav>"#
        ),
        0,
        "a valueless first role attribute masks later duplicates like the legacy helper"
    );
    assert_eq!(
        run_over_template(
            &rule,
            r#"<button role="button" role="switch">Click</button>"#
        ),
        1,
        "the first static role attribute controls duplicate-role behavior"
    );
    assert_eq!(
        run_over_template(
            &rule,
            r#"<button role="switch" role="button">Click</button>"#
        ),
        0,
        "later duplicate role attributes do not override the first one"
    );
}

#[test]
fn no_redundant_roles_jsx_direct_matches_lowered_boundaries() {
    let rule = NoRedundantRoles;
    for (source, expected, label) in [
        (
            r#"const A = () => <nav role="navigation" />;"#,
            1,
            "nav/navigation",
        ),
        (
            r#"const A = () => <div role="navigation" />;"#,
            0,
            "div/navigation",
        ),
        (
            r#"const A = () => <a href="/" role="link" />;"#,
            1,
            "static href anchor",
        ),
        (
            r#"const A = () => <a href={url} role="link" />;"#,
            0,
            "dynamic href anchor",
        ),
        (
            r#"const A = () => <img alt="" role="presentation" />;"#,
            1,
            "empty alt img",
        ),
        (
            r#"const A = () => <input type="checkbox" role="checkbox" />;"#,
            1,
            "typed input",
        ),
        (r#"const A = () => <nav role={role} />;"#, 0, "dynamic role"),
        (r#"const A = () => <button role />;"#, 0, "valueless role"),
        (
            r#"const A = () => <button Role="button" />;"#,
            0,
            "case-sensitive role attribute",
        ),
        (
            r#"const A = () => <button role="Button" />;"#,
            0,
            "case-sensitive role value",
        ),
        (
            r#"const A = () => <button role="button " />;"#,
            0,
            "untrimmed role value",
        ),
        (
            r#"const A = () => <button role="button checkbox" />;"#,
            0,
            "untokenized role value",
        ),
        (
            r#"const A = () => <button role="button" role="switch" />;"#,
            1,
            "first duplicate role wins",
        ),
        (
            r#"const A = () => <button role="switch" role="button" />;"#,
            0,
            "later duplicate roles stay ignored",
        ),
        (
            r#"const A = () => <svg:button role="button" />;"#,
            0,
            "namespaced JSX tag",
        ),
        (
            r#"const A = () => <Forms.button role="button" />;"#,
            0,
            "member JSX component",
        ),
        (
            r#"const A = () => <MyNav role="navigation" />;"#,
            0,
            "component",
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
