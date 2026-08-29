use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::a11y::PlaceholderLabelOption;
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
fn placeholder_label_option_template_boundaries() {
    let rule = PlaceholderLabelOption;
    for (source, expected, label) in [
        (
            r#"<select><option value="">Choose</option><option value="a">A</option></select>"#,
            1,
            "first empty-valued option requires disabled or hidden",
        ),
        (
            r#"<select><option value>Choose</option><option value="a">A</option></select>"#,
            1,
            "valueless value attribute is a placeholder",
        ),
        (
            r#"<select><option value="" disabled>Choose</option></select>"#,
            0,
            "disabled placeholder is valid",
        ),
        (
            r#"<select><option value="" hidden>Choose</option></select>"#,
            0,
            "hidden placeholder is valid",
        ),
        (
            r#"<select><option>Choose</option><option value="a">A</option></select>"#,
            0,
            "missing value attribute stays outside the legacy placeholder check",
        ),
        (
            r#"<select><option :value="choice">Choose</option></select>"#,
            0,
            "dynamic value does not prove a static placeholder",
        ),
        (
            r#"<select><option value="" :disabled="disabled">Choose</option></select>"#,
            1,
            "bound disabled does not satisfy the static disabled check",
        ),
        (
            r#"<select><option VALUE="">Choose</option></select>"#,
            0,
            "value attribute name remains exact",
        ),
        (
            r#"<select><option value="" DISABLED>Choose</option></select>"#,
            1,
            "disabled attribute name remains exact",
        ),
        (
            r#"<SELECT><option value="">Choose</option></SELECT>"#,
            0,
            "select tag name remains exact",
        ),
        (
            r#"<select><OPTION value="">Choose</OPTION></select>"#,
            0,
            "option tag name remains exact",
        ),
        (
            r#"<select><span></span><option value="">Choose</option></select>"#,
            1,
            "first direct option is found after non-option children",
        ),
        (
            r#"<select><span><option value="">Nested</option></span></select>"#,
            0,
            "nested option is not a direct child",
        ),
    ] {
        assert_eq!(
            run_over_template(&rule, source),
            expected,
            "template boundary changed for {label}"
        );
    }
}

#[test]
fn placeholder_label_option_jsx_direct_matches_lowered() {
    let rule = PlaceholderLabelOption;
    for (source, expected, label) in [
        (
            r#"const A = () => <select><option value="">Choose</option><option value="a">A</option></select>;"#,
            1,
            "first empty-valued option requires disabled or hidden",
        ),
        (
            r#"const A = () => <select><option value>Choose</option></select>;"#,
            1,
            "valueless value attribute is a placeholder",
        ),
        (
            r#"const A = () => <select><option value="" disabled>Choose</option></select>;"#,
            0,
            "disabled placeholder is valid",
        ),
        (
            r#"const A = () => <select><option value="" hidden>Choose</option></select>;"#,
            0,
            "hidden placeholder is valid",
        ),
        (
            r#"const A = () => <select><option value="" disabled={disabled}>Choose</option></select>;"#,
            1,
            "bound disabled does not satisfy the static disabled check",
        ),
        (
            r#"const A = () => <select><option value={""}>Choose</option></select>;"#,
            0,
            "dynamic value expression does not prove a static placeholder",
        ),
        (
            r#"const A = () => <select><option>Choose</option></select>;"#,
            0,
            "missing value attribute stays outside the legacy placeholder check",
        ),
        (
            r#"const A = () => <Select><option value="">Choose</option></Select>;"#,
            0,
            "capitalized select is a component",
        ),
        (
            r#"const A = () => <Forms.select><option value="">Choose</option></Forms.select>;"#,
            0,
            "member select is not an unqualified select tag",
        ),
        (
            r#"const A = () => <svg:select><option value="">Choose</option></svg:select>;"#,
            0,
            "namespaced select is not an unqualified select tag",
        ),
        (
            r#"const A = () => <select><Option value="">Choose</Option></select>;"#,
            0,
            "capitalized option is a component",
        ),
        (
            r#"const A = () => <select><svg:option value="">Choose</svg:option></select>;"#,
            0,
            "namespaced option is not an unqualified option tag",
        ),
        (
            r#"const A = () => <select><><option value="">Choose</option></></select>;"#,
            1,
            "JSX fragments under select are transparent like the lowering fallback",
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
