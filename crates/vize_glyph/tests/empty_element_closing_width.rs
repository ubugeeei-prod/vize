//! An empty element prints `></tag>` on its start-tag line, so the closing tag
//! counts toward that line's width exactly like ` />` on a self-closing tag.
use vize_glyph::{EndOfLine, FormatOptions, FormatPreset, format_sfc};

fn assert_fixed_point(input: &str, expected: &str, options: &FormatOptions) {
    let mut current: vize_l0::String = input.into();
    for pass in 0..3 {
        let result = format_sfc(&current, options).expect("format");
        assert_eq!(result.code.as_str(), expected, "pass {pass}");
        assert_eq!(result.changed, current.as_str() != expected, "pass {pass}");
        current = result.code;
    }
    let width = options.print_width as usize;
    assert!(expected.lines().all(|line| line.len() <= width));
}

#[test]
fn empty_element_wraps_attributes_when_its_closing_tag_overflows() {
    let input = include_str!(
        "../../../tests/_fixtures/differential/formatter/sfc-empty-element-closing-width/App.vue.txt"
    );
    let expected = include_str!(
        "../../../tests/_fixtures/differential/formatter/sfc-empty-element-closing-width/reference.expected.txt"
    );
    assert_fixed_point(input, expected, &FormatOptions::default());
    let crlf = FormatOptions {
        end_of_line: EndOfLine::Crlf,
        ..FormatOptions::default()
    };
    assert_fixed_point(
        &input.replace('\n', "\r\n"),
        &expected.replace('\n', "\r\n"),
        &crlf,
    );
}

#[test]
fn prettier_output_with_wrapped_empty_elements_is_unchanged() {
    // Prettier 3.10.1 leaves this file unchanged at its default print width.
    let input = include_str!(
        "../../../tests/_fixtures/differential/formatter/sfc-prettier-empty-element-closing-width/App.vue.txt"
    );
    let expected = include_str!(
        "../../../tests/_fixtures/differential/formatter/sfc-prettier-empty-element-closing-width/reference.expected.txt"
    );
    assert_eq!(input, expected);
    assert_fixed_point(
        input,
        expected,
        &FormatOptions::from_preset(FormatPreset::Prettier),
    );
}

#[test]
fn empty_element_that_fits_with_its_closing_tag_stays_joined() {
    // Exactly print width with `></section>`: one column more would wrap.
    let options = FormatOptions::from_preset(FormatPreset::Prettier);
    let fits = "<template>\n  <section id=\"a\" class=\"a-long-class-name another-long-class-name-x\"></section>\n</template>\n";
    assert_eq!(fits.lines().nth(1).map(str::len), Some(80));
    assert_fixed_point(fits, fits, &options);
    let over = fits.replace("-x\"", "-xy\"");
    let wrapped = "<template>\n  <section\n    id=\"a\"\n    class=\"a-long-class-name another-long-class-name-xy\"\n  ></section>\n</template>\n";
    assert_fixed_point(&over, wrapped, &options);
}
