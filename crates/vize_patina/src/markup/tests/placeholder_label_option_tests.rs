use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::a11y::PlaceholderLabelOption;
use vize_atelier_jsx::JsxLang;
use vize_l0::Allocator;

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
            r#"<select required><option value="a">A</option><option value="b">B</option></select>"#,
            1,
            "a required select must start with an empty option",
        ),
        (
            r#"<select><option value="a">A</option></select>"#,
            0,
            "a select that is not required has no placeholder",
        ),
        (
            r#"<select><option value="">Any</option><option value="a">A</option></select>"#,
            0,
            "an empty option in a select that is not required is an ordinary choice",
        ),
        (
            r#"<select required><option value="">Choose</option><option value="a">A</option></select>"#,
            0,
            "an empty first option is the placeholder",
        ),
        (
            r#"<select required><option value>Choose</option></select>"#,
            0,
            "a valueless value attribute is empty",
        ),
        (
            r#"<select required><option>Choose</option></select>"#,
            1,
            "without value, the text is the value",
        ),
        (
            r#"<select required><option :value="choice">Choose</option></select>"#,
            0,
            "a dynamic value is not judged",
        ),
        (
            r#"<select required multiple><option value="a">A</option></select>"#,
            0,
            "a multiple select has no placeholder",
        ),
        (
            r#"<select required size="4"><option value="a">A</option></select>"#,
            0,
            "a select with a display size above 1 has no placeholder",
        ),
        (
            r#"<select required><option VALUE="">Choose</option></select>"#,
            1,
            "value attribute name remains exact",
        ),
        (
            r#"<SELECT required><option value="a">A</option></SELECT>"#,
            0,
            "select tag name remains exact",
        ),
        (
            r#"<select required><OPTION value="a">A</OPTION></select>"#,
            0,
            "an unknown child leaves the options undecided",
        ),
        (
            r#"<select required><span></span><option value="a">A</option></select>"#,
            0,
            "other elements leave the options undecided",
        ),
        (
            r#"<select required><optgroup label="G"><option value="">Choose</option></optgroup></select>"#,
            1,
            "an option inside optgroup is not a placeholder",
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
            r#"const A = () => <select required><option value="a">A</option></select>;"#,
            1,
            "a required select must start with an empty option",
        ),
        (
            r#"const A = () => <select><option value="">Any</option></select>;"#,
            0,
            "a select that is not required has no placeholder",
        ),
        (
            r#"const A = () => <select required><option value="">Choose</option></select>;"#,
            0,
            "an empty first option is the placeholder",
        ),
        (
            r#"const A = () => <select required><option value>Choose</option></select>;"#,
            0,
            "a valueless value attribute is empty",
        ),
        (
            r#"const A = () => <select required><option value={""}>Choose</option></select>;"#,
            0,
            "a literal empty value is empty",
        ),
        (
            r#"const A = () => <select required><option value={choice}>Choose</option></select>;"#,
            0,
            "a dynamic value is not judged",
        ),
        (
            r#"const A = () => <select required><option>Choose</option></select>;"#,
            1,
            "without value, the text is the value",
        ),
        (
            r#"const A = () => <select required={strict}><option value="a">A</option></select>;"#,
            0,
            "a bound required is not judged",
        ),
        (
            r#"const A = () => <Select required><option value="a">A</option></Select>;"#,
            0,
            "capitalized select is a component",
        ),
        (
            r#"const A = () => <Forms.select required><option value="a">A</option></Forms.select>;"#,
            0,
            "member select is not an unqualified select tag",
        ),
        (
            r#"const A = () => <svg:select required><option value="a">A</option></svg:select>;"#,
            0,
            "namespaced select is not an unqualified select tag",
        ),
        (
            r#"const A = () => <select required><Option value="a">A</Option></select>;"#,
            0,
            "a component child leaves the options undecided",
        ),
        (
            r#"const A = () => <select required><><option value="a">A</option></></select>;"#,
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
