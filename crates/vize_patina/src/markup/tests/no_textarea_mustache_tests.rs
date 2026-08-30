use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::vue::NoTextareaMustache;
use vize_atelier_jsx::JsxLang;
use vize_s0::Allocator;

fn run_over_template<R: MarkupRule>(rule: &R, source: &str) -> Vec<(u32, u32)> {
    let allocator = Allocator::with_capacity(source.len() * 4 + 1024);
    let parser = vize_armature::Parser::new(&allocator, source);
    let (root, errors) = parser.parse();
    assert!(
        errors.is_empty(),
        "template fixture must parse without errors: {errors:?}"
    );
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
    assert!(
        lowered.diagnostics.is_empty(),
        "JSX fixture must lower without diagnostics: {:?}",
        lowered.diagnostics
    );

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

fn slices<'a>(source: &'a str, ranges: &[(u32, u32)]) -> Vec<&'a str> {
    ranges
        .iter()
        .map(|(start, end)| &source[*start as usize..*end as usize])
        .collect()
}

#[test]
fn no_textarea_mustache_template_boundaries() {
    let rule = NoTextareaMustache;
    for (source, expected, label) in [
        (
            r#"<textarea v-model="message"></textarea>"#,
            Vec::<&str>::new(),
            "v-model is the valid replacement",
        ),
        (
            r#"<div>{{ message }}</div>"#,
            Vec::<&str>::new(),
            "mustache outside textarea stays clean",
        ),
        (
            r#"<Textarea>{{ message }}</Textarea>"#,
            Vec::<&str>::new(),
            "component spelling stays clean",
        ),
        (
            r#"<TEXTAREA>{{ message }}</TEXTAREA>"#,
            Vec::<&str>::new(),
            "legacy tag check is lowercase exact",
        ),
        (
            r#"<textarea><span>{{ message }}</span></textarea>"#,
            vec![r#"{{ message }}"#],
            "textarea rawtext keeps legacy interpolation behavior",
        ),
        (
            r#"<textarea><!-- comment --></textarea>"#,
            Vec::<&str>::new(),
            "comments stay clean",
        ),
        (
            r#"<textarea>{{ message }}</textarea>"#,
            vec![r#"{{ message }}"#],
            "direct mustache warns",
        ),
        (
            r#"<textarea v-model="model">{{ message }}</textarea>"#,
            vec![r#"{{ message }}"#],
            "v-model does not suppress a direct mustache",
        ),
        (
            r#"<textarea>{{ first }}{{ second }}</textarea>"#,
            vec![r#"{{ first }}"#, r#"{{ second }}"#],
            "each direct mustache reports in source order",
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
fn no_textarea_mustache_jsx_lowered_boundaries() {
    let rule = NoTextareaMustache;
    for (source, expected, label) in [
        (
            r#"const A = () => <textarea defaultValue={message} />;"#,
            Vec::<&str>::new(),
            "defaultValue prop is clean",
        ),
        (
            r#"const A = () => <div>{message}</div>;"#,
            Vec::<&str>::new(),
            "expression outside textarea stays clean",
        ),
        (
            r#"const A = () => <Textarea>{message}</Textarea>;"#,
            Vec::<&str>::new(),
            "component spelling stays clean",
        ),
        (
            r#"const A = () => <Forms.textarea>{message}</Forms.textarea>;"#,
            Vec::<&str>::new(),
            "member component spelling stays clean",
        ),
        (
            r#"const A = () => <svg:textarea>{message}</svg:textarea>;"#,
            Vec::<&str>::new(),
            "qualified textarea tag stays clean",
        ),
        (
            r#"const A = () => <textarea><span>{message}</span></textarea>;"#,
            Vec::<&str>::new(),
            "nested child expression is not a direct textarea child",
        ),
        (
            r#"const A = () => <textarea>{/* comment */}</textarea>;"#,
            Vec::<&str>::new(),
            "empty expression comments stay clean",
        ),
        (
            r#"const A = () => <textarea>{message}</textarea>;"#,
            vec![r#"{message}"#],
            "direct expression warns",
        ),
        (
            r#"const A = () => <textarea>{a + b}</textarea>;"#,
            vec![r#"{a + b}"#],
            "complex direct expression warns",
        ),
        (
            r#"const A = () => <textarea>{null}</textarea>;"#,
            vec![r#"{null}"#],
            "null direct expression stays an interpolation",
        ),
        (
            r#"const A = () => <textarea>{0}</textarea>;"#,
            vec![r#"{0}"#],
            "numeric direct expression stays an interpolation",
        ),
        (
            r#"const A = () => <textarea>{"message"}</textarea>;"#,
            Vec::<&str>::new(),
            "literal string expression lowers to static text",
        ),
        (
            r#"const A = () => <textarea>{condition ? <span /> : <em />}</textarea>;"#,
            Vec::<&str>::new(),
            "lowered ternary element arms are not mustache interpolation",
        ),
        (
            r#"const A = () => <textarea>{cond && <span />}</textarea>;"#,
            Vec::<&str>::new(),
            "lowered control-flow element child is not a mustache interpolation",
        ),
        (
            r#"const A = () => <textarea>{items.map(() => <span />)}</textarea>;"#,
            Vec::<&str>::new(),
            "lowered list element child is not a mustache interpolation",
        ),
        (
            r#"const A = () => <Comp fallback={<textarea>{message}</textarea>} />;"#,
            Vec::<&str>::new(),
            "JSX prop values stay outside the lowered child surface",
        ),
        (
            r#"const A = () => <textarea>{`message`}</textarea>;"#,
            vec![r#"{`message`}"#],
            "template literal expression remains an interpolation",
        ),
        (
            r#"const A = () => <textarea>{first}{second}</textarea>;"#,
            vec![r#"{first}"#, r#"{second}"#],
            "multiple direct expressions report in source order",
        ),
        (
            r#"const A = () => <textarea><>{message}</></textarea>;"#,
            vec![r#"{message}"#],
            "lowered fragments are transparent under textarea",
        ),
    ] {
        assert_eq!(
            slices(source, &run_over_jsx_lowered(&rule, source)),
            expected,
            "lowered JSX boundary changed for {label}"
        );
    }
}
