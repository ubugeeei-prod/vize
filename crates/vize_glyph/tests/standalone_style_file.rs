//! A standalone CSS file is formatted as a plain-CSS `<style>` block is, and
//! ends with exactly one line terminator.
use vize_glyph::{EndOfLine, FormatOptions, format_sfc, format_style_file};

fn assert_fixed_point(input: &str, expected: &str, options: &FormatOptions) {
    let first = format_style_file(input, options);
    assert_eq!(first.as_str(), expected, "first pass");
    assert_eq!(
        format_style_file(&first, options).as_str(),
        expected,
        "second pass"
    );
}

#[test]
fn stylesheet_matches_the_style_block_body() {
    let css = ".a{color:red;margin:0 auto}\n\n\n.b > .c{transition:color .2s,background-color .2s}";
    let options = FormatOptions::default();
    let sfc = vize_l0::cstr!("<style>\n{css}\n</style>\n");
    let block = format_sfc(&sfc, &options).expect("style block");
    let body = block
        .code
        .strip_prefix("<style>\n")
        .and_then(|code| code.strip_suffix("</style>\n"))
        .expect("one style block");
    assert_fixed_point(css, body, &options);
}

#[test]
fn file_layout_edges() {
    let options = FormatOptions::default();
    assert_fixed_point("", "", &options);
    assert_fixed_point(" \n\n", "", &options);
    assert_fixed_point(
        ".a { color: red; }\n\n\n",
        ".a {\n  color: red;\n}\n",
        &options,
    );
    assert_fixed_point(
        "\u{feff}.a{color:red}",
        "\u{feff}.a {\n  color: red;\n}\n",
        &options,
    );
    let auto = FormatOptions {
        end_of_line: EndOfLine::Auto,
        ..FormatOptions::default()
    };
    assert_fixed_point(
        ".a {\r\n  color: red;\r\n}\r\n",
        ".a {\r\n  color: red;\r\n}\r\n",
        &auto,
    );
}

#[test]
fn css_the_engine_cannot_parse_keeps_its_authored_text() {
    let source = "\n.a {  color: red; } }\n\n";
    let options = FormatOptions::default();
    assert!(vize_glyph::format_style(source, &options).is_err());
    assert_fixed_point(source, ".a {  color: red; } }\n", &options);
}
