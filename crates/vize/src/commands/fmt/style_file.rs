//! Standalone CSS files, formatted as a plain-CSS `<style>` block is.

use vize_glyph::{EndOfLine, FormatOptions};
use vize_l0::String;

/// Format a standalone stylesheet with the `<style>` block formatter.
///
/// CSS the engine cannot parse keeps its authored text, as it does inside a
/// Vue file. The output ends with one line terminator, and a leading
/// byte-order mark is kept.
pub(super) fn format_style_file(source: &str, options: &FormatOptions) -> String {
    if let Some(document) = source.strip_prefix('\u{feff}') {
        let formatted = format_style_file(document, options);
        let mut output = String::with_capacity(formatted.len() + '\u{feff}'.len_utf8());
        output.push('\u{feff}');
        output.push_str(formatted.as_str());
        return output;
    }
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return String::default();
    }
    let formatted = vize_glyph::format_style(source, options);
    let body = formatted.as_ref().map_or(trimmed, |formatted| {
        formatted.trim_end_matches(['\r', '\n'])
    });
    let newline = match options.end_of_line {
        EndOfLine::Auto => source_line_ending(source),
        _ => options.newline_string(),
    };
    let mut output = String::with_capacity(body.len() + newline.len());
    output.push_str(body);
    output.push_str(newline);
    output
}

/// The first line terminator in `source`, which `endOfLine: auto` follows.
fn source_line_ending(source: &str) -> &'static str {
    let bytes = source.as_bytes();
    match memchr::memchr2(b'\r', b'\n', bytes) {
        Some(index) if bytes.get(index) == Some(&b'\r') => {
            if bytes.get(index + 1) == Some(&b'\n') {
                "\r\n"
            } else {
                "\r"
            }
        }
        _ => "\n",
    }
}

#[cfg(test)]
mod tests {
    use super::format_style_file;
    use vize_glyph::{EndOfLine, FormatOptions, format_sfc};

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
        let css =
            ".a{color:red;margin:0 auto}\n\n\n.b > .c{transition:color .2s,background-color .2s}";
        let options = FormatOptions::default();
        let sfc = vize_l0::cstr!("<style>\n{css}\n</style>\n");
        let block = format_sfc(&sfc, &options).unwrap();
        let body = block
            .code
            .strip_prefix("<style>\n")
            .and_then(|code| code.strip_suffix("</style>\n"))
            .unwrap();
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
}
