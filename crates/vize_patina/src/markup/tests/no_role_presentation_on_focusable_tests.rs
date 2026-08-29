use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::a11y::NoRolePresentationOnFocusable;
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
fn no_role_presentation_on_focusable_template() {
    let rule = NoRolePresentationOnFocusable;
    for (source, expected, label) in [
        (r#"<div role="presentation"></div>"#, 0, "non-focusable div"),
        (
            r#"<a role="presentation">decorative</a>"#,
            0,
            "anchor without href",
        ),
        (
            r#"<a href="/" role="presentation">Home</a>"#,
            1,
            "anchor with static href",
        ),
        (
            r#"<a :href="url" role="presentation">Home</a>"#,
            1,
            "anchor with static-arg bound href",
        ),
        (
            r#"<a :[href]="url" role="presentation">Home</a>"#,
            0,
            "anchor with dynamic href argument",
        ),
        (
            r#"<area href="/map" role="presentation" />"#,
            1,
            "area with static href",
        ),
        (
            r#"<button role="presentation">Click</button>"#,
            1,
            "native button presentation",
        ),
        (
            r#"<input role="none" type="text" />"#,
            1,
            "native input none",
        ),
        (
            r#"<button role="button">Click</button>"#,
            0,
            "non-presentation role",
        ),
        (
            r#"<button role="Presentation">Click</button>"#,
            0,
            "role value is exact",
        ),
        (
            r#"<button role="presentation ">Click</button>"#,
            0,
            "role value is not trimmed",
        ),
        (
            r#"<button role="presentation button">Click</button>"#,
            0,
            "role value is not tokenized",
        ),
        (
            r#"<button ROLE="presentation">Click</button>"#,
            0,
            "role attribute name is exact",
        ),
        (
            r#"<button :role="'presentation'">Click</button>"#,
            0,
            "bound role stays dynamic",
        ),
        (
            r#"<button role>Click</button>"#,
            0,
            "valueless role stays clean",
        ),
        (
            r#"<button role role="presentation">Click</button>"#,
            0,
            "first valueless duplicate role masks later values",
        ),
        (
            r#"<button role="presentation" role="button">Click</button>"#,
            1,
            "first presentation duplicate controls",
        ),
        (
            r#"<button role="none" role="button">Click</button>"#,
            1,
            "first none duplicate controls",
        ),
        (
            r#"<button role="button" role="presentation">Click</button>"#,
            0,
            "later duplicate role does not override first value",
        ),
        (
            r#"<div tabindex="0" role="presentation">Focusable</div>"#,
            1,
            "tabindex zero",
        ),
        (
            r#"<div :tabindex="0" role="presentation">Focusable</div>"#,
            0,
            "bound tabindex stays outside the legacy static-value helper",
        ),
        (
            r#"<div tabindex="" role="presentation">Focusable</div>"#,
            1,
            "empty tabindex remains focusable",
        ),
        (
            r#"<div tabindex="x" role="presentation">Focusable</div>"#,
            1,
            "non-numeric tabindex remains focusable",
        ),
        (
            r#"<div tabindex="-1" role="presentation">Programmatic</div>"#,
            0,
            "negative tabindex",
        ),
        (
            r#"<div tabindex tabindex="0" role="presentation">Maybe focusable</div>"#,
            0,
            "first valueless duplicate tabindex masks later values",
        ),
        (
            r#"<div contenteditable="true" role="presentation">Edit</div>"#,
            1,
            "contenteditable true",
        ),
        (
            r#"<div contenteditable="" role="presentation">Edit</div>"#,
            1,
            "empty contenteditable remains focusable",
        ),
        (
            r#"<div contenteditable="plaintext-only" role="presentation">Edit</div>"#,
            1,
            "plaintext-only contenteditable remains focusable",
        ),
        (
            r#"<div contenteditable="FALSE" role="presentation">Edit</div>"#,
            1,
            "contenteditable value is exact",
        ),
        (
            r#"<div contenteditable="false" role="presentation">Edit</div>"#,
            0,
            "contenteditable false",
        ),
        (
            r#"<div :contenteditable="true" role="presentation">Edit</div>"#,
            0,
            "bound contenteditable stays outside the legacy static-value helper",
        ),
        (
            r#"<div contenteditable="false" contenteditable="true" role="presentation">Edit</div>"#,
            0,
            "later duplicate contenteditable does not override first value",
        ),
        (
            r#"<audio role="presentation"></audio>"#,
            0,
            "audio is not focusable in the legacy helper",
        ),
        (
            r#"<video role="presentation"></video>"#,
            0,
            "video is not focusable in the legacy helper",
        ),
        (
            r#"<details role="presentation"></details>"#,
            0,
            "details is not focusable in the legacy helper",
        ),
        (
            r#"<MyButton role="presentation">Click</MyButton>"#,
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
