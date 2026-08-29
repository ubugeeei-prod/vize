use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::a11y::IframeHasTitle;
use vize_atelier_jsx::JsxLang;
use vize_s0::Allocator;

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
fn iframe_has_title_jsx_direct_matches_lowered() {
    let rule = IframeHasTitle;
    for (source, expected, label) in [
        (
            r#"const A = () => <iframe src="https://example.com" title="Example website" />;"#,
            0,
            "static title",
        ),
        (
            r#"const A = () => <iframe src="https://example.com" title="0" />;"#,
            0,
            "nonempty numeric-looking title",
        ),
        (
            r#"const A = () => <iframe src="https://example.com" title={frameTitle} />;"#,
            0,
            "dynamic title",
        ),
        (
            r#"const A = () => <iframe src="https://example.com" title={""} />;"#,
            0,
            "dynamic empty string title stays valid like the legacy fallback",
        ),
        (
            r#"const A = () => <iframe src="https://example.com" />;"#,
            1,
            "missing title",
        ),
        (
            r#"const A = () => <iframe src="https://example.com" title />;"#,
            1,
            "valueless title",
        ),
        (
            r#"const A = () => <iframe src="https://example.com" title="" />;"#,
            1,
            "empty title",
        ),
        (
            r#"const A = () => <iframe src="https://example.com" title="   " />;"#,
            1,
            "whitespace-only title",
        ),
        (
            r#"const A = () => <iframe src="https://example.com" TITLE="Example" />;"#,
            1,
            "title attribute name is exact",
        ),
        (
            r#"const A = () => <iframe src="https://example.com" {...frameAttrs} />;"#,
            1,
            "spread props do not prove title",
        ),
        (
            r#"const A = () => <iframe src="https://example.com" title="" title="Example" />;"#,
            0,
            "later nonempty duplicate static title can satisfy the rule",
        ),
        (
            r#"const A = () => <iframe src="https://example.com" title="" title={frameTitle} />;"#,
            0,
            "later dynamic duplicate title can satisfy the rule",
        ),
        (
            r#"const A = () => <iframe src="https://example.com" ns:title="Example" />;"#,
            1,
            "namespaced title is not an unqualified title attribute",
        ),
        (
            r#"const A = () => <Iframe src="https://example.com" />;"#,
            0,
            "capitalized component is not an iframe tag",
        ),
        (
            r#"const A = () => <Frame.iframe src="https://example.com" />;"#,
            0,
            "member JSX tag is not an iframe tag",
        ),
        (
            r#"const A = () => <svg:iframe src="https://example.com" />;"#,
            0,
            "namespaced tag is not an unqualified iframe tag",
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
