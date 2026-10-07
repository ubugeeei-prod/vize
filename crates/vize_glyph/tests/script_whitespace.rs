//! Script fast paths retain Unicode whitespace and parser error contracts.
#![expect(
    clippy::disallowed_macros,
    reason = "regressions assert complete public output and parser failures"
)]

use vize_glyph::{FormatOptions, format_script};

#[test]
fn every_unicode_whitespace_script_keeps_the_complete_empty_output() {
    let options = FormatOptions::default();
    for whitespace in [
        '\t', '\n', '\u{b}', '\u{c}', '\r', ' ', '\u{85}', '\u{a0}', '\u{1680}', '\u{2000}',
        '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}', '\u{2005}', '\u{2006}', '\u{2007}',
        '\u{2008}', '\u{2009}', '\u{200a}', '\u{2028}', '\u{2029}', '\u{202f}', '\u{205f}',
        '\u{3000}',
    ] {
        let source = format!("{whitespace} \t{whitespace}\r\n");
        assert_eq!(format_script(&source, &options).unwrap(), "");
    }
    assert_eq!(format_script("", &options).unwrap(), "");
}

#[test]
fn ascii_and_unicode_script_starts_keep_three_complete_fixed_points() {
    let options = FormatOptions::default();
    for (source, expected) in [
        ("const value=1", "const value = 1;\n"),
        ("\u{a0}const value=1", "const value = 1;\n"),
        ("\u{b}const value=1", "const value = 1;\n"),
        ("日本語=1", "日本語 = 1;\n"),
        ("const 日本語=1", "const 日本語 = 1;\n"),
    ] {
        let mut current = source.to_owned();
        for _ in 0..3 {
            current = format_script(&current, &options).unwrap().to_string();
            assert_eq!(current, expected);
        }
    }
}

#[test]
fn non_whitespace_control_scripts_keep_real_parser_errors() {
    let options = FormatOptions::default();
    for source in ["\0", "\u{1b}", "\u{7f}"] {
        assert!(format_script(source, &options).is_err());
    }
}
