//! #7924: authored leading comments keep tokens while continuation depth stabilizes.
use vize_glyph::{EndOfLine, FormatOptions, format_sfc};

const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/sfc-leading-line-comment-directive/Min.vue.txt"
);
const EXPECTED: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/sfc-leading-line-comment-directive/reference.expected.txt"
);

fn assert_three_passes(source: &str, expected: &str, options: &FormatOptions) {
    let saved_options = serde_json::to_value(options).unwrap();
    let first = format_sfc(source, options).unwrap();
    let second = format_sfc(&first.code, options).unwrap();
    let third = format_sfc(&second.code, options).unwrap();
    assert_eq!(first.code.as_str(), expected);
    assert_eq!(first.changed, source != expected);
    assert_eq!(second.code.as_str(), expected);
    assert_eq!(third.code.as_str(), expected);
    assert_eq!((second.changed, third.changed), (false, false));
    assert_eq!(serde_json::to_value(options).unwrap(), saved_options);
}

#[test]
fn whole_original_leading_comment_reaches_one_fixed_point() {
    assert_eq!(
        ORIGINAL,
        "<template>\n  <input\n    :placeholder=\"// note\n    x\"\n  />\n</template>\n"
    );
    assert_eq!(ORIGINAL.len(), 70);
    assert_eq!(
        EXPECTED,
        "<template>\n  <input\n    :placeholder=\"// note\n      x\"\n  />\n</template>\n"
    );
    assert_three_passes(ORIGINAL, EXPECTED, &FormatOptions::default());
    let crlf = ORIGINAL.replace('\n', "\r\n");
    assert_three_passes(&crlf, EXPECTED, &FormatOptions::default());
    assert_three_passes(
        &crlf,
        &EXPECTED.replace('\n', "\r\n"),
        &FormatOptions {
            end_of_line: EndOfLine::Crlf,
            ..FormatOptions::default()
        },
    );
}

#[test]
fn leading_event_comment_and_relative_code_depth_keep_whole_bytes() {
    for (source, expected) in [
        (
            "<template>\n  <input\n    @click=\"// note\n    run()\"\n  />\n</template>\n",
            "<template>\n  <input\n    @click=\"// note\n      run()\"\n  />\n</template>\n",
        ),
        (
            "<template>\n  <input\n    @click=\"// note\n    () => {\n      run()\n    }\"\n  />\n</template>\n",
            "<template>\n  <input\n    @click=\"// note\n      () => {\n        run()\n      }\"\n  />\n</template>\n",
        ),
        (
            "<template>\n  <input\n    :placeholder=\"// note\n    `first\n  raw`\"\n  />\n</template>\n",
            "<template>\n  <input\n    :placeholder=\"// note\n      `first\n  raw`\"\n  />\n</template>\n",
        ),
    ] {
        assert_three_passes(source, expected, &FormatOptions::default());
    }
}

#[test]
fn following_line_comment_and_quote_placement_stay_verbatim() {
    for source in [
        "<template>\n  <input\n    :placeholder=\"\n      // note\n      x\n    \"\n  />\n</template>\n",
        "<template>\n  <input\n    @click=\"\n      // note\n      () => run()\n    \"\n  />\n</template>\n",
    ] {
        assert_three_passes(source, source, &FormatOptions::default());
    }
}
