use super::{LexErrorCode, TokenEvent, tokenize};

// Comment and malformed-input tests.

#[test]
fn test_comment() {
    let cb = tokenize("<!-- comment -->");
    assert!(cb.events.contains(&TokenEvent::Comment(4, 13)));
}

#[test]
fn input_stream_characters_are_parse_errors() {
    let control = tokenize("<p>a\u{0001}b</p>");
    assert!(
        control
            .errors
            .contains(&(LexErrorCode::ControlCharacterInInputStream, 4)),
        "{:?}",
        control.errors
    );
    let null = tokenize("<p>a\u{0000}b</p>");
    assert!(
        null.errors
            .contains(&(LexErrorCode::UnexpectedNullCharacter, 4)),
        "{:?}",
        null.errors
    );
    let noncharacter = tokenize("<p>a\u{FFFE}b</p>");
    let at = noncharacter
        .errors
        .iter()
        .find(|(code, _)| *code == LexErrorCode::NoncharacterInInputStream);
    assert_eq!(
        at.map(|(_, index)| *index),
        Some(4),
        "{:?}",
        noncharacter.errors
    );
    let clean = tokenize("<p>ab</p>");
    assert!(clean.errors.is_empty(), "{:?}", clean.errors);
    let text = tokenize("<p>あ</p>");
    assert!(text.errors.is_empty(), "{:?}", text.errors);
}

#[test]
fn test_comment_abrupt_empty_close_reports_error() {
    let cb = tokenize("<!-->");
    assert!(
        cb.errors
            .contains(&(LexErrorCode::AbruptClosingOfEmptyComment, 4))
    );
    assert!(cb.events.contains(&TokenEvent::Comment(4, 4)));
}

#[test]
fn test_comment_abrupt_empty_close_after_dash_reports_error() {
    let cb = tokenize("<!--->");
    assert!(
        cb.errors
            .contains(&(LexErrorCode::AbruptClosingOfEmptyComment, 5))
    );
    assert!(cb.events.contains(&TokenEvent::Comment(4, 4)));
}

// ========================================================================
// CDATA tests (SVG / XML-style `<![CDATA[ ... ]]>`)
// ========================================================================

#[test]
fn test_cdata_basic() {
    let cb = tokenize("<![CDATA[hi]]>");
    assert!(cb.events.contains(&TokenEvent::Cdata(9, 11)));
}

#[test]
fn test_cdata_with_angle_brackets() {
    let cb = tokenize("<![CDATA[<a>]]>");
    // content `<a>` is [9, 12)
    assert!(cb.events.contains(&TokenEvent::Cdata(9, 12)));
}

#[test]
fn test_cdata_empty() {
    let cb = tokenize("<![CDATA[]]>");
    assert!(cb.events.contains(&TokenEvent::Cdata(9, 9)));
}

#[test]
fn test_cdata_then_text() {
    let cb = tokenize("<![CDATA[x]]>after");
    assert!(cb.events.contains(&TokenEvent::Cdata(9, 10)));
    assert!(cb.events.contains(&TokenEvent::Text(13, 18)));
}

#[test]
fn test_cdata_then_comment() {
    let cb = tokenize("<![CDATA[x]]><!-- comment -->");
    assert!(cb.events.contains(&TokenEvent::Cdata(9, 10)));
}

#[test]
fn test_cdata_partial_close_resets_then_finds_close() {
    let cb = tokenize("<![CDATA[x]y]]>");
    assert!(cb.events.contains(&TokenEvent::Cdata(9, 12)));
}

#[test]
fn test_cdata_extra_bracket_before_close() {
    let cb = tokenize("<![CDATA[a]]]>");
    assert!(cb.events.contains(&TokenEvent::Cdata(9, 11)));
}

#[test]
fn test_comment_extra_hyphens_before_close() {
    let cb = tokenize("<!-- z ---->");
    assert!(cb.events.contains(&TokenEvent::Comment(4, 9)));
}

#[test]
fn test_comment_nested_opener_reports_error_and_closes_at_first_end() {
    let cb = tokenize("<!-- <!-- nested --> -->");
    assert!(
        cb.errors
            .iter()
            .any(|(code, _)| *code == LexErrorCode::NestedComment)
    );
    assert!(cb.events.contains(&TokenEvent::Comment(4, 17)));
    assert!(cb.events.contains(&TokenEvent::Text(20, 24)));
}

// ========================================================================
// Error tests
// ========================================================================

#[test]
fn test_error_eof_in_tag() {
    let cb = tokenize("<div");
    assert!(
        cb.errors
            .iter()
            .any(|(code, _)| *code == LexErrorCode::EofInTag)
    );
}

#[test]
fn test_error_eof_in_comment() {
    let cb = tokenize("<!-- unterminated");
    assert!(
        cb.errors
            .iter()
            .any(|(code, _)| *code == LexErrorCode::EofInComment)
    );
}

#[test]
fn test_error_eof_in_empty_comment() {
    let cb = tokenize("<!--");
    assert!(
        cb.errors
            .iter()
            .any(|(code, _)| *code == LexErrorCode::EofInComment)
    );
    assert!(cb.events.contains(&TokenEvent::Comment(4, 4)));
}

#[test]
fn test_error_eof_in_empty_cdata() {
    let cb = tokenize("<![CDATA[");
    assert!(
        cb.errors
            .iter()
            .any(|(code, _)| *code == LexErrorCode::EofInCdata)
    );
    assert!(cb.events.contains(&TokenEvent::Cdata(9, 9)));
}

#[test]
fn test_error_processing_instruction_reports_question_mark() {
    let cb = tokenize(r#"<?xml version="1.0"?><div></div>"#);
    assert!(
        cb.errors
            .iter()
            .any(|(code, _)| *code == LexErrorCode::UnexpectedQuestionMarkInsteadOfTagName)
    );
    assert!(cb.events.contains(&TokenEvent::OpenTagName(22, 25)));
}

#[test]
fn test_error_unexpected_solidus_before_attribute() {
    let cb = tokenize("<div / id=foo></div>");
    assert!(
        cb.errors
            .iter()
            .any(|(code, _)| *code == LexErrorCode::UnexpectedSolidusInTag)
    );
    assert!(cb.events.contains(&TokenEvent::AttribName(7, 9)));
    assert!(cb.events.contains(&TokenEvent::AttribData(10, 13)));
}
