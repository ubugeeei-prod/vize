use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::vue::NoBareStringsInTemplate;
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
fn no_bare_strings_template_markup_boundaries() {
    let rule = NoBareStringsInTemplate;
    for (source, expected, label) in [
        (r#"<div>hello</div>"#, 1, "bare latin text warns"),
        (r#"<div>こんにちは</div>"#, 1, "bare non-ascii text warns"),
        (
            r#"<div><span>Save</span></div>"#,
            1,
            "nested element owns text",
        ),
        (
            r#"<button title="Close">x</button>"#,
            2,
            "target attr and direct text both warn",
        ),
        (r#"<img alt="a cat" />"#, 1, "target static attr warns"),
        (
            r#"<input placeholder="Search" />"#,
            1,
            "placeholder attr warns",
        ),
        (r#"<div aria-label="Menu"></div>"#, 1, "aria attr warns"),
        (
            r#"<button TITLE="Close"></button>"#,
            1,
            "target attrs keep case-insensitive matching",
        ),
        (
            r#"<div>{{ $t('hello') }}</div>"#,
            0,
            "interpolation is dynamic",
        ),
        (
            r#"<img :alt="$t('cat')" />"#,
            0,
            "bound target attr is dynamic",
        ),
        ("<div>   </div>", 0, "whitespace-only text is clean"),
        (r#"<div>-</div>"#, 0, "punctuation-only text is clean"),
        (r#"<div>123</div>"#, 0, "number-only text is clean"),
        (
            r#"<div class="container"></div>"#,
            0,
            "non-target attr is clean",
        ),
        (
            r#"<script>const label = "Hello";</script>"#,
            0,
            "script text is clean",
        ),
        (
            r#"<style>.button::before { content: "Hello"; }</style>"#,
            0,
            "style text is clean",
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
fn no_bare_strings_jsx_direct_matches_lowered_for_static_boundaries() {
    let rule = NoBareStringsInTemplate;
    for (source, expected, label) in [
        (
            r#"const A = () => <div>hello</div>;"#,
            1,
            "bare latin text warns",
        ),
        (
            r#"const A = () => <div>こんにちは</div>;"#,
            1,
            "bare non-ascii text warns",
        ),
        (
            r#"const A = () => <button title="Close">x</button>;"#,
            2,
            "target attr and direct text both warn",
        ),
        (
            r#"const A = () => <img alt="a cat" />;"#,
            1,
            "target static attr warns",
        ),
        (
            r#"const A = () => <img alt={caption} />;"#,
            0,
            "dynamic target attr is clean",
        ),
        (
            r#"const A = () => <img html:alt="a cat" />;"#,
            0,
            "namespaced target attr is not unqualified",
        ),
        (
            r#"const A = () => <button TITLE="Close" />;"#,
            1,
            "case-insensitive target attr warns",
        ),
        (
            r#"const A = () => <div>{label}</div>;"#,
            0,
            "expression-only child is clean",
        ),
        (
            r#"const A = () => <script>const label = "Hello";</script>;"#,
            0,
            "script text is clean",
        ),
        (
            r#"const A = () => <style>{`.x { color: red; }`}</style>;"#,
            0,
            "style expression is clean",
        ),
    ] {
        let direct = run_over_jsx_oxc(&rule, source);
        assert_eq!(
            direct, expected,
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
fn no_bare_strings_jsx_lowered_preserves_string_expression_children() {
    let rule = NoBareStringsInTemplate;
    for (source, expected, label) in [
        (
            r#"const A = () => <div>{'Hello'}</div>;"#,
            1,
            "single-quoted string expression lowers to text",
        ),
        (
            r#"const A = () => <div>{"Hello"}</div>;"#,
            1,
            "double-quoted string expression lowers to text",
        ),
        (
            r#"const A = () => <div>{`Hello`}</div>;"#,
            0,
            "template string expression stays dynamic",
        ),
        (
            r#"const A = () => <div>{/* comment */}Hello</div>;"#,
            1,
            "comments do not hide following text",
        ),
        (
            r#"const A = () => <div><>{'Hello'}</></div>;"#,
            1,
            "fragment string expression still warns after lowering",
        ),
        (
            r#"const A = () => <div>{cond && <span>Hello</span>}</div>;"#,
            1,
            "nested conditional element text still warns",
        ),
    ] {
        assert_eq!(
            run_over_jsx_lowered(&rule, source),
            expected,
            "lowered JSX string boundary changed for {label}"
        );
    }
}
