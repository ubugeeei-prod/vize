//! A tag that continues the line after its parent's `>` stays whole when it
//! overflows: the parent's `>` moves to the next line instead, as Prettier
//! does. The Prettier preset also writes the parent's closing `>` on its own
//! line, as Prettier does.
use vize_glyph::{EndOfLine, FormatOptions, FormatPreset, format_sfc};

fn assert_fixed_point(input: &str, expected: &str, options: &FormatOptions) {
    let mut current: vize_l0::String = input.into();
    for pass in 0..3 {
        let result = format_sfc(&current, options).expect("format");
        assert_eq!(result.code.as_str(), expected, "pass {pass}");
        assert_eq!(result.changed, current.as_str() != expected, "pass {pass}");
        current = result.code;
    }
}

const INPUT: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/sfc-continued-tag-parent-bracket/App.vue.txt"
);

#[test]
fn continued_tag_moves_the_parent_bracket_instead_of_wrapping() {
    let expected = include_str!(
        "../../../tests/_fixtures/differential/formatter/sfc-continued-tag-parent-bracket/reference.expected.txt"
    );
    assert_fixed_point(INPUT, expected, &FormatOptions::default());
    let crlf = FormatOptions {
        end_of_line: EndOfLine::Crlf,
        ..FormatOptions::default()
    };
    assert_fixed_point(
        &INPUT.replace('\n', "\r\n"),
        &expected.replace('\n', "\r\n"),
        &crlf,
    );
}

#[test]
fn prettier_continued_tag_matches_prettier() {
    // Prettier 3.10.1 output for the same input.
    let input = include_str!(
        "../../../tests/_fixtures/differential/formatter/sfc-prettier-continued-tag-parent-bracket/App.vue.txt"
    );
    let expected = include_str!(
        "../../../tests/_fixtures/differential/formatter/sfc-prettier-continued-tag-parent-bracket/reference.expected.txt"
    );
    assert_eq!(input, INPUT);
    assert_fixed_point(
        input,
        expected,
        &FormatOptions::from_preset(FormatPreset::Prettier),
    );
}

#[test]
fn parent_bracket_stays_when_moving_it_cannot_help() {
    let options = FormatOptions::from_preset(FormatPreset::Prettier);
    // The parent's `>` already starts its line below wrapped attributes.
    let wrapped = concat!(
        "<template>\n",
        "  <el-button\n",
        "    plain\n",
        "    size=\"small\"\n",
        "    type=\"primary\"\n",
        "    @click=\"previewTheSelectedDocument(true)\"\n",
        "  ><i class=\"el-icon-view-ab el-icon-with-a-longer-name\"></i>Prev</el-button>\n",
        "</template>\n",
    );
    let formatted = format_sfc(wrapped, &options).expect("format").code;
    assert!(
        !formatted.contains("\n  \n"),
        "a moved bracket must not leave a blank indented line: {formatted}"
    );
    assert_fixed_point(&formatted, &formatted, &options);

    // Even on its own line the child does not fit, so its attributes wrap.
    let long = "<template>\n  <div>\n    <b><i class=\"a-class-name-long-enough-that-the-tag-never-fits-on-one-line-alone\" title=\"x\"></i></b>\n  </div>\n</template>\n";
    let formatted = format_sfc(long, &options).expect("format").code;
    assert!(formatted.contains("<b><i\n"), "{formatted}");
    assert_fixed_point(&formatted, &formatted, &options);
}
