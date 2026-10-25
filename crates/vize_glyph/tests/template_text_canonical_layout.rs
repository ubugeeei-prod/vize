//! Complete consumer packets and semantics-sensitive text boundaries.
#![expect(clippy::disallowed_types, reason = "owned JSON fixture transport")]
use serde::Deserialize;
use unicode_width::UnicodeWidthStr;
use vize_glyph::{ArrowParens, EndOfLine, FormatOptions, TrailingComma, format_sfc};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    id: std::string::String,
    input: std::string::String,
    expected: Option<std::string::String>,
    print_width: u32,
}

#[test]
fn text_wraps_only_at_existing_whitespace_and_reaches_one_fixed_point()
-> Result<(), Box<dyn std::error::Error>> {
    let cases: Vec<Case> =
        serde_json::from_str(include_str!("fixtures/preset-layout-10609/text.json"))?;
    let mut deep_layouts = std::collections::BTreeMap::new();
    for case in cases {
        for (ending, newline) in [(EndOfLine::Lf, "\n"), (EndOfLine::Crlf, "\r\n")] {
            let mut options = FormatOptions::prettier_compat();
            options.print_width = case.print_width;
            options.trailing_comma = TrailingComma::None;
            options.arrow_parens = ArrowParens::Avoid;
            options.end_of_line = ending;
            let source = case.input.replace('\n', newline);
            let first = format_sfc(&source, &options)?;
            if case.id.starts_with("deep-plain") {
                if let Some(previous) = deep_layouts.get(newline) {
                    assert_eq!(&first.code, previous, "deep text input-layout canonicality");
                } else {
                    deep_layouts.insert(newline, first.code.clone());
                }
            }
            if let Some(expected) = &case.expected {
                assert_eq!(
                    first.code.as_str(),
                    expected.replace('\n', newline),
                    "{}",
                    case.id
                );
            }
            assert!(
                first
                    .code
                    .lines()
                    .all(|line| line.width() <= case.print_width as usize),
                "{}: width\n{}",
                case.id,
                first.code
            );
            let second = format_sfc(&first.code, &options)?;
            assert!(!second.changed, "{}: second pass", case.id);
            assert_eq!(first.code, second.code, "{}: second pass", case.id);
            assert_eq!(
                second.code,
                format_sfc(&second.code, &options)?.code,
                "{}: third pass",
                case.id
            );
        }
    }
    Ok(())
}

#[test]
fn empty_sfc_stays_empty() -> Result<(), Box<dyn std::error::Error>> {
    for options in [FormatOptions::default(), FormatOptions::prettier_compat()] {
        let result = format_sfc("", &options)?;
        assert!(!result.changed);
        assert!(result.code.is_empty());
    }
    Ok(())
}
