use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::a11y::InteractiveSupportsFocus;
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
fn interactive_supports_focus_jsx_direct_matches_lowered() {
    let rule = InteractiveSupportsFocus;
    for (source, expected, label) in [
        (
            r#"const A = () => <div role="button" onClick={handle}>Click</div>;"#,
            1,
            "div button role without focus",
        ),
        (
            r#"const A = () => <span role="link">Link</span>;"#,
            1,
            "span link role without focus",
        ),
        (
            r#"const A = () => <div role="button" tabindex="0">Click</div>;"#,
            0,
            "lowercase tabindex",
        ),
        (
            r#"const A = () => <div role="button" tabindex="">Click</div>;"#,
            0,
            "empty tabindex",
        ),
        (
            r#"const A = () => <div role="button" tabindex="x">Click</div>;"#,
            0,
            "non-numeric tabindex",
        ),
        (
            r#"const A = () => <div role="button" tabindex="-1">Click</div>;"#,
            1,
            "negative tabindex",
        ),
        (
            r#"const A = () => <div role="button" tabIndex="0">Click</div>;"#,
            1,
            "camel-case tabIndex is outside the legacy exact helper",
        ),
        (
            r#"const A = () => <div role="button" contenteditable="true">Click</div>;"#,
            0,
            "contenteditable true",
        ),
        (
            r#"const A = () => <div role="button" contenteditable="">Click</div>;"#,
            0,
            "empty contenteditable",
        ),
        (
            r#"const A = () => <div role="button" contenteditable="plaintext-only">Click</div>;"#,
            0,
            "plaintext-only contenteditable",
        ),
        (
            r#"const A = () => <div role="button" contenteditable="FALSE">Click</div>;"#,
            0,
            "contenteditable value is exact",
        ),
        (
            r#"const A = () => <div role="button" contentEditable="true">Click</div>;"#,
            1,
            "camel-case contentEditable is outside the legacy exact helper",
        ),
        (
            r#"const A = () => <area role="button" />;"#,
            1,
            "area without href",
        ),
        (
            r#"const A = () => <area href="/map" role="button" />;"#,
            0,
            "area with static href",
        ),
        (
            r#"const A = () => <area href={map} role="button" />;"#,
            0,
            "area with dynamic href value",
        ),
        (
            r#"const A = () => <area ns:href={map} role="button" />;"#,
            1,
            "namespaced href stays ignored",
        ),
        (
            r#"const A = () => <button role="link">Click</button>;"#,
            0,
            "native button",
        ),
        (
            r#"const A = () => <a role="button">Decorative link</a>;"#,
            0,
            "anchor is natively interactive even without href",
        ),
        (
            r#"const A = () => <details role="button" />;"#,
            0,
            "details is natively interactive in the legacy helper",
        ),
        (
            r#"const A = () => <audio role="button" />;"#,
            0,
            "audio is natively interactive in the legacy helper",
        ),
        (
            r#"const A = () => <video role="button" />;"#,
            0,
            "video is natively interactive in the legacy helper",
        ),
        (
            r#"const A = () => <div role="presentation">Content</div>;"#,
            0,
            "non-interactive role",
        ),
        (
            r#"const A = () => <div role="Button">Click</div>;"#,
            0,
            "role value is exact",
        ),
        (
            r#"const A = () => <div role="button ">Click</div>;"#,
            0,
            "role value is not trimmed",
        ),
        (
            r#"const A = () => <div role="button link">Click</div>;"#,
            0,
            "role value is not tokenized",
        ),
        (
            r#"const A = () => <div ROLE="button">Click</div>;"#,
            0,
            "case-sensitive role attr",
        ),
        (
            r#"const A = () => <div role={role}>Click</div>;"#,
            0,
            "dynamic role",
        ),
        (
            r#"const A = () => <div role={"button"}>Click</div>;"#,
            0,
            "dynamic string role",
        ),
        (
            r#"const A = () => <div role>Click</div>;"#,
            0,
            "valueless role",
        ),
        (
            r#"const A = () => <div role role="button">Click</div>;"#,
            0,
            "first valueless duplicate role masks later values",
        ),
        (
            r#"const A = () => <div role="button" role="presentation">Click</div>;"#,
            1,
            "first interactive duplicate controls",
        ),
        (
            r#"const A = () => <div role="presentation" role="button">Click</div>;"#,
            0,
            "later duplicate role does not override first value",
        ),
        (
            r#"const A = () => <Button role="button" />;"#,
            0,
            "components stay skipped",
        ),
        (
            r#"const A = () => <Forms.button role="button" />;"#,
            0,
            "member JSX tags stay outside unqualified native tags",
        ),
        (
            r#"const A = () => <ui:button role="button" />;"#,
            1,
            "namespaced JSX tags stay outside unqualified native tags",
        ),
        (
            r#"const A = () => <div a11y:role="button" />;"#,
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
