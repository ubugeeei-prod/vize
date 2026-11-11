use vize_l0::{Allocator, ErrorCode};

use super::parse_with_authored_diagnostics;

#[test]
fn input_stream_characters_keep_a_recovered_surface_error() {
    assert_input(
        "<p>a\u{0001}b</p>",
        ErrorCode::ControlCharacterInInputStream,
        4,
    );
    assert_input("<p>a\u{0000}b</p>", ErrorCode::UnexpectedNullCharacter, 4);
    assert_input("<p>a\u{FFFE}b</p>", ErrorCode::NoncharacterInInputStream, 4);
    assert_eq!(surface_codes("<p>ab</p>"), 0);
    assert_eq!(surface_codes("<p>あ</p>"), 0);
}

fn assert_input(source: &str, code: ErrorCode, offset: u32) {
    let allocator = Allocator::new();
    let (_, _, errors) = parse_with_authored_diagnostics(&allocator, source);
    let error = errors
        .iter()
        .find(|error| error.code == code)
        .unwrap_or_else(|| panic!("{source:?} -> {errors:?}"));
    assert_eq!(error.offset, offset);
    assert_eq!(error.message(source).as_str(), code.message());
    assert!(error.has_documented_recovery());
}

fn surface_codes(source: &str) -> usize {
    let allocator = Allocator::new();
    let (_, _, errors) = parse_with_authored_diagnostics(&allocator, source);
    errors
        .iter()
        .filter(|error| {
            matches!(
                error.code,
                ErrorCode::UnexpectedNullCharacter
                    | ErrorCode::ControlCharacterInInputStream
                    | ErrorCode::NoncharacterInInputStream
            )
        })
        .count()
}
