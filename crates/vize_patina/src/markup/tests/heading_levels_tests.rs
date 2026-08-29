use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::a11y::HeadingLevels;
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
fn heading_levels_template_sequence_contract() {
    let rule = HeadingLevels;
    for (source, expected, label) in [
        (
            r#"<h1>Title</h1><h2>Section</h2><h3>Sub</h3>"#,
            0,
            "sequential headings",
        ),
        (
            r#"<h2>Section A</h2><h2>Section B</h2>"#,
            0,
            "same heading level",
        ),
        (
            r#"<h1>Title</h1><h2>Section</h2><h3>Sub</h3><h2>Back</h2>"#,
            0,
            "heading decrease",
        ),
        (r#"<h3>Only heading</h3>"#, 0, "single heading"),
        (r#"<div>content</div>"#, 0, "no headings"),
        (r#"<h1>Title</h1><h3>Subsection</h3>"#, 1, "h1 to h3 skip"),
        (r#"<h1>T</h1><h3>S</h3><h6>D</h6>"#, 2, "multiple skips"),
        (
            r#"<h1>Title</h1><section><h3>Subsection</h3></section>"#,
            1,
            "nested heading stays in document order",
        ),
        (
            r#"<h1>Title</h1><template><h3>Subsection</h3></template>"#,
            1,
            "template wrapper descendants participate",
        ),
        (
            r#"<template v-if="ok"><h1>Title</h1></template><template v-else><h3>Subsection</h3></template>"#,
            1,
            "v-if branches preserve source-order traversal",
        ),
        (
            r#"<template v-for="item in items"><h1>Title</h1><h3>Subsection</h3></template>"#,
            1,
            "v-for children preserve source-order traversal",
        ),
        (
            r#"<H1>Title</H1><H3>Subsection</H3>"#,
            0,
            "uppercase component-like tags are ignored",
        ),
        (
            r#"<h1>Title</h1><svg:h3>Subsection</svg:h3>"#,
            0,
            "qualified tag names are ignored",
        ),
    ] {
        assert_eq!(
            run_over_template(&rule, source),
            expected,
            "template case changed for {label}"
        );
    }
}

#[test]
fn heading_levels_jsx_single_root_projection_matches_lowered() {
    let rule = HeadingLevels;
    for (source, expected, label) in [
        (
            "const A = () => <><h1>Title</h1><h2>Section</h2><h3>Sub</h3></>;",
            0,
            "sequential fragment headings",
        ),
        (
            "const A = () => <><h1>Title</h1><h3>Subsection</h3></>;",
            1,
            "h1 to h3 skip",
        ),
        (
            "const A = () => <section><h1>Title</h1><div><h3>Sub</h3></div></section>;",
            1,
            "nested heading in one root",
        ),
        (
            "const A = () => <><h1>T</h1><h3>S</h3><h6>D</h6></>;",
            2,
            "multiple skips",
        ),
        (
            "const A = () => <><H1>Title</H1><H3>Subsection</H3></>;",
            0,
            "component-like tags ignored",
        ),
        (
            "const A = () => <><h1>Title</h1><svg:h3>Subsection</svg:h3></>;",
            0,
            "qualified tag ignored",
        ),
    ] {
        let direct = run_over_jsx_oxc(&rule, source);
        assert_eq!(
            direct, expected,
            "direct JSX projection changed for {label}"
        );
        assert_eq!(
            direct,
            run_over_jsx_lowered(&rule, source),
            "direct JSX projection diverged from lowered JSX for {label}"
        );
    }
}

#[test]
fn heading_levels_lowered_jsx_keeps_render_roots_isolated() {
    let rule = HeadingLevels;
    let separate_roots = "const A = () => <h1>Title</h1>;\nconst B = () => <h3>Sub</h3>;";
    assert_eq!(
        run_over_jsx_lowered(&rule, separate_roots),
        0,
        "legacy JSX fallback scoped heading state per lowered render root"
    );

    let each_root_has_skip = "const A = () => <><h1>Title</h1><h3>Sub</h3></>;\nconst B = () => <><h2>A</h2><h4>B</h4></>;";
    assert_eq!(
        run_over_jsx_lowered(&rule, each_root_has_skip),
        2,
        "each render root reports its own heading skip"
    );
}
