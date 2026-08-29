use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::a11y::NoIForIcon;
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
fn no_i_for_icon_template() {
    let rule = NoIForIcon;
    assert_eq!(
        run_over_template(&rule, r#"<i class="fas fa-home"></i>"#),
        1,
        "template <i> with a static icon class must warn through the IR"
    );
    assert_eq!(
        run_over_template(&rule, r#"<i :class="'fas fa-home'"></i>"#),
        0,
        "template dynamic :class stays outside the legacy rule"
    );
    assert_eq!(
        run_over_template(&rule, r#"<i class="emphasis" class="fas fa-home"></i>"#),
        0,
        "only the first static class attribute participates"
    );
}

#[test]
fn no_i_for_icon_jsx_oxc() {
    let rule = NoIForIcon;
    assert_eq!(
        run_over_jsx_oxc(&rule, "const I = () => <i class=\"fas fa-home\" />;"),
        1,
        "JSX <i> with a static icon class must warn through the OXC IR path"
    );
    assert_eq!(
        run_over_jsx_oxc(&rule, "const I = () => <i className=\"fas fa-home\" />;"),
        0,
        "className is not the legacy Vue/JSX class spelling for this rule"
    );
    assert_eq!(
        run_over_jsx_oxc(&rule, "const I = () => <i class={iconClass} />;"),
        0,
        "dynamic class={{…}} stays outside the legacy rule"
    );
    assert_eq!(
        run_over_jsx_oxc(&rule, "const I = () => <Icons.i class=\"fas fa-home\" />;"),
        0,
        "JSX member components with local property `i` are not intrinsic <i>"
    );
    assert_eq!(
        run_over_jsx_oxc(&rule, "const I = () => <svg:i class=\"fas fa-home\" />;"),
        0,
        "JSX namespaced tags are not unqualified intrinsic <i> elements"
    );
    assert_eq!(
        run_over_jsx_oxc(&rule, "const I = () => <i icon:class=\"fas fa-home\" />;"),
        0,
        "JSX namespaced attributes are not unqualified static class attributes"
    );
}
