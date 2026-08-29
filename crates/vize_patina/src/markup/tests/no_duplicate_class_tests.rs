use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::opinionated::html::NoDuplicateClass;
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
fn no_duplicate_class_template() {
    let rule = NoDuplicateClass;
    for (source, expected, label) in [
        (
            r#"<div class="btn btn primary">x</div>"#,
            1,
            "one duplicate token",
        ),
        (
            r#"<div class="a a b b">x</div>"#,
            2,
            "two distinct duplicate tokens",
        ),
        (
            r#"<div class="x x x">x</div>"#,
            1,
            "triple duplicate reports once per token",
        ),
        (
            "<div class=\"a\ta\">x</div>",
            1,
            "ASCII whitespace tokenization is preserved",
        ),
        (
            r#"<div class="a" class="a">x</div>"#,
            0,
            "duplicate tokens across attributes are not combined",
        ),
        (
            r#"<div class="a a" class="b b">x</div>"#,
            2,
            "each static class attribute is checked independently",
        ),
        (r#"<div class>x</div>"#, 0, "valueless class is ignored"),
        (r#"<div class="">x</div>"#, 0, "empty class is clean"),
        (
            r#"<div :class="['btn', 'btn']">x</div>"#,
            0,
            "dynamic class binding is ignored",
        ),
        (
            r#"<div v-bind:class="'btn btn'">x</div>"#,
            0,
            "long-form dynamic class binding is ignored",
        ),
        (
            r#"<div CLASS="btn btn">x</div>"#,
            0,
            "attribute names are case-sensitive",
        ),
        (
            r#"<div class="a a" CLASS="b b">x</div>"#,
            1,
            "only exact lowercase class attributes participate",
        ),
        (
            r#"<div class="foo Foo">x</div>"#,
            0,
            "class tokens are case-sensitive",
        ),
        (
            r#"<div class="foo foo Foo Foo">x</div>"#,
            2,
            "case-distinct duplicate tokens report independently",
        ),
        (
            r#"<div v-bind="{ class: 'btn btn' }">x</div>"#,
            0,
            "object v-bind stays dynamic and ignored",
        ),
        (
            r#"<MyWidget class="btn btn">x</MyWidget>"#,
            1,
            "components are still inspected because the rule is attr-only",
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
fn no_duplicate_class_jsx_direct_matches_lowered_static_boundaries() {
    let rule = NoDuplicateClass;
    for (source, expected, label) in [
        (
            r#"const A = () => <div class="btn btn primary" />;"#,
            1,
            "one duplicate token",
        ),
        (
            r#"const A = () => <div class="a a b b" />;"#,
            2,
            "two distinct duplicate tokens",
        ),
        (
            r#"const A = () => <div class="x x x" />;"#,
            1,
            "triple duplicate reports once per token",
        ),
        (
            r#"const A = () => <div class="a" class="a" />;"#,
            0,
            "duplicate tokens across attributes are not combined",
        ),
        (
            r#"const A = () => <div class="a a" class="b b" />;"#,
            2,
            "each static class attribute is checked independently",
        ),
        (
            r#"const A = () => <div class />;"#,
            0,
            "valueless class is ignored",
        ),
        (
            r#"const A = () => <div className="btn btn" />;"#,
            0,
            "className is not the legacy class spelling",
        ),
        (
            r#"const A = () => <div CLASS="btn btn" />;"#,
            0,
            "attribute names are case-sensitive",
        ),
        (
            r#"const A = () => <div class={'btn btn'} />;"#,
            0,
            "expression-valued class is dynamic and ignored",
        ),
        (
            r#"const A = () => <div class={classes} />;"#,
            0,
            "dynamic class is ignored",
        ),
        (
            r#"const A = () => <div {...props} />;"#,
            0,
            "spread props are ignored",
        ),
        (
            r#"const A = () => <div html:class="btn btn" />;"#,
            0,
            "namespaced class attributes are ignored",
        ),
        (
            r#"const A = () => <Component class="btn btn" />;"#,
            1,
            "components are still inspected like the legacy visitor",
        ),
        (
            r#"const A = () => <Icons.Div class="btn btn" />;"#,
            1,
            "member components are still inspected because the rule is attr-only",
        ),
        (
            r#"const A = () => <svg:div class="btn btn" />;"#,
            1,
            "namespaced tags are still inspected because the rule is attr-only",
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
