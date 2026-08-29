use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::a11y::IframeHasTitle;
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

#[test]
fn iframe_has_title_template() {
    let rule = IframeHasTitle;
    for (source, expected, label) in [
        (
            r#"<iframe src="https://example.com" title="Example website"></iframe>"#,
            0,
            "static title",
        ),
        (
            r#"<iframe src="https://example.com" title="0"></iframe>"#,
            0,
            "nonempty numeric-looking title",
        ),
        (
            r#"<iframe src="https://example.com" :title="frameTitle"></iframe>"#,
            0,
            "bound shorthand title",
        ),
        (
            r#"<iframe src="https://example.com" v-bind:title="frameTitle"></iframe>"#,
            0,
            "bound longhand title",
        ),
        (
            r#"<iframe src="https://example.com" :title></iframe>"#,
            0,
            "valueless bound title matches legacy arg-only behavior",
        ),
        (
            r#"<iframe src="https://example.com" :title.trim="frameTitle"></iframe>"#,
            0,
            "bound title modifiers keep matching legacy arg-only behavior",
        ),
        (
            r#"<iframe src="https://example.com" v-bind:title=""></iframe>"#,
            0,
            "empty bound title expression is still a title binding",
        ),
        (
            r#"<iframe src="https://example.com" :[title]="frameTitle"></iframe>"#,
            0,
            "dynamic argument title matches legacy simple arg content",
        ),
        (
            r#"<iframe src="https://example.com" v-bind:[title]="frameTitle"></iframe>"#,
            0,
            "longhand dynamic argument title matches legacy simple arg content",
        ),
        (
            r#"<iframe src="https://example.com"></iframe>"#,
            1,
            "missing title",
        ),
        (
            r#"<iframe src="https://example.com" title></iframe>"#,
            1,
            "valueless title",
        ),
        (
            r#"<iframe src="https://example.com" title=""></iframe>"#,
            1,
            "empty title",
        ),
        (
            r#"<iframe src="https://example.com" title="   "></iframe>"#,
            1,
            "whitespace-only title",
        ),
        (
            r#"<iframe src="https://example.com" TITLE="Example"></iframe>"#,
            1,
            "title attribute name is exact",
        ),
        (
            r#"<iframe src="https://example.com" :Title="frameTitle"></iframe>"#,
            1,
            "bound title argument is exact",
        ),
        (
            r#"<iframe src="https://example.com" :[foo]="frameTitle"></iframe>"#,
            1,
            "dynamic argument with a different simple content is ignored",
        ),
        (
            r#"<iframe src="https://example.com" v-bind="frameAttrs"></iframe>"#,
            1,
            "object v-bind does not prove title",
        ),
        (
            r#"<iframe src="https://example.com" title="" :title="frameTitle"></iframe>"#,
            0,
            "later bound title can satisfy the rule",
        ),
        (
            r#"<iframe src="https://example.com" :title="frameTitle" title=""></iframe>"#,
            0,
            "earlier bound title can satisfy the rule",
        ),
        (
            r#"<iframe src="https://example.com" title title="Example"></iframe>"#,
            0,
            "later nonempty duplicate title can satisfy the rule",
        ),
        (
            r#"<iframe src="https://example.com" title="" title="Example"></iframe>"#,
            0,
            "later nonempty duplicate static title can satisfy the rule",
        ),
        (
            r#"<iframe src="https://example.com" @title="noop"></iframe>"#,
            1,
            "event named title is ignored",
        ),
        (
            r#"<Iframe src="https://example.com"></Iframe>"#,
            0,
            "capitalized component is not an iframe tag",
        ),
        (
            r#"<iframe-widget src="https://example.com"></iframe-widget>"#,
            0,
            "custom element is not exact iframe",
        ),
    ] {
        assert_eq!(
            run_over_template(&rule, source),
            expected,
            "template case failed: {label}"
        );
    }
}
