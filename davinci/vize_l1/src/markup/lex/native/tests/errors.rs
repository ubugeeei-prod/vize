use super::{LexErrorCode, TokenEvent, tokenize};

// Comment and malformed-input tests.

#[test]
fn test_comment() {
    let cb = tokenize("<!-- comment -->");
    assert_eq!(
        cb.events.as_slice(),
        &[TokenEvent::Comment(4, 13), TokenEvent::End]
    );
}

#[test]
fn input_stream_characters_are_parse_errors() {
    let control = tokenize("<p>a\u{0001}b</p>");
    assert_eq!(
        control.errors.as_slice(),
        &[(LexErrorCode::ControlCharacterInInputStream, 4)]
    );
    let null = tokenize("<p>a\u{0000}b</p>");
    assert_eq!(
        null.errors.as_slice(),
        &[(LexErrorCode::UnexpectedNullCharacter, 4)]
    );
    let noncharacter = tokenize("<p>a\u{FFFE}b</p>");
    assert_eq!(
        noncharacter.errors.as_slice(),
        &[(LexErrorCode::NoncharacterInInputStream, 4)]
    );
    let clean = tokenize("<p>ab</p>");
    assert!(clean.errors.is_empty(), "{:?}", clean.errors);
    let text = tokenize("<p>あ</p>");
    assert!(text.errors.is_empty(), "{:?}", text.errors);
}

#[test]
fn test_comment_abrupt_empty_close_reports_error() {
    let cb = tokenize("<!-->");
    assert_eq!(
        cb.errors.as_slice(),
        &[(LexErrorCode::AbruptClosingOfEmptyComment, 4)]
    );
    assert_eq!(
        cb.events.as_slice(),
        &[TokenEvent::Comment(4, 4), TokenEvent::End]
    );
}

#[test]
fn test_comment_abrupt_empty_close_after_dash_reports_error() {
    let cb = tokenize("<!--->");
    assert_eq!(
        cb.errors.as_slice(),
        &[(LexErrorCode::AbruptClosingOfEmptyComment, 5)]
    );
    assert_eq!(
        cb.events.as_slice(),
        &[TokenEvent::Comment(4, 4), TokenEvent::End]
    );
}

// ========================================================================
// CDATA tests (SVG / XML-style `<![CDATA[ ... ]]>`)
// ========================================================================

#[test]
fn test_cdata_basic() {
    let cb = tokenize("<![CDATA[hi]]>");
    assert_eq!(
        cb.events.as_slice(),
        &[TokenEvent::Cdata(9, 11), TokenEvent::End]
    );
}

#[test]
fn test_cdata_with_angle_brackets() {
    let cb = tokenize("<![CDATA[<a>]]>");
    // content `<a>` is [9, 12)
    assert_eq!(
        cb.events.as_slice(),
        &[TokenEvent::Cdata(9, 12), TokenEvent::End]
    );
}

#[test]
fn test_cdata_empty() {
    let cb = tokenize("<![CDATA[]]>");
    assert_eq!(
        cb.events.as_slice(),
        &[TokenEvent::Cdata(9, 9), TokenEvent::End]
    );
}

#[test]
fn test_cdata_then_text() {
    let cb = tokenize("<![CDATA[x]]>after");
    assert_eq!(
        cb.events.as_slice(),
        &[
            TokenEvent::Cdata(9, 10),
            TokenEvent::Text(13, 18),
            TokenEvent::End
        ]
    );
}

#[test]
fn test_cdata_then_comment() {
    let cb = tokenize("<![CDATA[x]]><!-- comment -->");
    assert_eq!(
        cb.events.as_slice(),
        &[
            TokenEvent::Cdata(9, 10),
            TokenEvent::Comment(17, 26),
            TokenEvent::End
        ]
    );
}

#[test]
fn test_cdata_partial_close_resets_then_finds_close() {
    let cb = tokenize("<![CDATA[x]y]]>");
    assert_eq!(
        cb.events.as_slice(),
        &[TokenEvent::Cdata(9, 12), TokenEvent::End]
    );
}

#[test]
fn test_cdata_extra_bracket_before_close() {
    let cb = tokenize("<![CDATA[a]]]>");
    assert_eq!(
        cb.events.as_slice(),
        &[TokenEvent::Cdata(9, 11), TokenEvent::End]
    );
}

#[test]
fn test_comment_extra_hyphens_before_close() {
    let cb = tokenize("<!-- z ---->");
    assert_eq!(
        cb.events.as_slice(),
        &[TokenEvent::Comment(4, 9), TokenEvent::End]
    );
}

#[test]
fn test_comment_nested_opener_reports_error_and_closes_at_first_end() {
    let cb = tokenize("<!-- <!-- nested --> -->");
    assert!(
        cb.errors
            .iter()
            .any(|(code, _)| *code == LexErrorCode::NestedComment)
    );
    assert_eq!(
        cb.events.as_slice(),
        &[
            TokenEvent::Comment(4, 17),
            TokenEvent::Text(20, 24),
            TokenEvent::End
        ]
    );
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
    assert_eq!(
        cb.events.as_slice(),
        &[TokenEvent::Comment(4, 4), TokenEvent::End]
    );
}

#[test]
fn test_error_eof_in_empty_cdata() {
    let cb = tokenize("<![CDATA[");
    assert!(
        cb.errors
            .iter()
            .any(|(code, _)| *code == LexErrorCode::EofInCdata)
    );
    assert_eq!(
        cb.events.as_slice(),
        &[TokenEvent::Cdata(9, 9), TokenEvent::End]
    );
}

#[test]
fn test_error_processing_instruction_reports_question_mark() {
    let cb = tokenize(r#"<?xml version="1.0"?><div></div>"#);
    assert!(
        cb.errors
            .iter()
            .any(|(code, _)| *code == LexErrorCode::UnexpectedQuestionMarkInsteadOfTagName)
    );
    assert_eq!(
        cb.events.as_slice(),
        &[
            TokenEvent::OpenTagName(22, 25),
            TokenEvent::OpenTagEnd(25),
            TokenEvent::CloseTag(28, 31),
            TokenEvent::End,
        ]
    );
}

#[test]
fn test_error_unexpected_solidus_before_attribute() {
    let cb = tokenize("<div / id=foo></div>");
    assert!(
        cb.errors
            .iter()
            .any(|(code, _)| *code == LexErrorCode::UnexpectedSolidusInTag)
    );
    assert_eq!(
        cb.events.as_slice(),
        &[
            TokenEvent::OpenTagName(1, 4),
            TokenEvent::AttribName(7, 9),
            TokenEvent::AttribData(10, 13),
            TokenEvent::AttribEnd(crate::markup::token::QuoteType::Unquoted, 13),
            TokenEvent::OpenTagEnd(13),
            TokenEvent::CloseTag(16, 19),
            TokenEvent::End,
        ]
    );
}
