use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::a11y::NoAriaHiddenOnFocusable;
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
fn no_aria_hidden_on_focusable_jsx_direct_matches_lowered() {
    let rule = NoAriaHiddenOnFocusable;
    for (source, expected, label) in [
        (
            r#"const A = () => <button aria-hidden="true" />;"#,
            1,
            "native button",
        ),
        (
            r#"const A = () => <button aria-hidden="false" />;"#,
            0,
            "aria-hidden false",
        ),
        (
            r#"const A = () => <button aria-hidden={true} />;"#,
            0,
            "dynamic aria-hidden",
        ),
        (
            r#"const A = () => <button aria-hidden={"true"} />;"#,
            0,
            "dynamic string aria-hidden",
        ),
        (
            r#"const A = () => <button aria-hidden />;"#,
            0,
            "valueless aria-hidden",
        ),
        (
            r#"const A = () => <button aria-hidden aria-hidden="true" />;"#,
            0,
            "first duplicate aria-hidden wins",
        ),
        (
            r#"const A = () => <button ARIA-HIDDEN="true" />;"#,
            0,
            "case-sensitive aria-hidden attr",
        ),
        (
            r#"const A = () => <a aria-hidden="true">decorative</a>;"#,
            0,
            "anchor without href",
        ),
        (
            r#"const A = () => <a href="/" aria-hidden="true">Home</a>;"#,
            1,
            "anchor with static href",
        ),
        (
            r#"const A = () => <a href={url} aria-hidden="true">Home</a>;"#,
            1,
            "anchor with dynamic href value",
        ),
        (
            r#"const A = () => <a {...props} aria-hidden="true">Home</a>;"#,
            0,
            "spread href is not inspected by the legacy helper",
        ),
        (
            r#"const A = () => <a ns:href={url} aria-hidden="true">Home</a>;"#,
            0,
            "namespaced href stays ignored",
        ),
        (
            r#"const A = () => <div tabindex="0" aria-hidden="true">Focusable</div>;"#,
            1,
            "lowercase tabindex",
        ),
        (
            r#"const A = () => <div tabindex="" aria-hidden="true">Focusable</div>;"#,
            1,
            "empty tabindex",
        ),
        (
            r#"const A = () => <div tabIndex="0" aria-hidden="true">Focusable</div>;"#,
            0,
            "camel-case tabIndex is outside the legacy exact helper",
        ),
        (
            r#"const A = () => <div contenteditable="true" aria-hidden="true">Edit</div>;"#,
            1,
            "contenteditable true",
        ),
        (
            r#"const A = () => <div contenteditable="" aria-hidden="true">Edit</div>;"#,
            1,
            "empty contenteditable",
        ),
        (
            r#"const A = () => <div contentEditable="true" aria-hidden="true">Edit</div>;"#,
            0,
            "camel-case contentEditable is outside the legacy exact helper",
        ),
        (
            r#"const A = () => <Button aria-hidden="true" />;"#,
            0,
            "components stay skipped",
        ),
        (
            r#"const A = () => <Forms.button aria-hidden="true" />;"#,
            0,
            "member JSX tags stay outside unqualified native tags",
        ),
        (
            r#"const A = () => <ui:button aria-hidden="true" />;"#,
            0,
            "namespaced JSX tags stay outside unqualified native tags",
        ),
        (
            r#"const A = () => <button ariaHidden="true" />;"#,
            0,
            "camel-case ariaHidden is outside the legacy exact helper",
        ),
        (
            r#"const A = () => <button a11y:aria-hidden="true" />;"#,
            0,
            "namespaced aria-hidden is outside exact unqualified attributes",
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
