use super::super::{ColorLiteral, CssMode, colors_in};
use super::color_at;

fn tuple(literal: &ColorLiteral) -> (usize, usize, [f32; 4]) {
    (
        literal.start,
        literal.end,
        [literal.red, literal.green, literal.blue, literal.alpha],
    )
}

#[test]
fn selected_query_matches_complete_authored_vectors_at_every_boundary() {
    let cases = [
        (
            CssMode::Stylesheet,
            ".日本🧭 { color: #1234; background: rgb(1 2 3 / 50%); border: hsl(120 50% 50%); --custom: red; }",
        ),
        (
            CssMode::Stylesheet,
            "#abc, rgb(0 0 0) { color: #123; content: '#fff'; background: url(#abc); /* blue */ }",
        ),
        (
            CssMode::Stylesheet,
            ".outer { color:red:hover { color:blue; } @supports (color: #abc) { color:#def; } }",
        ),
        (
            CssMode::Stylesheet,
            "@container style(--theme: red) { .x { color: rgba(0, 128, 255, .5); } }",
        ),
        (
            CssMode::DeclarationList,
            "color:#123; background: rgb(1, 2, 3); outline:rebeccapurple",
        ),
        (
            CssMode::Preprocessor,
            "$theme: red; @mixin x($a: #abc, $b: rgb(1 2 3)) { color: #{'#def'}; color:blue; }",
        ),
        (
            CssMode::Preprocessor,
            "@theme: red; .mixin(@a: #abc) { color:@{name}; border:blue; } // #fff\n",
        ),
        (
            CssMode::IndentedDeclarations,
            ".日本🧭\r\n  color: #123\r\n  background: rgb(1 2 3)\r\n  outline: red\r\n",
        ),
        (
            CssMode::StylusDeclarations,
            "theme = red\n#abc\n  color #123\n  background rgb(1 2 3)\n  outline blue\n",
        ),
        (
            CssMode::Stylesheet,
            ".x { color: rgb(1 /* ```css\n**link** */ 2 3); color: rgba(1, invalid, 3, 1); color:#12345; color:var(--red); }",
        ),
    ];
    for (mode, source) in cases {
        let complete = colors_in(source, (0, source.len()), mode);
        for offset in 0..=source.len() {
            let expected = source
                .is_char_boundary(offset)
                .then(|| {
                    complete
                        .iter()
                        .find(|literal| literal.start <= offset && offset < literal.end)
                        .map(tuple)
                })
                .flatten();
            let actual = color_at(source, (0, source.len()), mode, offset).literal;
            assert_eq!(
                actual.as_ref().map(tuple),
                expected,
                "{source:?} at byte {offset}"
            );
        }
    }
}

#[test]
fn selected_tentative_nested_selector_rolls_back_before_publication() {
    let source = ".outer { color:red:hover { color:blue; } }";
    let red = source.find("red").unwrap();
    let blue = source.find("blue").unwrap();
    let whole = colors_in(source, (0, source.len()), CssMode::Stylesheet);
    assert_eq!(
        whole.iter().map(tuple).collect::<Vec<_>>(),
        vec![(blue, blue + 4, [0.0, 0.0, 1.0, 1.0])]
    );
    for offset in red..red + 3 {
        assert_eq!(
            color_at(source, (0, source.len()), CssMode::Stylesheet, offset)
                .literal
                .as_ref()
                .map(tuple),
            None
        );
    }
    for offset in blue..blue + 4 {
        assert_eq!(
            color_at(source, (0, source.len()), CssMode::Stylesheet, offset)
                .literal
                .as_ref()
                .map(tuple),
            Some((blue, blue + 4, [0.0, 0.0, 1.0, 1.0]))
        );
    }
}

#[test]
fn selected_query_never_scans_the_suffix_after_its_statement_commits() {
    let head = ".head{color:#123;}";
    let offset = head.find("#123").unwrap();
    for repetitions in [1_000, 2_000, 4_000, 8_000] {
        let source = [
            head,
            &".tail{color:red;background:#abcdef;}".repeat(repetitions),
        ]
        .concat();
        let selected = color_at(&source, (0, source.len()), CssMode::Stylesheet, offset);
        assert_eq!(
            selected
                .literal
                .as_ref()
                .map(|literal| (literal.start, literal.end)),
            Some((offset, offset + 4))
        );
        assert_eq!(selected.consumed_bytes, head.len() - 1);
    }
}

#[test]
fn selected_source_work_and_cold_warm_observations_preserve_whole_literals() {
    use super::super::{
        DECLARATION_NAME_WORK, RGB_PROBES, SCAN_STEPS, colors_in_mode_with_metrics,
    };
    let mut observations = Vec::new();
    for repetitions in [1_000, 2_000, 4_000, 8_000] {
        for placement in ["head", "tail"] {
            let repeated = ".tail{color:red;background:#abcdef;}".repeat(repetitions);
            let target = ".selected{color:#336699;}";
            let source = if placement == "head" {
                [target, &repeated].concat()
            } else {
                [&repeated, target].concat()
            };
            let offset = source.find("#336699").unwrap();
            let (all, complete_work, _) =
                colors_in_mode_with_metrics(&source, (0, source.len()), CssMode::Stylesheet);
            assert_eq!(all.len(), repetitions * 2 + 1);
            let expected = (
                offset,
                offset + 7,
                [51.0 / 255.0, 102.0 / 255.0, 153.0 / 255.0, 1.0],
            );
            let mut samples = Vec::new();
            for sample in 0..17 {
                SCAN_STEPS.set(0);
                RGB_PROBES.set(0);
                DECLARATION_NAME_WORK.set(0);
                let started = std::time::Instant::now();
                let selected = color_at(&source, (0, source.len()), CssMode::Stylesheet, offset);
                let nanos = started.elapsed().as_nanos();
                assert_eq!(selected.literal.as_ref().map(tuple), Some(expected));
                samples.push(serde_json::json!({
                    "sample":sample,"phase":if sample==0 {"cold"} else {"warm"},
                    "elapsedNanos":nanos,"scannerWork":SCAN_STEPS.get()+DECLARATION_NAME_WORK.get(),
                    "rgbParserProbes":RGB_PROBES.get(),"consumedPrefixBytes":selected.consumed_bytes
                }));
            }
            observations.push(serde_json::json!({
                "placement":placement,"repetitions":repetitions,"inputBytes":source.len(),
                "completeCollectionWork":complete_work,"completeColorCount":all.len(),
                "completeSelectedLiteral":[expected.0,expected.1,expected.2],"selectedSamples":samples
            }));
        }
    }
    if let Some(path) = std::env::var_os("VIZE_CSS_SELECTED_WORK_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&observations).unwrap()).unwrap();
    }
}
