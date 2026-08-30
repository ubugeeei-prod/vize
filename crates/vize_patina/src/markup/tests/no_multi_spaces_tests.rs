use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::vue::NoMultiSpaces;
use vize_atelier_jsx::JsxLang;
use vize_s0::Allocator;

fn run_over_template<R: MarkupRule>(rule: &R, source: &str) -> Vec<(u32, u32)> {
    let allocator = Allocator::with_capacity(source.len() * 4 + 1024);
    let parser = vize_armature::Parser::new(&allocator, source);
    let (root, _errors) = parser.parse();
    let document = MarkupDocument::new(&root, TemplateSyntax::Vue);

    let mut lint = LintContext::new(&allocator, source, "test.vue");
    let mut ctx = MarkupContext::new(&mut lint, &document);
    document.visit_with(rule, &mut ctx);
    lint.diagnostics()
        .iter()
        .map(|diagnostic| (diagnostic.start, diagnostic.end))
        .collect()
}

fn run_over_jsx_lowered<R: MarkupRule>(rule: &R, source: &str) -> Vec<(u32, u32)> {
    let allocator = Allocator::with_capacity(source.len() * 4 + 1024);
    let lowered =
        vize_atelier_jsx::lower_source(&allocator, allocator.as_oxc(), source, JsxLang::Jsx);

    let mut ranges = Vec::new();
    for lowered_root in &lowered.roots {
        let document = MarkupDocument::new(&lowered_root.root, TemplateSyntax::Vue);
        let mut lint = LintContext::new(&allocator, source, "test.jsx");
        let mut ctx = MarkupContext::new(&mut lint, &document);
        document.visit_with(rule, &mut ctx);
        ranges.extend(
            lint.diagnostics()
                .iter()
                .map(|diagnostic| (diagnostic.start, diagnostic.end)),
        );
    }
    ranges
}

fn run_over_jsx_oxc<R: MarkupRule>(rule: &R, source: &str) -> Vec<(u32, u32)> {
    let oxc_allocator = oxc_allocator::Allocator::default();
    let parsed = vize_atelier_jsx::parse_module(&oxc_allocator, source, JsxLang::Jsx);
    let document = MarkupDocument::from_jsx(&parsed.program, TemplateSyntax::Vue, 0);

    let lint_allocator = Allocator::with_capacity(source.len() * 4 + 1024);
    let mut lint = LintContext::new(&lint_allocator, source, "test.jsx");
    let mut ctx = MarkupContext::new(&mut lint, &document);
    document.visit_with(rule, &mut ctx);
    lint.diagnostics()
        .iter()
        .map(|diagnostic| (diagnostic.start, diagnostic.end))
        .collect()
}

fn slices<'a>(source: &'a str, ranges: &[(u32, u32)]) -> Vec<&'a str> {
    ranges
        .iter()
        .map(|(start, end)| &source[*start as usize..*end as usize])
        .collect()
}

#[test]
fn no_multi_spaces_template_boundaries() {
    let rule = NoMultiSpaces::default();
    for (source, expected, label) in [
        (
            r#"<div class="foo" id="bar"></div>"#,
            Vec::<&str>::new(),
            "single spaces are clean",
        ),
        (
            r#"<div  class="foo"></div>"#,
            vec!["  "],
            "multiple spaces before first attr",
        ),
        (
            r#"<div class="foo"  id="bar"></div>"#,
            vec!["  "],
            "multiple spaces between attrs",
        ),
        (
            "<div class=\"foo\"\t\tid=\"bar\"></div>",
            vec!["\t\t"],
            "multiple tabs between attrs",
        ),
        (
            r#"<button
  class="btn"
  :disabled="isDisabled"
></button>"#,
            Vec::<&str>::new(),
            "multiline attributes stay clean",
        ),
        (
            r#"<div :class="{  active: isActive }"  id="panel"></div>"#,
            vec!["  "],
            "directive expression internals are ignored but attr gap is checked",
        ),
    ] {
        let ranges = run_over_template(&rule, source);
        assert_eq!(
            slices(source, &ranges),
            expected,
            "template boundary changed for {label}"
        );
    }
}

#[test]
fn no_multi_spaces_jsx_direct_matches_lowered_boundaries() {
    let rule = NoMultiSpaces::default();
    for (source, expected, label) in [
        (
            r#"const A = () => <div className="foo" id="bar" />;"#,
            Vec::<&str>::new(),
            "single spaces are clean",
        ),
        (
            r#"const A = () => <div  className="foo" />;"#,
            vec!["  "],
            "multiple spaces before first prop",
        ),
        (
            r#"const A = () => <div className="foo"  id="bar" />;"#,
            vec!["  "],
            "multiple spaces between props",
        ),
        (
            "const A = () => <div className=\"foo\"\t\tid=\"bar\" />;",
            vec!["\t\t"],
            "multiple tabs between props",
        ),
        (
            r#"const A = () => <Button.Icon  aria-label="Save" />;"#,
            vec!["  "],
            "member component tag gap maps to authored source",
        ),
        (
            r#"const A = () => <svg:path  stroke="currentColor" />;"#,
            vec!["  "],
            "namespaced JSX tag gap maps to authored source",
        ),
        (
            r#"const A = () => <div {...props}  id="bar" />;"#,
            vec!["  "],
            "spread props participate in legacy opening item windows",
        ),
        (
            r#"const A = () => <button
  className="btn"
  disabled={isDisabled}
/>;"#,
            Vec::<&str>::new(),
            "multiline JSX props stay clean",
        ),
    ] {
        let direct = run_over_jsx_oxc(&rule, source);
        assert_eq!(
            slices(source, &direct),
            expected,
            "JSX direct boundary failed for {label}"
        );
        assert_eq!(
            direct,
            run_over_jsx_lowered(&rule, source),
            "JSX direct and lowered fallback diverged for {label}"
        );
    }
}
