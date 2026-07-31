use tower_lsp::lsp_types::{Color, ColorInformation, Position, Range};

use super::DocumentColorService;

/// `(line, start, end, r, g, b, a)`, with byte channels and percent alpha.
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
    // `#abcdef0` must not be read as `#abcdef` with a stray `0`.
    let source = "<style>\n.a { --x: #abcde; --y: #abcdef0; --z: #ffgg }\n</style>\n";
    assert_eq!(colors(source), Vec::new());
}

#[test]
fn functional_notation_is_recognised_in_both_syntaxes() {
    let source = "<style>\n.a { color: rgb(255, 0, 0); background: rgba(255, 0, 0, 0.5); border-color: rgb(100% 0% 0% / 50%); outline-color: r\\67 b(0 0 255) }\n</style>\n";
    assert_eq!(
        colors(source),
        vec![
            (1, 12, 26, 255, 0, 0, 100),
            (1, 40, 60, 255, 0, 0, 50),
            (1, 76, 97, 255, 0, 0, 50),
            (1, 114, 129, 0, 0, 255, 100),
        ]
    );
}

#[test]
fn existing_colour_forms_remain_visible_in_supports_conditions() {
    let source = "<style>\n@supports (color: #f00) and (background: rgb(0, 255, 0)) { .a { color: blue } }\n</style>\n";
    assert_eq!(
        colors(source),
        vec![
            (1, 18, 22, 255, 0, 0, 100),
            (1, 41, 55, 0, 255, 0, 100),
            (1, 71, 75, 0, 0, 255, 100),
        ]
    );
}

#[test]
fn invalid_rgb_functions_do_not_expose_named_arguments() {
    let source = "<style>\n.a { --a: rgb(red 0 0); --b: r\\67 b(red 0 0); --c: RGBA(blue, 0, 0, 1); --d: \\72 gba(blue, 0, 0, 1); color: green }\n</style>\n";
    assert_eq!(colors(source), vec![(1, 108, 113, 0, 128, 0, 100)]);
}

#[test]
fn named_colours_report_the_complete_exact_output() {
    let source = "<style>\n.a { color: red; background: ReBeCcApUrPlE; border-color: transparent; outline: gray; caret-color: grey; --h: hsl(0 100% 50%); --ha: hsla(0, 100%, 50%, .5) }\n</style>\n";
    assert_eq!(
        colors(source),
        vec![
            (1, 12, 15, 255, 0, 0, 100),
            (1, 29, 42, 102, 51, 153, 100),
            (1, 58, 69, 0, 0, 0, 0),
            (1, 80, 84, 128, 128, 128, 100),
            (1, 99, 103, 128, 128, 128, 100),
            (1, 110, 125, 255, 0, 0, 100),
            (1, 133, 155, 255, 0, 0, 50),
        ]
    );
}

#[test]
fn hsl_supports_modern_legacy_units_wrapping_and_alpha() {
    let source = "<style>\n.a { --a: hsl(120deg 100% 25%); --b: hsl(240 100% 50% / 25%); --c: hsla(.5turn, 100%, 50%, .5); --d: hsl(200grad 100% 50%); --e: hsl(-120 100% 50%); --f: hsl(30 0% 25%); --g: h\\73l(60 100% 50%); --h: hsl(0 100 50); --i: hsl(none none none / none); --j: hsl(0/**/100%/**/50%); --k: hsl(30 300% 75%) }\n</style>\n";
    let found = colors(source);
    assert_eq!(found.len(), 11, "{found:?}");
    assert_eq!(
        found
            .into_iter()
            .map(|(_, _, _, red, green, blue, alpha)| (red, green, blue, alpha))
            .collect::<Vec<_>>(),
        vec![
            (0, 128, 0, 100),
            (0, 0, 255, 25),
            (0, 255, 255, 50),
            (0, 255, 255, 100),
            (0, 0, 255, 100),
            (64, 64, 64, 100),
            (255, 255, 0, 100),
            (255, 0, 0, 100),
            (0, 0, 0, 0),
            (255, 0, 0, 100),
            (255, 191, 0, 100),
        ]
    );
}

