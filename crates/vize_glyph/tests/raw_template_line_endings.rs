//! #7745: configured layout must not normalize authored raw-region separators.
#![expect(
    clippy::disallowed_macros,
    reason = "regressions construct complete independent expected bytes"
)]
use vize_glyph::{EndOfLine, FormatOptions, VueVersion, format_sfc_with_vue_version};

#[test]
fn mixed_raw_line_ending_corpus_preserves_every_byte_and_reaches_a_fixed_point() {
    let source = include_str!(
        "../../../tests/_fixtures/differential/formatter/sfc-mixed-raw-line-endings/App.vue.txt"
    );
    let expected = include_bytes!(
        "../../../tests/_fixtures/differential/formatter/sfc-mixed-raw-line-endings/reference.expected.txt"
    );
    let options = FormatOptions {
        end_of_line: EndOfLine::Crlf,
        ..FormatOptions::default()
    };
    let first = format_sfc_with_vue_version(source, &options, VueVersion::V3).unwrap();
    assert_eq!(first.code.as_bytes(), expected);
    assert!(first.changed);
    for _ in 0..2 {
        let fixed = format_sfc_with_vue_version(&first.code, &options, VueVersion::V3).unwrap();
        assert_eq!(fixed.code.as_bytes(), expected);
        assert!(!fixed.changed);
    }
    assert_eq!(options.end_of_line, EndOfLine::Crlf);
}

#[test]
fn raw_separators_survive_every_layout_style_and_vue_version() {
    for (ending, layout) in [
        (EndOfLine::Lf, "\n"),
        (EndOfLine::Crlf, "\r\n"),
        (EndOfLine::Cr, "\r"),
    ] {
        let options = FormatOptions {
            end_of_line: ending,
            ..FormatOptions::default()
        };
        for raw in ["\n", "\r\n", "\r"] {
            for (open, close) in [
                ("<pre>", "</pre>"),
                ("<textarea>", "</textarea>"),
                ("<listing>", "</listing>"),
                ("<div v-pre>", "</div>"),
            ] {
                for body in [
                    format!("first{raw}  second{raw}{raw}  third"),
                    format!("{raw}first{raw}  second{raw}{raw}  third{raw}"),
                    "first\n  second\r\n  third\r  fourth".to_owned(),
                ] {
                    let source = format!(
                        "<template>{layout}{open}{body}{close}{layout}<p>{{{{value}}}}</p>{layout}</template>{layout}"
                    );
                    let expected = format!(
                        "<template>{layout}  {open}{body}{close}{layout}  <p>{{{{ value }}}}</p>{layout}</template>{layout}"
                    );
                    for version in [VueVersion::V2, VueVersion::V2_7, VueVersion::V3] {
                        let first =
                            format_sfc_with_vue_version(&source, &options, version).unwrap();
                        assert_eq!(
                            first.code.as_bytes(),
                            expected.as_bytes(),
                            "{ending:?}/{raw:?}/{open}/{version:?}"
                        );
                        for _ in 0..2 {
                            let fixed = format_sfc_with_vue_version(&first.code, &options, version)
                                .unwrap();
                            assert_eq!(fixed.code.as_bytes(), expected.as_bytes());
                            assert!(!fixed.changed);
                        }
                    }
                }
            }
        }
    }
}
