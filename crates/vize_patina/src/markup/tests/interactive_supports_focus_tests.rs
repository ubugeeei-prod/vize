use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::a11y::InteractiveSupportsFocus;
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
fn interactive_supports_focus_template() {
    let rule = InteractiveSupportsFocus;
    for (source, expected, label) in [
        (
            r#"<div role="button" @click="handle">Click</div>"#,
            1,
            "div button role without focus",
        ),
        (
            r#"<span role="link">Link</span>"#,
            1,
            "span link role without focus",
        ),
        (
            r#"<div role="button" tabindex="0" @click="handle">Click</div>"#,
            0,
            "tabindex zero",
        ),
        (
            r#"<div role="button" tabindex="">Click</div>"#,
            0,
            "empty tabindex remains focusable",
        ),
        (
            r#"<div role="button" tabindex="x">Click</div>"#,
            0,
            "non-numeric tabindex remains focusable",
        ),
        (
            r#"<div role="button" tabindex="-1">Click</div>"#,
            1,
            "negative tabindex is not focusable",
        ),
        (
            r#"<div role="button" tabindex tabindex="0">Click</div>"#,
            1,
            "first valueless duplicate tabindex masks later values",
        ),
        (
            r#"<div role="button" :tabindex="0">Click</div>"#,
            1,
            "bound tabindex stays outside the legacy static-value helper",
        ),
        (
            r#"<div role="button" contenteditable="true">Click</div>"#,
            0,
            "contenteditable true",
        ),
        (
            r#"<div role="button" contenteditable="">Click</div>"#,
            0,
            "empty contenteditable remains focusable",
        ),
        (
            r#"<div role="button" contenteditable="plaintext-only">Click</div>"#,
            0,
            "plaintext-only contenteditable remains focusable",
        ),
        (
            r#"<div role="button" contenteditable="FALSE">Click</div>"#,
            0,
            "contenteditable value is exact",
        ),
        (
            r#"<div role="button" contenteditable="false">Click</div>"#,
            1,
            "contenteditable false",
        ),
        (
            r#"<div role="button" :contenteditable="true">Click</div>"#,
            1,
            "bound contenteditable stays outside the legacy static-value helper",
        ),
        (
            r#"<div role="button" contenteditable="false" contenteditable="true">Click</div>"#,
            1,
            "later duplicate contenteditable does not override first value",
        ),
        (
            r#"<area role="button" />"#,
            1,
            "area is focusable only with href",
        ),
        (
            r#"<area href="/map" role="button" />"#,
            0,
            "area with static href",
        ),
        (
            r#"<area :href="map" role="button" />"#,
            0,
            "area with static-arg bound href",
        ),
        (
            r#"<area :[href]="map" role="button" />"#,
            1,
            "area with dynamic href argument",
        ),
        (r#"<button role="link">Click</button>"#, 0, "native button"),
        (
            r#"<a role="button">Decorative link</a>"#,
            0,
            "anchor is natively interactive even without href",
        ),
        (
            r#"<details role="button"></details>"#,
            0,
            "details is natively interactive in the legacy helper",
        ),
        (
            r#"<audio role="button"></audio>"#,
            0,
            "audio is natively interactive in the legacy helper",
        ),
        (
            r#"<video role="button"></video>"#,
            0,
            "video is natively interactive in the legacy helper",
        ),
        (
            r#"<div role="presentation">Content</div>"#,
            0,
            "non-interactive role",
        ),
        (
            r#"<div role="Button">Click</div>"#,
            0,
            "role value is exact",
        ),
        (
            r#"<div role="button ">Click</div>"#,
            0,
            "role value is not trimmed",
        ),
        (
            r#"<div role="button link">Click</div>"#,
            0,
            "role value is not tokenized",
        ),
        (
            r#"<div ROLE="button">Click</div>"#,
            0,
            "role attribute name is exact",
        ),
        (
            r#"<div :role="'button'">Click</div>"#,
            0,
            "bound role stays dynamic",
        ),
        (r#"<div role>Click</div>"#, 0, "valueless role stays clean"),
        (
            r#"<div role role="button">Click</div>"#,
            0,
            "first valueless duplicate role masks later values",
        ),
        (
            r#"<div role="button" role="presentation">Click</div>"#,
            1,
            "first interactive duplicate controls",
        ),
        (
            r#"<div role="presentation" role="button">Click</div>"#,
            0,
            "later duplicate role does not override first value",
        ),
        (
            r#"<MyButton role="button">Click</MyButton>"#,
            0,
            "components stay skipped",
        ),
    ] {
        assert_eq!(
            run_over_template(&rule, source),
            expected,
            "template case failed: {label}"
        );
    }
}