#[test]
fn malformed_hsl_is_not_offered_as_a_colour() {
    let source = "<style>\n.a { --a: hsl(0, 100, 50); --b: hsl(0, 100%, 50% / .5); --c: hsl(0 100%); --d: hsl(NaN 100% 50%); --e: hsl(0 100% 50% extra); --f: hsl(red 100% 50%); --g: hsl(0. 100% 50%); --h: hsl(0 100.% 50%); --i: hsl(0 100% 50% / 1.); --j: hsl(0\u{000b}100%\u{000b}50%); --k: hsl(0,\u{00a0}100%,\u{00a0}50%) }\n</style>\n";
    assert_eq!(colors(source), Vec::new());
}

#[test]
fn preprocessor_variables_keep_named_and_existing_colour_forms() {
    let source = "<style lang=\"scss\">\n$named: red;\n$hex: #0f0;\n$rgb: rgb(0, 0, 255);\n// red #f00 rgb(255, 0, 0)\n</style>\n<style lang=\"less\">\n@named: blue;\n@hex: #fff;\n@rgb: rgb(255, 0, 0);\n// red #f00 rgb(255, 0, 0)\n</style>\n";
    assert_eq!(
        colors(source),
        vec![
            (1, 8, 11, 255, 0, 0, 100),
            (2, 6, 10, 0, 255, 0, 100),
            (3, 6, 20, 0, 0, 255, 100),
            (7, 8, 12, 0, 0, 255, 100),
            (8, 6, 10, 255, 255, 255, 100),
            (9, 6, 20, 255, 0, 0, 100),
        ]
    );
}

#[test]
fn indented_sass_keeps_named_and_existing_colour_forms() {
    let source = "<style lang=\"sass\">\n.a\n  color: red\n  background: #0f0\n  border-color: rgb(0, 0, 255)\n  // red #f00 rgb(255, 0, 0)\n</style>\n";
    assert_eq!(
        colors(source),
        vec![
            (2, 9, 12, 255, 0, 0, 100),
            (3, 14, 18, 0, 255, 0, 100),
            (4, 16, 30, 0, 0, 255, 100),
        ]
    );
}

#[test]
fn css_escape_whitespace_follows_css_input_preprocessing() {
    let source = "<style>\r\n.a { color: r\\65\r\nd; outline: r\\65\u{000b}d }\r\n</style>\r\n";
    assert_eq!(
        DocumentColorService::colors(source, "/App.vue"),
        vec![ColorInformation {
            range: Range {
                start: Position {
                    line: 1,
                    character: 12,
                },
                end: Position {
                    line: 2,
                    character: 1,
                },
            },
            color: Color {
                red: 1.0,
                green: 0.0,
                blue: 0.0,
                alpha: 1.0,
            },
        }]
    );
}

#[test]
fn escaped_hash_tokens_consume_crlf_but_not_vertical_tab() {
    let source = "<style>\r\n.a { --crlf: #x\\20\r\nred; --vt: #x\\20\u{000b}red; color: blue }\r\n</style>\r\n";
    assert_eq!(
        colors(source),
        vec![(2, 17, 20, 255, 0, 0, 100), (2, 29, 33, 0, 0, 255, 100),]
    );
}

#[test]
fn named_colours_require_identifier_boundaries_and_skip_comments() {
    let source = r#"<style>
/* red BLUE transparent */
.reddish { --redish: 1; border-red: 2; color: redish; --a: éred; --b: redé; --c: \ red; --d: red\ish; --e: x\20 red }
</style>
"#;
    assert_eq!(colors(source), Vec::new());
}

