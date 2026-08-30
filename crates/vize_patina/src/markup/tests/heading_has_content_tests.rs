use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::a11y::HeadingHasContent;
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
fn heading_has_content_template_contract() {
    let rule = HeadingHasContent;
    for (source, expected, label) in [
        (r#"<h1>Hello World</h1>"#, 0, "text"),
        (r#"<h2>{{ title }}</h2>"#, 0, "interpolation"),
        (r#"<h1><span>Title</span></h1>"#, 0, "nested text"),
        (r#"<h1><slot></slot></h1>"#, 0, "default slot"),
        (
            r#"<h1 aria-hidden="true"></h1>"#,
            0,
            "static aria-hidden true",
        ),
        (
            r#"<h1 aria-label="Dashboard"></h1>"#,
            0,
            "static aria-label",
        ),
        (
            r#"<h1 aria-label=""></h1>"#,
            0,
            "empty static aria-label exists",
        ),
        (r#"<h1 :aria-label="title"></h1>"#, 0, "bound aria-label"),
        (
            r#"<h1 :aria-labelledby="labelId"></h1>"#,
            0,
            "bound aria-labelledby",
        ),
        (r#"<h1></h1>"#, 1, "empty heading"),
        (r#"<h1>   </h1>"#, 1, "whitespace text"),
        (r#"<h1 aria-hidden></h1>"#, 1, "valueless aria-hidden"),
        (
            r#"<h1 aria-hidden="True"></h1>"#,
            1,
            "case-sensitive hidden value",
        ),
        (
            r#"<h1 :aria-hidden="true"></h1>"#,
            1,
            "bound aria-hidden ignored",
        ),
        (
            r#"<h1 ARIA-LABEL="Title"></h1>"#,
            1,
            "aria label name is exact",
        ),
        (
            r#"<h1 a11y:aria-label="Title"></h1>"#,
            1,
            "namespaced aria-label ignored",
        ),
        (r#"<H1></H1>"#, 0, "component-like uppercase tag"),
        (r#"<svg:h1></svg:h1>"#, 0, "qualified tag"),
    ] {
        assert_eq!(
            run_over_template(&rule, source),
            expected,
            "template case changed for {label}"
        );
    }
}

#[test]
fn heading_has_content_jsx_direct_matches_lowered_boundaries() {
    let rule = HeadingHasContent;
    for (source, expected, label) in [
        (r#"const A = () => <h1>Hello World</h1>;"#, 0, "text"),
        (r#"const A = () => <h1>{title}</h1>;"#, 0, "interpolation"),
        (
            r#"const A = () => <h1>{0}</h1>;"#,
            0,
            "numeric interpolation",
        ),
        (
            r#"const A = () => <h1>{'Title'}</h1>;"#,
            0,
            "string literal text",
        ),
        (
            r#"const A = () => <h1><span>Title</span></h1>;"#,
            0,
            "nested text",
        ),
        (r#"const A = () => <h1><slot /></h1>;"#, 0, "slot outlet"),
        (
            r#"const A = () => <h1><><slot /></></h1>;"#,
            0,
            "fragment slot",
        ),
        (
            r#"const A = () => <h1 aria-hidden="true" />;"#,
            0,
            "static hidden",
        ),
        (
            r#"const A = () => <h1 aria-label="Dashboard" />;"#,
            0,
            "static aria-label",
        ),
        (
            r#"const A = () => <h1 aria-label="" />;"#,
            0,
            "empty aria-label",
        ),
        (
            r#"const A = () => <h1 aria-label={title} />;"#,
            0,
            "bound aria-label",
        ),
        (
            r#"const A = () => <h1 aria-labelledby={labelId} />;"#,
            0,
            "bound aria-labelledby",
        ),
        (
            r#"const A = () => <H1 />;"#,
            0,
            "component-like uppercase tag",
        ),
        (r#"const A = () => <Headings.h1 />;"#, 0, "member tag"),
        (r#"const A = () => <svg:h1 />;"#, 0, "qualified tag"),
        (
            r#"const A = () => <Comp render={<h1 />} />;"#,
            0,
            "attribute JSX root is outside legacy fallback boundary",
        ),
        (
            r#"const A = () => <Comp render={() => <h1 />} />;"#,
            0,
            "attribute callback JSX root is outside legacy fallback boundary",
        ),
        (r#"const A = () => <h1 />;"#, 1, "empty heading"),
        (r#"const A = () => <h1>{}</h1>;"#, 1, "empty expression"),
        (
            r#"const A = () => <h1>{''}</h1>;"#,
            1,
            "empty string literal",
        ),
        (
            r#"const A = () => <h1>{' '}</h1>;"#,
            1,
            "space string literal",
        ),
        (
            r#"const A = () => <h1>{ok && <span>Title</span>}</h1>;"#,
            1,
            "conditional child does not prove stable content",
        ),
        (
            r#"const A = () => <h1>{ok ? <span>Title</span> : null}</h1>;"#,
            1,
            "ternary child does not prove stable content",
        ),
        (
            r#"const A = () => <h1>{items.map((item) => <span>{item}</span>)}</h1>;"#,
            1,
            "mapped child does not prove stable content",
        ),
        (
            r#"const A = () => <h1 aria-hidden />;"#,
            1,
            "valueless aria-hidden",
        ),
        (
            r#"const A = () => <h1 aria-hidden="True" />;"#,
            1,
            "case-sensitive hidden value",
        ),
        (
            r#"const A = () => <h1 aria-hidden={true} />;"#,
            1,
            "bound aria-hidden ignored",
        ),
        (
            r#"const A = () => <h1 ARIA-LABEL="Title" />;"#,
            1,
            "aria label name is exact",
        ),
        (
            r#"const A = () => <h1 a11y:aria-label="Title" />;"#,
            1,
            "namespaced aria-label ignored",
        ),
        (
            r#"const A = () => <h1 {...attrs} />;"#,
            1,
            "spread attrs do not prove content",
        ),
    ] {
        let direct = run_over_jsx_oxc(&rule, source);
        assert_eq!(direct, expected, "JSX direct case changed for {label}");
        assert_eq!(
            direct,
            run_over_jsx_lowered(&rule, source),
            "JSX direct and lowered fallback diverged for {label}"
        );
    }
}
