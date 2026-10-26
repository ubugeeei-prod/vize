//! Authored layouts Prettier 3.8.3 preserves: expanded object literals in
//! bindings and inline nodes on their own lines. Expected outputs are
//! Prettier's own. (#10771)
#![expect(clippy::disallowed_types, reason = "owned JSON fixture transport")]
use serde::Deserialize;
use vize_glyph::{EndOfLine, FormatOptions, format_sfc};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    id: std::string::String,
    input: std::string::String,
    expected: std::string::String,
    print_width: u32,
}

#[test]
fn prettier_preset_keeps_authored_layouts_prettier_keeps() -> Result<(), Box<dyn std::error::Error>>
{
    let cases: Vec<Case> =
        serde_json::from_str(include_str!("fixtures/prettier-layout-10771.json"))?;
    for case in cases {
        for (ending, newline) in [
            (EndOfLine::Lf, "\n"),
            (EndOfLine::Crlf, "\r\n"),
            (EndOfLine::Cr, "\r"),
        ] {
            let mut options = FormatOptions::prettier_compat();
            options.print_width = case.print_width;
            options.end_of_line = ending;
            let expected = case.expected.replace('\n', newline);
            let first = format_sfc(&case.input.replace('\n', newline), &options)?;
            assert_eq!(first.code.as_str(), expected, "{} {ending:?}", case.id);
            let second = format_sfc(&first.code, &options)?;
            assert!(!second.changed, "{} {ending:?}: second pass", case.id);
        }
    }
    Ok(())
}
