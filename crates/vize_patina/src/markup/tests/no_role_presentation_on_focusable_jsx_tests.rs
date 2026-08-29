use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::a11y::NoRolePresentationOnFocusable;
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
fn no_role_presentation_on_focusable_jsx_direct_matches_lowered() {
    let rule = NoRolePresentationOnFocusable;
    for (source, expected, label) in [
        (
            r#"const A = () => <button role="presentation" />;"#,
            1,
            "native button presentation",
        ),
        (
            r#"const A = () => <input role="none" />;"#,
            1,
            "native input none",
        ),
        (
            r#"const A = () => <button role="button" />;"#,
            0,
            "non-presentation role",
        ),
        (
            r#"const A = () => <button role="Presentation" />;"#,
            0,
            "role value is exact",
        ),
        (
            r#"const A = () => <button role="presentation " />;"#,
            0,
            "role value is not trimmed",
        ),
        (
            r#"const A = () => <button role="presentation button" />;"#,
            0,
            "role value is not tokenized",
        ),
        (
            r#"const A = () => <button ROLE="presentation" />;"#,
            0,
            "case-sensitive role attr",
        ),
        (
            r#"const A = () => <button role={role} />;"#,
            0,
            "dynamic role",
        ),
        (
            r#"const A = () => <button role={"presentation"} />;"#,
            0,
            "dynamic string role",
        ),
        (r#"const A = () => <button role />;"#, 0, "valueless role"),
        (
            r#"const A = () => <button role role="presentation" />;"#,
            0,
            "first duplicate role wins",
        ),
        (
            r#"const A = () => <button role="presentation" role="button" />;"#,
            1,
            "first presentation duplicate controls",
        ),
        (
            r#"const A = () => <button role="button" role="presentation" />;"#,
            0,
            "later duplicate role does not override first value",
        ),
        (
            r#"const A = () => <a role="presentation">decorative</a>;"#,
            0,
            "anchor without href",
        ),
        (
            r#"const A = () => <a href="/" role="presentation">Home</a>;"#,
            1,
            "anchor with static href",
        ),
        (
            r#"const A = () => <a href={url} role="presentation">Home</a>;"#,
            1,
            "anchor with dynamic href value",
        ),
        (
            r#"const A = () => <a {...props} role="presentation">Home</a>;"#,
            0,
            "spread href is not inspected by the legacy helper",
        ),
        (
            r#"const A = () => <a ns:href={url} role="presentation">Home</a>;"#,
            0,
            "namespaced href stays ignored",
        ),
        (
            r#"const A = () => <div tabindex="0" role="presentation">Focusable</div>;"#,
            1,
            "lowercase tabindex",
        ),
        (
            r#"const A = () => <div tabindex="" role="presentation">Focusable</div>;"#,
            1,
            "empty tabindex",
        ),
        (
            r#"const A = () => <div tabindex="x" role="presentation">Focusable</div>;"#,
            1,
            "non-numeric tabindex",
        ),
        (
            r#"const A = () => <div tabIndex="0" role="presentation">Focusable</div>;"#,
            0,
            "camel-case tabIndex is outside the legacy exact helper",
        ),
        (
            r#"const A = () => <div contenteditable="true" role="presentation">Edit</div>;"#,
            1,
            "contenteditable true",
        ),
        (
            r#"const A = () => <div contenteditable="" role="presentation">Edit</div>;"#,
            1,
            "empty contenteditable",
        ),
        (
            r#"const A = () => <div contenteditable="plaintext-only" role="presentation">Edit</div>;"#,
            1,
            "plaintext-only contenteditable",
        ),
        (
            r#"const A = () => <div contenteditable="FALSE" role="presentation">Edit</div>;"#,
            1,
            "contenteditable value is exact",
        ),
        (
            r#"const A = () => <div contentEditable="true" role="presentation">Edit</div>;"#,
            0,
            "camel-case contentEditable is outside the legacy exact helper",
        ),
        (
            r#"const A = () => <Button role="presentation" />;"#,
            0,
            "components stay skipped",
        ),
        (
            r#"const A = () => <Forms.button role="presentation" />;"#,
            0,
            "member JSX tags stay outside unqualified native tags",
        ),
        (
            r#"const A = () => <ui:button role="presentation" />;"#,
            0,
            "namespaced JSX tags stay outside unqualified native tags",
        ),
        (
            r#"const A = () => <button a11y:role="presentation" />;"#,
            0,
            "namespaced role is outside exact unqualified attributes",
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
