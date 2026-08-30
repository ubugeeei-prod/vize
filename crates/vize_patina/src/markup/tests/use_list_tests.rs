use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::a11y::UseList;
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
fn use_list_template_markup_boundaries() {
    let rule = UseList;
    for (source, expected, label) in [
        (r#"<p>Normal text</p>"#, 0, "normal text is clean"),
        (
            r#"<p>-word</p>"#,
            0,
            "dash without following space is clean",
        ),
        (r#"<p>   </p>"#, 0, "whitespace-only text is clean"),
        (
            r#"<ul><li>- Item</li></ul>"#,
            0,
            "list item context is clean",
        ),
        (
            r#"<ol><li>* Item</li></ol>"#,
            0,
            "ordered list context is clean",
        ),
        (r#"<li>+ Item</li>"#, 0, "direct li context is clean"),
        (
            r#"<pre>- markdown content</pre>"#,
            0,
            "pre context is clean",
        ),
        (r#"<code>- flag</code>"#, 0, "code context is clean"),
        (r#"<script>- script</script>"#, 0, "script context is clean"),
        (r#"<style>- css</style>"#, 0, "style context is clean"),
        (
            r#"<p><span></span>- Item</p>"#,
            0,
            "first non-text child stops scanning",
        ),
        (
            r#"<p><!-- comment -->- Item</p>"#,
            0,
            "comment before text stops scanning",
        ),
        (
            r#"<p>{{ lead }} - Item</p>"#,
            0,
            "interpolation before text stops scanning",
        ),
        (r#"<MyPanel>- Item</MyPanel>"#, 0, "components are clean"),
        (r#"<p>- Item one</p>"#, 1, "dash bullet warns"),
        (r#"<span>* Item one</span>"#, 1, "asterisk bullet warns"),
        (r#"<div>+ Item one</div>"#, 1, "plus bullet warns"),
        (
            r#"<p>   - Item one</p>"#,
            1,
            "leading whitespace is ignored",
        ),
        (r#"<section>- Item</section>"#, 1, "native section warns"),
        (
            r#"<my-panel>- Item</my-panel>"#,
            1,
            "lowercase custom element warns",
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
fn use_list_list_context_ancestors_suppress_nested_bullets() {
    let rule = UseList;
    for (source, expected, label) in [
        (
            r#"<ul><li><span>- Item</span></li></ul>"#,
            0,
            "ul/li ancestor suppresses nested span",
        ),
        (
            r#"<ol><li><p>* Item</p></li></ol>"#,
            0,
            "ol/li ancestor suppresses nested paragraph",
        ),
        (
            r#"<pre><span>- literal</span></pre>"#,
            0,
            "pre ancestor suppresses nested literal text",
        ),
        (
            r#"<code><span>- flag</span></code>"#,
            0,
            "code ancestor suppresses nested literal text",
        ),
        (
            r#"<script><span>- text</span></script>"#,
            0,
            "script ancestor suppresses nested text",
        ),
        (
            r#"<style><span>- text</span></style>"#,
            0,
            "style ancestor suppresses nested text",
        ),
        (
            r#"<div><p>- Item</p></div>"#,
            1,
            "ordinary ancestor does not suppress bullet text",
        ),
    ] {
        assert_eq!(
            run_over_template(&rule, source),
            expected,
            "template ancestor boundary changed for {label}"
        );
    }
}

#[test]
fn use_list_jsx_direct_matches_lowered_for_element_boundaries() {
    let rule = UseList;
    for (source, expected, label) in [
        (
            r#"const A = () => <p>Normal text</p>;"#,
            0,
            "normal text is clean",
        ),
        (
            r#"const A = () => <p>-word</p>;"#,
            0,
            "dash without following space is clean",
        ),
        (
            r#"const A = () => <ul><li>- Item</li></ul>;"#,
            0,
            "list context is clean",
        ),
        (
            r#"const A = () => <pre>- markdown content</pre>;"#,
            0,
            "pre context is clean",
        ),
        (
            r#"const A = () => <p>{lead} - Item</p>;"#,
            0,
            "expression before text stops scanning",
        ),
        (
            r#"const A = () => <p><span />- Item</p>;"#,
            0,
            "element before text stops scanning",
        ),
        (
            r#"const A = () => <Panel>- Item</Panel>;"#,
            0,
            "capitalized component is clean",
        ),
        (
            r#"const A = () => <Icons.Panel>- Item</Icons.Panel>;"#,
            0,
            "member component is clean",
        ),
        (r#"const A = () => <p>- Item</p>;"#, 1, "dash bullet warns"),
        (
            r#"const A = () => <span>* Item</span>;"#,
            1,
            "asterisk bullet warns",
        ),
        (
            r#"const A = () => <div>+ Item</div>;"#,
            1,
            "plus bullet warns",
        ),
        (
            r#"const A = () => <p><span>- Item</span></p>;"#,
            1,
            "nested child element text still warns",
        ),
        (
            r#"const A = () => <panel>- Item</panel>;"#,
            1,
            "lowercase JSX intrinsic-style custom element warns",
        ),
        (
            r#"const A = () => <my-panel>- Item</my-panel>;"#,
            1,
            "hyphenated lowercase JSX custom element warns",
        ),
        (
            r#"const A = () => <List.ul><p>- Item</p></List.ul>;"#,
            1,
            "member property named ul is not list context",
        ),
    ] {
        assert_eq!(
            run_over_jsx_oxc(&rule, source),
            expected,
            "direct JSX markup boundary changed for {label}"
        );
        assert_eq!(
            run_over_jsx_lowered(&rule, source),
            expected,
            "lowered JSX markup boundary changed for {label}"
        );
    }
}

#[test]
fn use_list_unicode_bullet_prefixes_require_space() {
    let rule = UseList;
    for bullet in [
        "\u{2022}", "\u{2023}", "\u{25E6}", "\u{2043}", "\u{2219}", "\u{25AA}", "\u{25CF}",
    ] {
        let invalid = format!("<p>{bullet} Item</p>");
        assert_eq!(
            run_over_template(&rule, &invalid),
            1,
            "unicode bullet with a following space must warn for {bullet:?}"
        );

        let valid = format!("<p>{bullet}Item</p>");
        assert_eq!(
            run_over_template(&rule, &valid),
            0,
            "unicode bullet without a following space must stay clean for {bullet:?}"
        );
    }
}
