use super::super::types::{is_end_of_tag_section, is_tag_start_char, is_whitespace};
use super::{QuoteType, TokenEvent, tokenize};

#[test]
fn test_is_tag_start_char() {
    assert!(is_tag_start_char(b'a'));
    assert!(is_tag_start_char(b'z'));
    assert!(is_tag_start_char(b'A'));
    assert!(is_tag_start_char(b'Z'));
    assert!(!is_tag_start_char(b'0'));
    assert!(!is_tag_start_char(b' '));
    assert!(!is_tag_start_char(b'<'));
    assert!(!is_tag_start_char(b'-'));
}

#[test]
fn test_is_whitespace() {
    assert!(is_whitespace(b' '));
    assert!(is_whitespace(b'\n'));
    assert!(is_whitespace(b'\t'));
    assert!(is_whitespace(b'\r'));
    assert!(is_whitespace(0x0C)); // form feed
    assert!(!is_whitespace(b'a'));
    assert!(!is_whitespace(b'<'));
}

#[test]
fn test_is_end_of_tag_section() {
    assert!(is_end_of_tag_section(b'/'));
    assert!(is_end_of_tag_section(b'>'));
    assert!(is_end_of_tag_section(b' '));
    assert!(is_end_of_tag_section(b'\n'));
    assert!(!is_end_of_tag_section(b'a'));
    assert!(!is_end_of_tag_section(b'"'));
}

// ========================================================================
// Basic text tests
// ========================================================================

#[test]
fn test_text() {
    let cb = tokenize("hello");
    assert!(cb.events.contains(&TokenEvent::Text(0, 5)));
    assert!(cb.events.contains(&TokenEvent::End));
}

#[test]
fn test_less_than_before_non_tag_start_is_literal_text() {
    let cb = tokenize("a < b");

    assert!(cb.errors.is_empty(), "{:?}", cb.errors);
    assert!(cb.events.contains(&TokenEvent::Text(0, 2)));
    assert!(cb.events.contains(&TokenEvent::Text(2, 5)));
}

#[test]
fn test_less_than_digit_is_literal_text() {
    let cb = tokenize("<1div>");

    assert!(cb.errors.is_empty(), "{:?}", cb.errors);
    assert!(cb.events.contains(&TokenEvent::Text(0, 6)));
}

// ========================================================================
// Element tests
// ========================================================================

#[test]
fn test_element() {
    let cb = tokenize("<div></div>");
    assert!(cb.events.contains(&TokenEvent::OpenTagName(1, 4)));
    assert!(cb.events.contains(&TokenEvent::OpenTagEnd(4)));
    assert!(cb.events.contains(&TokenEvent::CloseTag(7, 10)));
}

#[test]
fn test_self_closing() {
    let cb = tokenize("<br />");
    assert!(cb.events.contains(&TokenEvent::OpenTagName(1, 3)));
    assert!(cb.events.contains(&TokenEvent::SelfClosingTag(5)));
}

// ========================================================================
// Interpolation tests
// ========================================================================

#[test]
fn test_interpolation() {
    let cb = tokenize("{{ msg }}");
    assert!(cb.events.contains(&TokenEvent::Interpolation(2, 7)));
}

#[test]
fn test_text_and_interpolation() {
    let cb = tokenize("hello {{ name }} world");
    assert!(cb.events.contains(&TokenEvent::Text(0, 6)));
    assert!(cb.events.contains(&TokenEvent::Interpolation(8, 14)));
    assert!(cb.events.contains(&TokenEvent::Text(16, 22)));
}

// ========================================================================
// Attribute tests
// ========================================================================

#[test]
fn test_attribute_double_quote() {
    let cb = tokenize(r#"<div id="foo">"#);
    assert!(cb.events.contains(&TokenEvent::AttribName(5, 7)));
    assert!(cb.events.contains(&TokenEvent::AttribData(9, 12)));
    assert!(
        cb.events
            .contains(&TokenEvent::AttribEnd(QuoteType::Double, 12))
    );
}

#[test]
fn test_attribute_single_quote() {
    let cb = tokenize("<div id='foo'>");
    assert!(cb.events.contains(&TokenEvent::AttribName(5, 7)));
    assert!(cb.events.contains(&TokenEvent::AttribData(9, 12)));
    assert!(
        cb.events
            .contains(&TokenEvent::AttribEnd(QuoteType::Single, 12))
    );
}

#[test]
fn test_attribute_unquoted() {
    let cb = tokenize("<div id=foo>");
    assert!(cb.events.contains(&TokenEvent::AttribName(5, 7)));
    assert!(cb.events.contains(&TokenEvent::AttribData(8, 11)));
    assert!(
        cb.events
            .contains(&TokenEvent::AttribEnd(QuoteType::Unquoted, 11))
    );
}

#[test]
fn test_attribute_unquoted_slash_is_value_char() {
    // Per HTML spec, only whitespace and `>` terminate an unquoted attribute
    // value. `/` is an ordinary value character. Regression for #959.
    let cb = tokenize("<a href=a/b/c>x</a>");
    assert!(cb.events.contains(&TokenEvent::AttribName(3, 7)));
    assert!(cb.events.contains(&TokenEvent::AttribData(8, 13)));
    assert!(
        cb.events
            .contains(&TokenEvent::AttribEnd(QuoteType::Unquoted, 13))
    );
}

#[test]
fn test_attribute_unquoted_slash_in_url_value() {
    let cb = tokenize("<a href=//cdn/x>x</a>");
    assert!(cb.events.contains(&TokenEvent::AttribData(8, 15)));
    assert!(
        cb.events
            .contains(&TokenEvent::AttribEnd(QuoteType::Unquoted, 15))
    );
}

#[test]
fn test_attribute_no_value() {
    let cb = tokenize("<input disabled>");
    assert!(cb.events.contains(&TokenEvent::AttribName(7, 15)));
    assert!(
        cb.events
            .contains(&TokenEvent::AttribEnd(QuoteType::NoValue, 15))
    );
}

// ========================================================================
// Directive tests
// ========================================================================

#[test]
fn test_directive_v_if() {
    let cb = tokenize(r#"<div v-if="ok">"#);
    assert!(cb.events.contains(&TokenEvent::DirName(5, 9)));
    assert!(cb.events.contains(&TokenEvent::AttribData(11, 13)));
}

#[test]
fn test_shorthand_bind() {
    let cb = tokenize(r#"<div :class="c">"#);
    assert!(cb.events.contains(&TokenEvent::DirName(5, 6)));
    assert!(cb.events.contains(&TokenEvent::DirArg(6, 11)));
}

#[test]
fn test_shorthand_on() {
    let cb = tokenize(r#"<div @click="h">"#);
    assert!(cb.events.contains(&TokenEvent::DirName(5, 6)));
    assert!(cb.events.contains(&TokenEvent::DirArg(6, 11)));
}

#[test]
fn test_modifier() {
    let cb = tokenize(r#"<div @click.stop="h">"#);
    assert!(cb.events.contains(&TokenEvent::DirName(5, 6)));
    assert!(cb.events.contains(&TokenEvent::DirArg(6, 11)));
    assert!(cb.events.contains(&TokenEvent::DirModifier(12, 16)));
}

#[test]
fn test_dynamic_arg() {
    let cb = tokenize(r#"<div v-bind:[attr]="v">"#);
    assert!(cb.events.contains(&TokenEvent::DirName(5, 11)));
    assert!(cb.events.contains(&TokenEvent::DirArg(13, 17)));
}
