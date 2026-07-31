use tower_lsp::lsp_types::Color;

use super::DocumentColorService;

/// One colour as `(line, start_character, end_character, r, g, b, a)` with the
/// channels back in 0..=255 / 0..=100 so a test reads like the CSS it pins.
type FlatColor = (u32, u32, u32, u32, u32, u32, u32);

fn colors(source: &str) -> Vec<FlatColor> {
    DocumentColorService::colors(source, "/App.vue")
        .into_iter()
        .map(|info| {
            assert_eq!(
                info.range.start.line, info.range.end.line,
                "a colour literal never spans lines: {info:?}"
            );
            (
                info.range.start.line,
                info.range.start.character,
                info.range.end.character,
                byte(info.color.red),
                byte(info.color.green),
                byte(info.color.blue),
                percent(info.color.alpha),
            )
        })
        .collect()
}

fn byte(channel: f32) -> u32 {
    (channel * 255.0).round() as u32
}

fn percent(alpha: f32) -> u32 {
    (alpha * 100.0).round() as u32
}

fn labels(red: f32, green: f32, blue: f32, alpha: f32) -> Vec<String> {
    DocumentColorService::presentations(Color {
        red,
        green,
        blue,
        alpha,
    })
    .into_iter()
    .map(|presentation| {
        assert!(
            presentation.text_edit.is_none(),
            "the label is the inserted text; a textEdit would override the client's own range"
        );
        presentation.label
    })
    .collect()
}

#[test]
fn every_hex_form_is_recognised_with_its_exact_span() {
    let source = "<style>\n.a { color: #f00; background: #ff0000; border-color: #f00c; outline-color: #ff0000cc }\n</style>\n";
    assert_eq!(
        colors(source),
        vec![
            (1, 12, 16, 255, 0, 0, 100),
            (1, 30, 37, 255, 0, 0, 100),
            (1, 53, 58, 255, 0, 0, 80),
            (1, 75, 84, 255, 0, 0, 80),
        ]
    );
}

#[test]
fn a_hex_run_that_is_not_a_colour_is_not_one() {
    // 5 and 7 digits are not CSS colours, and `#abcdef0` must not be read as
    // `#abcdef` with a stray `0`.
    let source = "<style>\n.a { --x: #abcde; --y: #abcdef0; --z: #ffgg }\n</style>\n";
    assert_eq!(colors(source), Vec::new());
}

#[test]
fn functional_notation_is_recognised_in_both_syntaxes() {
    let source = "<style>\n.a { color: rgb(255, 0, 0); background: rgba(255, 0, 0, 0.5); border-color: rgb(100% 0% 0% / 50%) }\n</style>\n";
    assert_eq!(
        colors(source),
        vec![
            (1, 12, 26, 255, 0, 0, 100),
            (1, 40, 60, 255, 0, 0, 50),
            (1, 76, 97, 255, 0, 0, 50),
        ]
    );
}

#[test]
fn a_colour_inside_a_css_comment_is_not_offered() {
    // A swatch there would offer to rewrite text the stylesheet never reads.
    let source = "<style>\n/* #f00 */\n.a { color: #0f0 }\n</style>\n";
    assert_eq!(colors(source), vec![(2, 12, 16, 0, 255, 0, 100)]);
}

#[test]
fn a_static_style_attribute_is_css_and_a_bound_one_is_not() {
    let source = "<template>\n  <div style=\"color: #f00\" />\n  <span :style=\"{ color: '#0f0' }\" />\n</template>\n";
    // Only the static attribute contributes: `:style` holds a JavaScript
    // expression, so `'#0f0'` is a string literal in code, not CSS.
    assert_eq!(colors(source), vec![(1, 21, 25, 255, 0, 0, 100)]);
}

#[test]
fn an_attribute_whose_name_merely_ends_in_style_is_not_scanned() {
    let source = "<template>\n  <div data-style=\"#f00\" />\n</template>\n";
    assert_eq!(colors(source), Vec::new());
}

#[test]
fn script_and_template_text_are_never_scanned_for_colours() {
    // `#f00` in a script body or a text node is not CSS.
    let source = "<script setup lang=\"ts\">\nconst tag = '#f00'\n</script>\n\n<template>\n  <div>#00ff00</div>\n</template>\n";
    assert_eq!(colors(source), Vec::new());
}

#[test]
fn presentations_offer_hex_first_then_the_functional_form() {
    assert_eq!(
        labels(1.0, 0.0, 0.0, 1.0),
        vec!["#ff0000".to_string(), "rgb(255, 0, 0)".to_string()]
    );
    // A translucent colour needs the 8-digit hex and `rgba()`; the alpha is
    // written the way CSS writes it, not as `0.500000`.
    assert_eq!(
        labels(1.0, 0.0, 0.0, 0.5),
        vec!["#ff000080".to_string(), "rgba(255, 0, 0, 0.5)".to_string()]
    );
    assert_eq!(
        labels(0.0, 0.0, 0.0, 0.0),
        vec!["#00000000".to_string(), "rgba(0, 0, 0, 0)".to_string()]
    );
}
