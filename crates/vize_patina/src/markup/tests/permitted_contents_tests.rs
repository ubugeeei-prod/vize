use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::vue::PermittedContents;
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
fn permitted_contents_template_markup_ancestor_boundaries() {
    let rule = PermittedContents;
    for (source, expected, label) in [
        (
            r#"<span><div>block</div></span>"#,
            1,
            "block element inside phrasing ancestor",
        ),
        (
            r##"<main><a href="#"><div>block</div></a></main>"##,
            0,
            "transparent anchor inherits a flow parent",
        ),
        (
            r#"<span><a><div>block</div></a></span>"#,
            1,
            "transparent anchor does not hide phrasing ancestor",
        ),
        (
            r##"<a href="#"><button>click</button></a>"##,
            1,
            "interactive element inside interactive ancestor",
        ),
        (
            r#"<ul><div>not li</div></ul>"#,
            1,
            "constrained list parent rejects non-li child",
        ),
        (r#"<ul><li>item</li></ul>"#, 0, "li is a valid list child"),
        (
            r#"<ul><MyItem /></ul>"#,
            0,
            "unknown component child stays exempt",
        ),
        (
            r#"<ul><motion.div>item</motion.div></ul>"#,
            1,
            "configured intrinsic member component keeps its native content model",
        ),
    ] {
        assert_eq!(
            run_over_template(&rule, source),
            expected,
            "template markup boundary changed for {label}"
        );
    }
}

#[test]
fn permitted_contents_jsx_lowered_matches_legacy_fallback_boundaries() {
    let rule = PermittedContents;
    for (source, expected, label) in [
        (
            r#"const A = () => <span><div>block</div></span>;"#,
            1,
            "block element inside phrasing ancestor",
        ),
        (
            r##"const A = () => <main><a href="#"><div>block</div></a></main>;"##,
            0,
            "transparent anchor inherits a flow parent",
        ),
        (
            r#"const A = () => <span><a><div>block</div></a></span>;"#,
            1,
            "transparent anchor does not hide phrasing ancestor",
        ),
        (
            r##"const A = () => <a href="#"><button>click</button></a>;"##,
            1,
            "interactive element inside interactive ancestor",
        ),
        (
            r#"const A = () => <ul><div>not li</div></ul>;"#,
            1,
            "constrained list parent rejects non-li child",
        ),
        (
            r#"const A = () => <ul><li>item</li></ul>;"#,
            0,
            "li is a valid list child",
        ),
        (
            r#"const A = () => <ul><MyItem /></ul>;"#,
            0,
            "unknown component child stays exempt",
        ),
        (
            r#"const A = () => <table><MyRow /></table>;"#,
            0,
            "unknown component can render a table row",
        ),
        (
            r#"const A = () => <select><div>bad</div></select>;"#,
            1,
            "constrained select parent rejects non-option child",
        ),
        (
            r#"const A = () => <ul><motion.div>item</motion.div></ul>;"#,
            0,
            "member JSX tags lower to dynamic components and stay exempt",
        ),
    ] {
        assert_eq!(
            run_over_jsx_lowered(&rule, source),
            expected,
            "lowered JSX boundary changed for {label}"
        );
    }
}
