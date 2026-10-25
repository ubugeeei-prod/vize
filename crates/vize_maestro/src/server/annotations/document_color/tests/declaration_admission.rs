use super::{Color, ColorInformation, DocumentColorService, Position, Range};

fn expected(source: &str, prefix: &str, literal: &str, rgba: [f32; 4]) -> ColorInformation {
    let relative = prefix.len().checked_sub(literal.len()).unwrap();
    assert_eq!(prefix.get(relative..), Some(literal));
    let start = source.find(prefix).unwrap() + relative;
    let point = |at| {
        let at = vize_l0::line_index::offset_to_position(source, at);
        Position::new(at.line, at.character)
    };
    let [red, green, blue, alpha] = rgba;
    ColorInformation {
        range: Range::new(point(start), point(start + literal.len())),
        color: Color {
            red,
            green,
            blue,
            alpha,
        },
    }
}

fn assert_colors(source: &str, expected: Vec<ColorInformation>) {
    let descriptor = vize_incremental::descriptor::parse_descriptor("/Colors.vue", source).unwrap();
    assert_eq!(DocumentColorService::colors(source, &descriptor), expected);
}

#[test]
fn selector_hex_and_function_lookalikes_never_become_colour_values() {
    for newline in ["\n", "\r\n"] {
        for lang in ["css", "scss", "less", "postcss"] {
            let source = format!(
                "<style lang=\"{lang}\">\n/* 日本 🧭 */\n#abc {{ color: #ff0000; }}\n#ff0000:hover {{ background: red; }}\nrgb(0 0 255) {{ color: blue; }}\n.x {{ button:red {{ --ok: #00ff00; }} }}\n</style>"
            ).replace('\n', newline);
            assert_colors(
                &source,
                vec![
                    expected(&source, "color: #ff0000", "#ff0000", [1., 0., 0., 1.]),
                    expected(&source, "background: red", "red", [1., 0., 0., 1.]),
                    expected(&source, "color: blue", "blue", [0., 0., 1., 1.]),
                    expected(&source, "--ok: #00ff00", "#00ff00", [0., 1., 0., 1.]),
                ],
            );
        }
    }
}

#[test]
fn genuine_at_rule_declaration_conditions_keep_all_colour_notations() {
    let source = "<style>@supports(color: #f00) and (background: rgb(0 255 0)) and (outline: red) and selector(#abc) { .x { color: blue } } @container style(--theme: hsl(0 100% 50%)) { .y { color: #00f } }</style>";
    assert_colors(
        source,
        vec![
            expected(source, "color: #f00", "#f00", [1., 0., 0., 1.]),
            expected(
                source,
                "background: rgb(0 255 0)",
                "rgb(0 255 0)",
                [0., 1., 0., 1.],
            ),
            expected(source, "outline: red", "red", [1., 0., 0., 1.]),
            expected(source, "color: blue", "blue", [0., 0., 1., 1.]),
            expected(
                source,
                "--theme: hsl(0 100% 50%)",
                "hsl(0 100% 50%)",
                [1., 0., 0., 1.],
            ),
            expected(source, "color: #00f", "#00f", [0., 0., 1., 1.]),
        ],
    );
}

#[test]
fn stylus_colonless_named_and_function_values_share_actual_declaration_admission() {
    for lang in ["stylus", "styl"] {
        for newline in ["\n", "\r\n"] {
            let source = format!("<style lang=\"{lang}\">\n#abc\n  color red\n  background #00ff00\n  border-color rgb(0 0 255)\n  outline-color: transparent\n  .red\n    color: blue\n</style>").replace('\n', newline);
            assert_colors(
                &source,
                vec![
                    expected(&source, "color red", "red", [1., 0., 0., 1.]),
                    expected(&source, "background #00ff00", "#00ff00", [0., 1., 0., 1.]),
                    expected(
                        &source,
                        "border-color rgb(0 0 255)",
                        "rgb(0 0 255)",
                        [0., 0., 1., 1.],
                    ),
                    expected(
                        &source,
                        "outline-color: transparent",
                        "transparent",
                        [0., 0., 0., 0.],
                    ),
                    expected(&source, "color: blue", "blue", [0., 0., 1., 1.]),
                ],
            );
        }
    }
}

#[test]
fn preprocessor_expressions_comments_strings_and_urls_do_not_publish_css_colours() {
    for lang in ["scss", "less", "postcss", "sass", "stylus"] {
        let source = format!(
            "<style lang=\"{lang}\">.x {{ color: #{{rgb(255 0 0)}}; --other: @{{red}}; --url: url(red#abc); content: '#ff0000'; /* #abc red */ color: blue; }}</style>"
        );
        assert_colors(
            &source,
            vec![expected(&source, "color: blue", "blue", [0., 0., 1., 1.])],
        );
    }
}

#[test]
fn stylus_assignments_and_preprocessor_parameter_values_are_owned_values() {
    let source = "<style lang=\"stylus\">theme = #f00\n$accent = red\n.card\n  color theme\n  border-color: blue\n</style>";
    assert_colors(
        source,
        vec![
            expected(source, "theme = #f00", "#f00", [1., 0., 0., 1.]),
            expected(source, "$accent = red", "red", [1., 0., 0., 1.]),
            expected(source, "border-color: blue", "blue", [0., 0., 1., 1.]),
        ],
    );
    let source = "<style lang=\"scss\">@mixin theme($base: #f00, $accent: red) { .x { color: blue; } }</style>";
    assert_colors(
        source,
        vec![
            expected(source, "$base: #f00", "#f00", [1., 0., 0., 1.]),
            expected(source, "$accent: red", "red", [1., 0., 0., 1.]),
            expected(source, "color: blue", "blue", [0., 0., 1., 1.]),
        ],
    );
}

#[test]
fn rejected_supports_headers_keep_linear_internal_work() {
    let mut previous = 0;
    let mut observations = Vec::new();
    for count in [1_000, 2_000, 4_000, 8_000] {
        let source = format!(
            "@supports (/*{}*/ #bad{}) {{ .x {{ color: blue; }} }}",
            "x".repeat(count),
            ":a".repeat(count)
        );
        let (found, work, rgb_probes) = super::super::scan::colors_in_mode_with_metrics(
            &source,
            (0, source.len()),
            super::super::scan::CssMode::Stylesheet,
        );
        let values = found
            .iter()
            .map(|literal| {
                (
                    literal.start,
                    literal.end,
                    literal.red,
                    literal.green,
                    literal.blue,
                    literal.alpha,
                )
            })
            .collect::<Vec<_>>();
        let at = source.find("blue").unwrap();
        assert_eq!(values, vec![(at, at + 4, 0., 0., 1., 1.)]);
        observations.push(serde_json::json!({
            "inputBytes": source.len(), "repetitions": count,
            "scannerWork": work, "rgbParserProbes": rgb_probes,
            "completeColors": values,
        }));
        if previous > 0 {
            assert!(work <= previous * 2 + 32, "{count}: {work}");
        }
        previous = work;
    }
    if let Ok(path) = std::env::var("VIZE_CSS_COLOR_WORK_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&observations).unwrap()).unwrap();
    }
}
