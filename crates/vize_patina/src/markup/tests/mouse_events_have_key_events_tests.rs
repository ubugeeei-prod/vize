use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::a11y::MouseEventsHaveKeyEvents;
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

#[test]
fn mouse_events_template() {
    let rule = MouseEventsHaveKeyEvents;
    assert_eq!(
        run_over_template(&rule, r#"<div @mouseenter="show"></div>"#),
        1,
        "template mouseenter without focus must warn through markup IR"
    );
    assert_eq!(
        run_over_template(&rule, r#"<div @mouseover="show" @focus="show"></div>"#),
        0,
        "template mouseover paired with focus is clean"
    );
    assert_eq!(
        run_over_template(
            &rule,
            r#"<div @mouseenter="show" @mouseleave="hide"></div>"#
        ),
        2,
        "missing focus and blur produce the two legacy diagnostics"
    );
    assert_eq!(
        run_over_template(&rule, r#"<div @mouseenter="show" @mouseover="show"></div>"#),
        1,
        "enter-group mouse events report once per element"
    );
    assert_eq!(
        run_over_template(&rule, r#"<div @mouseleave="hide" @mouseout="hide"></div>"#),
        1,
        "leave-group mouse events report once per element"
    );
    assert_eq!(
        run_over_template(&rule, r#"<div @[mouseenter]="show"></div>"#),
        0,
        "dynamic mouse event arguments stay outside the legacy rule"
    );
    assert_eq!(
        run_over_template(&rule, r#"<div @mouseenter="show" @[focus]="show"></div>"#),
        1,
        "dynamic focus arguments do not satisfy the static companion event"
    );
    assert_eq!(
        run_over_template(&rule, r#"<div @mouseEnter="show" @focus="show"></div>"#),
        0,
        "event arguments remain exact and case-sensitive"
    );
    assert_eq!(
        run_over_template(&rule, r#"<div @mouseenter="show" @Focus="show"></div>"#),
        1,
        "companion event arguments remain exact and case-sensitive"
    );
}

#[test]
fn mouse_events_jsx_lowered() {
    let rule = MouseEventsHaveKeyEvents;
    assert_eq!(
        run_over_jsx_lowered(&rule, "const A = () => <div onMouseEnter={show} />;"),
        0,
        "standard JSX onMouseEnter stays clean, matching the old fallback"
    );
    assert_eq!(
        run_over_jsx_lowered(&rule, "const A = () => <div onMouseEnterCapture={show} />;"),
        0,
        "standard JSX event suffix casing stays clean"
    );
    assert_eq!(
        run_over_jsx_lowered(&rule, "const A = () => <div onMouseenterCapture={show} />;"),
        1,
        "legacy JSX lowering maps this casing to mouseenter"
    );
    assert_eq!(
        run_over_jsx_lowered(&rule, "const A = () => <div v-on:mouseenter={show} />;"),
        1,
        "JSX Vue directive spelling keeps the legacy fallback warning"
    );
    assert_eq!(
        run_over_jsx_lowered(
            &rule,
            "const A = () => <div v-on:mouseenter={show} onFocus={show} />;"
        ),
        1,
        "standard JSX onFocus does not satisfy a legacy v-on:focus companion"
    );
    assert_eq!(
        run_over_jsx_lowered(
            &rule,
            "const A = () => <div v-on:mouseenter={show} v-on:focus={show} />;"
        ),
        0,
        "JSX Vue directive spelling can provide the focus companion"
    );
    assert_eq!(
        run_over_jsx_lowered(
            &rule,
            "const A = () => <div onMouseenterCapture={show} onFocusCapture={show} />;"
        ),
        0,
        "legacy JSX lowering also maps onFocusCapture to the companion focus event"
    );
    assert_eq!(
        run_over_jsx_lowered(
            &rule,
            "const A = () => <Tooltip onMouseenterCapture={show} />;"
        ),
        0,
        "JSX components stay skipped"
    );
}