#[test]
fn named_colours_are_limited_to_css_value_tokens() {
    let source = "<style>\n.red, #red { content: \"red /*\"; background-image: url(red); width: tan(red); color: blue }\n</style>\n";
    assert_eq!(colors(source), vec![(1, 84, 88, 0, 0, 255, 100)]);
}

#[test]
fn nested_pseudo_selectors_are_not_colour_values() {
    let source = "<style>\n.outer {\n  :deep(.red) { color: blue }\n  :is(.red) { color: blue }\n  :global(.red) { color: blue }\n  a:is(.red) { color: blue }\n  &:global(.red) { color: blue }\n}\n</style>\n";
    assert_eq!(
        colors(source),
        vec![
            (2, 23, 27, 0, 0, 255, 100),
            (3, 21, 25, 0, 0, 255, 100),
            (4, 25, 29, 0, 0, 255, 100),
            (5, 22, 26, 0, 0, 255, 100),
            (6, 26, 30, 0, 0, 255, 100),
        ]
    );
}

#[test]
fn escaped_urls_and_preprocessor_or_hash_tokens_are_not_colours() {
    let source = r#"<style>
.a { background: u\72l(red), \75rl(blue), u\000072 l(tan); --scss: $red; --less: @blue; --hash: #red; color: green }
</style>
"#;
    assert_eq!(colors(source), vec![(1, 109, 114, 0, 128, 0, 100)]);
}

#[test]
fn comment_trivia_before_a_declaration_colon_keeps_value_context() {
    let source = "<style>\n.a { color/* red */: blue }\n</style>\n";
    assert_eq!(colors(source), vec![(1, 21, 25, 0, 0, 255, 100)]);
}

#[test]
fn escaped_named_colours_use_the_complete_authored_identifier_range() {
    let source = r#"<style>
.a { color: r\65 d; background: \72 ed }
.b { --a: \ red; --b: x\ red; color: blue }
</style>
"#;
    assert_eq!(
        colors(source),
        vec![
            (1, 12, 18, 255, 0, 0, 100),
            (1, 32, 38, 255, 0, 0, 100),
            (2, 37, 41, 0, 0, 255, 100),
        ]
    );
}

#[test]
fn named_colour_scanning_has_linear_work_at_1k_through_8k() {
    let mut previous_steps = 0;
    for count in [1_000, 2_000, 4_000, 8_000] {
        let mut source = String::from("--colours: ");
        source.reserve(count * 4);
        for _ in 0..count {
            source.push_str("red ");
        }
        let source_len = source.len();
        let (found, steps, rgb_probes) =
            super::scan::colors_in_with_metrics(&source, (0, source_len), true);
        assert_eq!(found.len(), count);
        assert_eq!(
            rgb_probes, 0,
            "named tokens must never probe the rgb parser"
        );
        assert!(
            steps <= source_len,
            "scanner revisited input at {count} tokens"
        );
        if previous_steps > 0 {
            assert!(
                steps <= previous_steps * 2 + 16,
                "scanner work grew faster than input at {count} tokens"
            );
        }
        previous_steps = steps;
    }
}

#[test]
fn declaration_context_scanning_counts_internal_work_linearly() {
    let mut previous_work = 0;
    for count in [1_000, 2_000, 4_000, 8_000] {
        let source = format!(
            ".a {{ /*{}*/ --chain: {}red; }}",
            "x".repeat(count),
            "a:".repeat(count)
        );
        let source_len = source.len();
        let (found, work, rgb_probes) =
            super::scan::colors_in_with_metrics(&source, (0, source_len), false);
        assert_eq!(found.len(), 1);
        assert_eq!(rgb_probes, 0);
        assert!(work <= source_len * 2, "too much work at {count}: {work}");
        if previous_work > 0 {
            assert!(
                work <= previous_work * 2 + 32,
                "internal work grew faster than input at {count}: {work}"
            );
        }
        previous_work = work;
    }
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
