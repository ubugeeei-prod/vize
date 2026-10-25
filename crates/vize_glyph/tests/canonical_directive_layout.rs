//! Whole directive packets reach one layout regardless of authored quote trivia.
#![expect(clippy::disallowed_types, reason = "owned JSON fixture transport")]
use serde::Deserialize;
use vize_glyph::{ArrowParens, EndOfLine, FormatOptions, TrailingComma, format_sfc};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    id: std::string::String,
    input: std::string::String,
    expected: std::string::String,
    print_width: u32,
}

#[test]
fn directive_layout_uses_ast_and_final_indentation() -> Result<(), Box<dyn std::error::Error>> {
    let cases: Vec<Case> =
        serde_json::from_str(include_str!("fixtures/preset-layout-10609/directives.json"))?;
    for case in cases {
        for (ending, newline) in [(EndOfLine::Lf, "\n"), (EndOfLine::Crlf, "\r\n")] {
            let mut options = FormatOptions::prettier_compat();
            options.print_width = case.print_width;
            options.trailing_comma = TrailingComma::None;
            options.arrow_parens = ArrowParens::Avoid;
            options.end_of_line = ending;
            let mut previous = case.input.replace('\n', newline);
            let expected = case.expected.replace('\n', newline);
            for pass in 0..3 {
                let result = format_sfc(&previous, &options)?;
                assert_eq!(result.code.as_str(), expected, "{} pass {pass}", case.id);
                assert_eq!(
                    result.changed,
                    previous != expected,
                    "{} pass {pass}",
                    case.id
                );
                previous = result.code.to_string();
            }
        }
    }
    Ok(())
}

#[test]
fn short_directives_collapse_quote_layout_at_large_width() -> Result<(), Box<dyn std::error::Error>>
{
    let options = FormatOptions {
        print_width: 10_000,
        ..FormatOptions::prettier_compat()
    };
    for value in [
        "visible ? 'yes' : 'no'",
        "visible && unread > 0",
        "{ active: visible, disabled: !visible }",
    ] {
        let expected = format!("<template>\n  <div :title=\"{value}\" />\n</template>\n");
        for source in [
            expected.clone(),
            format!("<template>\n  <div :title=\"\n    {value}\n  \" />\n</template>\n"),
        ] {
            assert_eq!(format_sfc(&source, &options)?.code.as_str(), expected);
        }
    }
    Ok(())
}
