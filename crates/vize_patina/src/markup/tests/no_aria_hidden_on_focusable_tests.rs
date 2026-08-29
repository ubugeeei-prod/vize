use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::a11y::NoAriaHiddenOnFocusable;
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
fn no_aria_hidden_on_focusable_template() {
    let rule = NoAriaHiddenOnFocusable;
    for (source, expected, label) in [
        (r#"<div aria-hidden="true"></div>"#, 0, "non-focusable div"),
        (
            r#"<a aria-hidden="true">decorative</a>"#,
            0,
            "anchor without href",
        ),
        (
            r#"<a href="/" aria-hidden="true">Home</a>"#,
            1,
            "anchor with static href",
        ),
        (
            r#"<a :href="url" aria-hidden="true">Home</a>"#,
            1,
            "anchor with static-arg bound href",
        ),
        (
            r#"<a :[href]="url" aria-hidden="true">Home</a>"#,
            0,
            "anchor with dynamic href argument",
        ),
        (
            r#"<area href="/map" aria-hidden="true" />"#,
            1,
            "area with static href",
        ),
        (
            r#"<button aria-hidden="true">Click</button>"#,
            1,
            "native button",
        ),
        (r#"<input aria-hidden="true" />"#, 1, "native input"),
        (
            r#"<button aria-hidden="false">Click</button>"#,
            0,
            "aria-hidden false",
        ),
        (
            r#"<button aria-hidden="">Click</button>"#,
            0,
            "empty aria-hidden value",
        ),
        (
            r#"<button aria-hidden="True">Click</button>"#,
            0,
            "aria-hidden value is exact",
        ),
        (
            r#"<button :aria-hidden="true">Click</button>"#,
            0,
            "bound aria-hidden boolean stays dynamic",
        ),
        (
            r#"<button :aria-hidden="'true'">Click</button>"#,
            0,
            "bound aria-hidden stays dynamic",
        ),
        (
            r#"<button ARIA-HIDDEN="true">Click</button>"#,
            0,
            "aria-hidden attribute name is exact",
        ),
        (
            r#"<button aria-hidden>Click</button>"#,
            0,
            "valueless aria-hidden stays clean",
        ),
        (
            r#"<button aria-hidden aria-hidden="true">Click</button>"#,
            0,
            "first valueless duplicate aria-hidden masks later values",
        ),
        (
            r#"<button aria-hidden="true" aria-hidden="false">Click</button>"#,
            1,
            "first true duplicate aria-hidden controls",
        ),
        (
            r#"<button aria-hidden="false" aria-hidden="true">Click</button>"#,
            0,
            "later duplicate aria-hidden does not override first value",
        ),
        (
            r#"<div tabindex="0" aria-hidden="true">Focusable</div>"#,
            1,
            "tabindex zero",
        ),
        (
            r#"<div :tabindex="0" aria-hidden="true">Focusable</div>"#,
            0,
            "bound tabindex stays outside the legacy static-value helper",
        ),
        (
            r#"<div tabindex="" aria-hidden="true">Focusable</div>"#,
            1,
            "empty tabindex remains focusable",
        ),
        (
            r#"<div tabindex="-1" aria-hidden="true">Programmatic</div>"#,
            0,
            "negative tabindex",
        ),
        (
            r#"<div tabindex="x" aria-hidden="true">Focusable</div>"#,
            1,
            "non-numeric tabindex remains focusable",
        ),
        (
            r#"<div tabindex aria-hidden="true">Maybe focusable</div>"#,
            0,
            "valueless tabindex matches legacy static-value helper",
        ),
        (
            r#"<div tabindex tabindex="0" aria-hidden="true">Maybe focusable</div>"#,
            0,
            "first valueless duplicate tabindex masks later values",
        ),
        (
            r#"<div contenteditable="true" aria-hidden="true">Edit</div>"#,
            1,
            "contenteditable true",
        ),
        (
            r#"<div contenteditable="" aria-hidden="true">Edit</div>"#,
            1,
            "empty contenteditable remains focusable",
        ),
        (
            r#"<div contenteditable="plaintext-only" aria-hidden="true">Edit</div>"#,
            1,
            "plaintext-only contenteditable remains focusable",
        ),
        (
            r#"<div contenteditable="FALSE" aria-hidden="true">Edit</div>"#,
            1,
            "contenteditable value is exact",
        ),
        (
            r#"<div contenteditable="false" aria-hidden="true">Edit</div>"#,
            0,
            "contenteditable false",
        ),
        (
            r#"<div :contenteditable="true" aria-hidden="true">Edit</div>"#,
            0,
            "bound contenteditable stays outside the legacy static-value helper",
        ),
        (
            r#"<div contenteditable="false" contenteditable="true" aria-hidden="true">Edit</div>"#,
            0,
            "later duplicate contenteditable does not override first value",
        ),
        (
            r#"<select aria-hidden="true"></select>"#,
            1,
            "native select",
        ),
        (
            r#"<textarea aria-hidden="true"></textarea>"#,
            1,
            "native textarea",
        ),
        (
            r#"<summary aria-hidden="true">Summary</summary>"#,
            1,
            "native summary",
        ),
        (
            r#"<audio aria-hidden="true"></audio>"#,
            0,
            "audio is not focusable in the legacy helper",
        ),
        (
            r#"<video aria-hidden="true"></video>"#,
            0,
            "video is not focusable in the legacy helper",
        ),
        (
            r#"<details aria-hidden="true"></details>"#,
            0,
            "details is not focusable in the legacy helper",
        ),
        (
            r#"<MyButton aria-hidden="true">Click</MyButton>"#,
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
