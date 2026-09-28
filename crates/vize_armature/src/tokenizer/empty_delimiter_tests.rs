use super::{
    Tokenizer,
    tests::{TestCallbacks, TokenEvent},
};
use vize_relief::ErrorCode;

fn tokenize(input: &str, open: &[u8], close: &[u8]) -> TestCallbacks {
    let callbacks = TestCallbacks::default();
    let mut tokenizer = Tokenizer::with_delimiters(input, callbacks, open, close);
    tokenizer.tokenize();
    tokenizer.callbacks
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
            .contains(&(ErrorCode::MissingInterpolationEnd, 9))
    );
    assert!(callbacks.events.contains(&TokenEvent::Text(0, 9)));
    assert!(
        !callbacks
            .events
            .iter()
            .any(|event| matches!(event, TokenEvent::Interpolation(..)))
    );
}
