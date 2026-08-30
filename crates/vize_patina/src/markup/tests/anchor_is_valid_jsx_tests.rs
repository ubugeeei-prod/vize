use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::a11y::AnchorIsValid;
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
fn anchor_is_valid_jsx_direct_matches_lowered() {
    let rule = AnchorIsValid;
    for (source, expected, label) in [
        (
            r#"const Link = () => <a href="/about">About</a>;"#,
            0,
            "static valid href",
        ),
        (
            r#"const Link = () => <a href={url}>Link</a>;"#,
            0,
            "dynamic href",
        ),
        (
            r##"const Link = () => <a href={'#'}>Link</a>;"##,
            0,
            "dynamic hash expression stays valid like fallback",
        ),
        (
            r#"const Link = () => <a href="">Link</a>;"#,
            1,
            "empty href",
        ),
        (
            r##"const Link = () => <a href="#">Link</a>;"##,
            1,
            "hash href",
        ),
        (
            r#"const Link = () => <a href="javascript:void(0)">Link</a>;"#,
            1,
            "javascript href",
        ),
        (
            r#"const Link = () => <a href>Link</a>;"#,
            1,
            "valueless href",
        ),
        (r#"const Link = () => <a>Link</a>;"#, 1, "missing href"),
        (
            r#"const Link = () => <a {...props}>Link</a>;"#,
            1,
            "spread props do not prove href",
        ),
        (
            r#"const Link = () => <a href="" href="/about">Link</a>;"#,
            1,
            "first static duplicate href wins",
        ),
        (
            r##"const Link = () => <a href={url} href="#">Link</a>;"##,
            0,
            "first dynamic duplicate href wins",
        ),
        (
            r#"const Link = () => <A href="/about">Link</A>;"#,
            0,
            "capitalized component is not an anchor tag",
        ),
        (
            r#"const Link = () => <Link.a href="/about">Link</Link.a>;"#,
            0,
            "member JSX tag is not an anchor tag",
        ),
        (
            r#"const Link = () => <svg:a href="/about">Link</svg:a>;"#,
            0,
            "namespaced JSX tag is not an anchor tag",
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
