use super::{
    Delimiters, LexOptions,
    tests::{TestCallbacks, TokenEvent, lex_with},
};
use crate::markup::token::LexErrorCode;

fn tokenize(input: &str, open: &[u8], close: &[u8]) -> TestCallbacks {
    let options = LexOptions {
        delimiters: Delimiters { open, close },
        ..LexOptions::default()
    };
    lex_with(input, options)
}

#[test]
fn empty_opening_delimiter_treats_input_as_text() {
    let callbacks = tokenize("{{ msg }}", b"", b"}}");

    assert!(callbacks.errors.is_empty());
    assert!(callbacks.events.contains(&TokenEvent::Text(0, 9)));
    assert!(
        !callbacks
            .events
            .iter()
            .any(|event| matches!(event, TokenEvent::Interpolation(..)))
    );
}

#[test]
fn empty_closing_delimiter_reports_unfinished_interpolation() {
    let callbacks = tokenize("{{ msg }}", b"{{", b"");

    assert!(
        callbacks
            .errors
            .contains(&(LexErrorCode::MissingInterpolationEnd, 9))
    );
    assert!(callbacks.events.contains(&TokenEvent::Text(0, 9)));
    assert!(
        !callbacks
            .events
            .iter()
            .any(|event| matches!(event, TokenEvent::Interpolation(..)))
    );
}
