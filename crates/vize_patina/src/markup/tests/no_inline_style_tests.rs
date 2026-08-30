use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::vue::NoInlineStyle;
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
fn no_inline_style_template_boundaries() {
    let rule = NoInlineStyle;
    for (source, expected, label) in [
        (
            r#"<div class="foo"></div>"#,
            Vec::<&str>::new(),
            "class attribute stays clean",
        ),
        (
            r#"<div :class="{ active: isActive }"></div>"#,
            Vec::<&str>::new(),
            "dynamic class binding stays clean",
        ),
        (
            r#"<div style="color:red"></div>"#,
            vec![r#"style="color:red""#],
            "static style warns",
        ),
        (
            r#"<div style></div>"#,
            vec!["style"],
            "valueless style warns",
        ),
        (
            r#"<div style=""></div>"#,
            vec![r#"style="""#],
            "empty static style warns",
        ),
        (
            r#"<div :style="{ color: activeColor }"></div>"#,
            vec![r#":style="{ color: activeColor }""#],
            "shorthand dynamic style warns",
        ),
        (
            r#"<div :style.prop="styles"></div>"#,
            vec![r#":style.prop="styles""#],
            "modifier dynamic style warns",
        ),
        (
            r#"<div v-bind:style="styles"></div>"#,
            vec![r#"v-bind:style="styles""#],
            "long-form dynamic style warns",
        ),
        (
            r#"<div v-bind:[style]="styles"></div>"#,
            vec![r#"v-bind:[style]="styles""#],
            "legacy dynamic style arg warns",
        ),
        (
            r#"<MyComponent style="color:red" :style="styles" />"#,
            vec![r#"style="color:red""#, r#":style="styles""#],
            "component style props keep legacy behavior",
        ),
        (
            r#"<div STYLE="color:red"></div>"#,
            Vec::<&str>::new(),
            "attribute name matching remains case-sensitive",
        ),
        (
            r#"<div :Style="styles" v-style="styles" v-bind="attrs" foo:style="x"></div>"#,
            Vec::<&str>::new(),
            "non-matching directive and namespaced spellings stay clean",
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
fn no_inline_style_jsx_direct_matches_lowered_boundaries() {
    let rule = NoInlineStyle;
    for (source, expected, label) in [
        (
            r#"const A = () => <div className="foo" />;"#,
            Vec::<&str>::new(),
            "className stays clean",
        ),
        (
            r#"const A = () => <div {...props} />;"#,
            Vec::<&str>::new(),
            "spread props stay clean",
        ),
        (
            r#"const A = () => <div STYLE="color:red" foo:style="x" />;"#,
            Vec::<&str>::new(),
            "uppercase and namespaced style attributes stay clean",
        ),
        (
            r#"const A = () => <div v-bind:Style={styles} />;"#,
            Vec::<&str>::new(),
            "v-bind style argument remains case-sensitive",
        ),
        (
            r#"const A = () => <div onStyle={handler} />;"#,
            Vec::<&str>::new(),
            "event-shaped prop does not become a style binding",
        ),
        (
            r#"const A = () => <div style="color:red" />;"#,
            vec![r#"style="color:red""#],
            "static style warns",
        ),
        (
            r#"const A = () => <div style />;"#,
            vec!["style"],
            "valueless style warns",
        ),
        (
            r#"const A = () => <div style="" />;"#,
            vec![r#"style="""#],
            "empty static style warns",
        ),
        (
            r#"const A = () => <div style={{ color: activeColor }} />;"#,
            vec![r#"style={{ color: activeColor }}"#],
            "expression style warns",
        ),
        (
            r#"const A = () => <div style={dynamicStyles} />;"#,
            vec![r#"style={dynamicStyles}"#],
            "identifier expression style warns",
        ),
        (
            r#"const A = () => <div v-bind:style={styles} />;"#,
            vec![r#"v-bind:style={styles}"#],
            "JSX directive spelling warns",
        ),
        (
            r#"const A = () => <Widget style="color:red" />;"#,
            vec![r#"style="color:red""#],
            "component style props keep legacy behavior",
        ),
        (
            r#"const A = () => <Design.Box style="color:red" />;"#,
            vec![r#"style="color:red""#],
            "member component style props keep legacy behavior",
        ),
        (
            r#"const A = () => <div>{cond && <span style={{ color }} />}</div>;"#,
            vec![r#"style={{ color }}"#],
            "nested expression-container JSX style warns",
        ),
        (
            r#"const A = () => <><div style="color:red" /><span style={{ marginTop }} /></>;"#,
            vec![r#"style="color:red""#, r#"style={{ marginTop }}"#],
            "multiple JSX styles report in source order",
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
