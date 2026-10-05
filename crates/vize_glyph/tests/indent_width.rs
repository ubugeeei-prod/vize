//! #7793: byte indentation agrees with every supported configured width.
#![expect(
    clippy::disallowed_macros,
    reason = "regressions construct independent complete output and immutable options"
)]
use vize_glyph::{EndOfLine, FormatOptions, VueVersion, format_sfc_with_vue_version};

const SOURCE: &str = "<template><p>hello</p></template>\n";
const REFERENCE_THREE: &str = "<template>\n   <p>hello</p>\n</template>\n";

fn assert_contract(source: &str, expected: &str, options: &FormatOptions, version: VueVersion) {
    let before_source = source.as_bytes().to_vec();
    let before_options = serde_json::to_value(options).unwrap();
    let first = format_sfc_with_vue_version(source, options, version).unwrap();
    assert_eq!(
        first.code.as_bytes(),
        expected.as_bytes(),
        "{options:?}/{version:?}"
    );
    assert_eq!(first.changed, source.as_bytes() != expected.as_bytes());
    let second = format_sfc_with_vue_version(&first.code, options, version).unwrap();
    let third = format_sfc_with_vue_version(&second.code, options, version).unwrap();
    assert_eq!(second.code.as_bytes(), expected.as_bytes());
    assert_eq!(third.code.as_bytes(), expected.as_bytes());
    assert!(!second.changed && !third.changed);
    assert_eq!(source.as_bytes(), before_source);
    assert_eq!(serde_json::to_value(options).unwrap(), before_options);
}

#[test]
fn supported_byte_widths_agree_with_string_and_script_projection() {
    for tab_width in 0..=24 {
        for use_tabs in [false, true] {
            let options = FormatOptions {
                tab_width,
                use_tabs,
                ..FormatOptions::default()
            };
            let expected = if use_tabs {
                "\t".to_owned()
            } else {
                " ".repeat(usize::from(tab_width))
            };
            assert_eq!(options.indent_bytes(), expected.as_bytes());
            assert_eq!(options.indent_string().as_bytes(), expected.as_bytes());
            assert_eq!(
                options.to_oxc_format_options().indent_width.value(),
                tab_width
            );
        }
    }
    for tab_width in 25..=u8::MAX {
        let options = FormatOptions {
            tab_width,
            ..FormatOptions::default()
        };
        assert_eq!(
            options.indent_bytes(),
            b"  ",
            "legacy fallback for {tab_width}"
        );
        assert_eq!(options.to_oxc_format_options().indent_width.value(), 2);
        assert_eq!(options.indent_string().len(), usize::from(tab_width));
        assert_eq!(
            FormatOptions {
                use_tabs: true,
                ..options
            }
            .indent_bytes(),
            b"\t"
        );
    }
}

#[test]
fn supported_sfc_widths_preserve_complete_output_and_fixed_points() {
    for tab_width in 0..=24 {
        for use_tabs in [false, true] {
            let indent = if use_tabs {
                "\t".to_owned()
            } else {
                " ".repeat(usize::from(tab_width))
            };
            for (end_of_line, layout) in [
                (EndOfLine::Lf, "\n"),
                (EndOfLine::Crlf, "\r\n"),
                (EndOfLine::Cr, "\r"),
                (EndOfLine::Auto, "\n"),
            ] {
                let options = FormatOptions {
                    tab_width,
                    use_tabs,
                    end_of_line,
                    ..FormatOptions::default()
                };
                let expected =
                    format!("<template>{layout}{indent}<p>hello</p>{layout}</template>{layout}");
                for version in [VueVersion::V2, VueVersion::V2_7, VueVersion::V3] {
                    assert_contract(SOURCE, &expected, &options, version);
                }
            }
        }
    }
    assert_contract(
        SOURCE,
        "<template>\n  <p>hello</p>\n</template>\n",
        &FormatOptions::default(),
        VueVersion::V3,
    );
}

#[test]
fn unusual_widths_preserve_raw_body_bytes_and_following_layout() {
    let body = "first\n  second\r\n third\r fourth";
    for tab_width in [0, 3, 5, 24] {
        for use_tabs in [false, true] {
            let indent = if use_tabs {
                "\t".to_owned()
            } else {
                " ".repeat(usize::from(tab_width))
            };
            for (end_of_line, layout) in [
                (EndOfLine::Lf, "\n"),
                (EndOfLine::Crlf, "\r\n"),
                (EndOfLine::Cr, "\r"),
            ] {
                let options = FormatOptions {
                    tab_width,
                    use_tabs,
                    end_of_line,
                    ..FormatOptions::default()
                };
                for (open, close) in [
                    ("<pre>", "</pre>"),
                    ("<textarea>", "</textarea>"),
                    ("<listing>", "</listing>"),
                    ("<div v-pre>", "</div>"),
                ] {
                    let source = format!(
                        "<template>{layout}{open}{body}{close}{layout}<p>hello</p>{layout}</template>{layout}"
                    );
                    let expected = format!(
                        "<template>{layout}{indent}{open}{body}{close}{layout}{indent}<p>hello</p>{layout}</template>{layout}"
                    );
                    for version in [VueVersion::V2, VueVersion::V2_7, VueVersion::V3] {
                        assert_contract(&source, &expected, &options, version);
                    }
                }
            }
        }
    }
}

#[test]
fn configured_three_space_corpus_keeps_original_input_and_complete_reference() {
    let source = include_str!(
        "../../../tests/_fixtures/differential/formatter/sfc-three-space-indent/App.vue.txt"
    );
    let expected = include_str!(
        "../../../tests/_fixtures/differential/formatter/sfc-three-space-indent/reference.expected.txt"
    );
    let config: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/formatter/sfc-three-space-indent/vize.config.json"
    ))
    .unwrap();
    let options: FormatOptions = serde_json::from_value(config["formatter"].clone()).unwrap();
    assert_eq!(source, SOURCE);
    assert_eq!(source.len(), 34);
    assert_eq!(expected, REFERENCE_THREE);
    assert_eq!(expected.len(), 39);
    assert_eq!(options.tab_width, 3);
    for version in [VueVersion::V2, VueVersion::V2_7, VueVersion::V3] {
        assert_contract(source, expected, &options, version);
    }
}
