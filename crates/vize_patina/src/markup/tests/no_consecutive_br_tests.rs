use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::html::NoConsecutiveBr;
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
fn no_consecutive_br_template_child_sequence() {
    let rule = NoConsecutiveBr;
    for (source, expected, label) in [
        (r#"<p>line<br>line</p>"#, 0, "single br"),
        (r#"<p>line<br><br>more</p>"#, 1, "adjacent br"),
        (r#"<p>line<br> <br>more</p>"#, 1, "whitespace text"),
        (r#"<p>line<br><!-- spacer --><br>more</p>"#, 1, "comment"),
        (
            r#"<p>line<br>{{ spacer }}<br>more</p>"#,
            0,
            "interpolation resets",
        ),
        (
            r#"<p>line<br><span></span><br>more</p>"#,
            0,
            "element resets",
        ),
        (
            r#"<p>line<br><template><br></template><br>more</p>"#,
            0,
            "template element resets",
        ),
        (
            r#"<section><template><br><br></template></section>"#,
            1,
            "inner template element scans its own children",
        ),
        (r#"<p>line<br><br><br>more</p>"#, 2, "three brs"),
        (r#"<p>line<BR><BR>more</p>"#, 0, "case-sensitive tag"),
    ] {
        assert_eq!(
            run_over_template(&rule, source),
            expected,
            "template boundary changed for {label}"
        );
    }
}

#[test]
fn no_consecutive_br_template_parent_boundaries() {
    let rule = NoConsecutiveBr;
    for (source, expected, label) in [
        (r#"<br><br>"#, 0, "root siblings have no legacy parent"),
        (
            r#"<MyText><br><br></MyText>"#,
            0,
            "component parent is skipped",
        ),
        (
            r#"<MyText><p><br><br></p></MyText>"#,
            1,
            "native descendant inside component still reports",
        ),
        (
            r#"<p><br><br></p><p><br><br></p>"#,
            2,
            "separate parents do not share state",
        ),
    ] {
        assert_eq!(
            run_over_template(&rule, source),
            expected,
            "template parent boundary changed for {label}"
        );
    }
}

#[test]
fn no_consecutive_br_jsx_lowered_child_sequence() {
    let rule = NoConsecutiveBr;
    for (source, expected, label) in [
        ("const A = () => <p>line<br />line</p>;", 0, "single br"),
        (
            "const A = () => <p>line<br /><br />more</p>;",
            1,
            "adjacent br",
        ),
        (
            "const A = () => <p>line<br />{' '}<br />more</p>;",
            1,
            "explicit space text",
        ),
        (
            "const A = () => <p>line<br />{'x'}<br />more</p>;",
            0,
            "significant string text resets",
        ),
        (
            "const A = () => <p>line<br />{spacer}<br />more</p>;",
            0,
            "expression interpolation resets",
        ),
        (
            "const A = () => <p>line<br />{/* spacer */}<br />more</p>;",
            1,
            "JSX comments stay transparent through lowering",
        ),
        (
            "const A = () => <p><><br /><br /></></p>;",
            1,
            "nested fragment children are spliced into parent",
        ),
        (
            "const A = () => <p><br />{cond && <br />}<br /></p>;",
            0,
            "control-flow child resets the parent sequence",
        ),
        (
            "const A = () => <p>{cond && <><br /><br /></>}</p>;",
            1,
            "fragment inside control-flow reports inside its branch",
        ),
        (
            "const A = () => <p>{items.map((item) => <><br /><br /></>)}</p>;",
            1,
            "fragment inside list reports inside its branch",
        ),
        (
            "const A = () => <p>line<BR /><BR />more</p>;",
            0,
            "uppercase JSX intrinsic is not legacy lowercase br",
        ),
    ] {
        assert_eq!(
            run_over_jsx_lowered(&rule, source),
            expected,
            "lowered JSX boundary changed for {label}"
        );
    }
}

#[test]
fn no_consecutive_br_jsx_lowered_parent_boundaries() {
    let rule = NoConsecutiveBr;
    for (source, expected, label) in [
        (
            "const A = () => <><br /><br /></>;",
            0,
            "top-level fragment children have no legacy parent",
        ),
        (
            "const A = () => <Box><br /><br /></Box>;",
            0,
            "component parent is skipped",
        ),
        (
            "const A = () => <Box><p><br /><br /></p></Box>;",
            1,
            "native descendant inside component still reports",
        ),
        (
            "const A = () => <p><br /><br /></p>;\nconst B = () => <p><br /><br /></p>;",
            2,
            "separate render roots do not share state",
        ),
    ] {
        assert_eq!(
            run_over_jsx_lowered(&rule, source),
            expected,
            "lowered JSX parent boundary changed for {label}"
        );
    }
}
