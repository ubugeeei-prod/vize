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
        for (ending, newline) in [
            (EndOfLine::Lf, "\n"),
            (EndOfLine::Crlf, "\r\n"),
            (EndOfLine::Cr, "\r"),
        ] {
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
fn cr_object_layout_keeps_literal_spaces_with_configured_separators()
-> Result<(), Box<dyn std::error::Error>> {
    let options = FormatOptions {
        end_of_line: EndOfLine::Cr,
        trailing_comma: TrailingComma::None,
        print_width: 80,
        ..FormatOptions::prettier_compat()
    };
    for raw in ["\n", "\r\n", "\r"] {
        let literal = format!("String.raw`first{raw}  exact ${{unread}} line{raw}last`");
        let source = format!(
            "<template>\r  <div :class=\"{{ [{literal}]: visible, active: visible }}\" />\r</template>\r"
        );
        let printed_literal = "String.raw`first\r  exact ${unread} line\rlast`";
        let expected = format!(
            "<template>\r  <div\r    :class=\"{{\r      [{printed_literal}]: visible,\r      active: visible\r    }}\"\r  />\r</template>\r"
        );
        let first = format_sfc(&source, &options)?;
        assert_eq!(first.code.as_str(), expected, "raw {raw:?}");
        let mut previous = first.code;
        for _ in 0..2 {
            let next = format_sfc(&previous, &options)?;
            assert!(!next.changed, "raw {raw:?}: {}", next.code);
            assert_eq!(next.code, previous, "raw {raw:?}");
            assert_eq!(next.code.as_str(), expected, "raw {raw:?}");
            previous = next.code;
        }
    }
    Ok(())
}

#[test]
fn cr_object_output_is_canonical_from_all_code_line_endings()
-> Result<(), Box<dyn std::error::Error>> {
    let cases: Vec<Case> =
        serde_json::from_str(include_str!("fixtures/preset-layout-10609/directives.json"))?;
    let options = FormatOptions {
        end_of_line: EndOfLine::Cr,
        trailing_comma: TrailingComma::None,
        print_width: 80,
        ..FormatOptions::prettier_compat()
    };
    for case in cases
        .into_iter()
        .filter(|case| case.id.starts_with("object-properties"))
    {
        let expected = case.expected.replace('\n', "\r");
        for source_newline in ["\n", "\r\n", "\r"] {
            let mut source = case.input.replace('\n', source_newline);
            for _ in 0..3 {
                let result = format_sfc(&source, &options)?;
                assert_eq!(
                    result.code.as_str(),
                    expected,
                    "{} {source_newline:?}",
                    case.id
                );
                source = result.code.to_string();
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
