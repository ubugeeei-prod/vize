use super::{
    types::{is_end_of_tag_section, is_tag_start_char, is_whitespace, Callbacks, QuoteType},
    Tokenizer,
};
use vize_relief::ErrorCode;

// ========================================================================
// Test callback infrastructure
// ========================================================================

#[derive(Debug, PartialEq)]
enum TokenEvent {
    Text(usize, usize),
    TextEntity(char, usize, usize),
    Interpolation(usize, usize),
    OpenTagName(usize, usize),
    OpenTagEnd(usize),
    SelfClosingTag(usize),
    CloseTag(usize, usize),
    AttribName(usize, usize),
    AttribData(usize, usize),
    AttribEnd(QuoteType, usize),
    AttribEntity(char, usize, usize),
    DirName(usize, usize),
    DirArg(usize, usize),
    DirModifier(usize, usize),
    Comment(usize, usize),
    End,
}

#[derive(Debug, Default)]
struct TestCallbacks {
    events: Vec<TokenEvent>,
    errors: Vec<(ErrorCode, usize)>,
}

impl Callbacks for TestCallbacks {
    fn on_text(&mut self, start: usize, end: usize) {
        self.events.push(TokenEvent::Text(start, end));
    }
    fn on_text_entity(&mut self, _char: char, start: usize, end: usize) {
        self.events.push(TokenEvent::TextEntity(_char, start, end));
    }
    fn on_interpolation(&mut self, start: usize, end: usize) {
        self.events.push(TokenEvent::Interpolation(start, end));
    }
    fn on_open_tag_name(&mut self, start: usize, end: usize) {
        self.events.push(TokenEvent::OpenTagName(start, end));
    }
    fn on_open_tag_end(&mut self, end: usize) {
        self.events.push(TokenEvent::OpenTagEnd(end));
    }
    fn on_self_closing_tag(&mut self, end: usize) {
        self.events.push(TokenEvent::SelfClosingTag(end));
    }
    fn on_close_tag(&mut self, start: usize, end: usize) {
        self.events.push(TokenEvent::CloseTag(start, end));
    }
    fn on_attrib_name(&mut self, start: usize, end: usize) {
        self.events.push(TokenEvent::AttribName(start, end));
    }
    fn on_attrib_name_end(&mut self, _end: usize) {}
    fn on_attrib_data(&mut self, start: usize, end: usize) {
        self.events.push(TokenEvent::AttribData(start, end));
    }
    fn on_attrib_entity(&mut self, ch: char, start: usize, end: usize) {
        self.events.push(TokenEvent::AttribEntity(ch, start, end));
    }
    fn on_attrib_end(&mut self, quote: QuoteType, end: usize) {
        self.events.push(TokenEvent::AttribEnd(quote, end));
    }
    fn on_dir_name(&mut self, start: usize, end: usize) {
        self.events.push(TokenEvent::DirName(start, end));
    }
    fn on_dir_arg(&mut self, start: usize, end: usize) {
        self.events.push(TokenEvent::DirArg(start, end));
    }
    fn on_dir_modifier(&mut self, start: usize, end: usize) {
        self.events.push(TokenEvent::DirModifier(start, end));
    }
    fn on_comment(&mut self, start: usize, end: usize) {
        self.events.push(TokenEvent::Comment(start, end));
    }
    fn on_cdata(&mut self, _start: usize, _end: usize) {}
    fn on_processing_instruction(&mut self, _start: usize, _end: usize) {}
    fn on_end(&mut self) {
        self.events.push(TokenEvent::End);
    }
    fn on_error(&mut self, code: ErrorCode, index: usize) {
        self.errors.push((code, index));
    }
}

fn tokenize(input: &str) -> TestCallbacks {
    let cb = TestCallbacks::default();
    let mut tok = Tokenizer::new(input, cb);
    tok.tokenize();
    tok.callbacks
}

// ========================================================================
// Utility function tests
// ========================================================================

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
// Position calculation tests
// ========================================================================

#[test]
fn test_get_pos_single_line() {
    let cb = TestCallbacks::default();
    let tok = Tokenizer::new("hello", cb);
    let pos = tok.get_pos(0);
    assert_eq!(pos.offset, 0);
    assert_eq!(pos.line, 1);
    assert_eq!(pos.column, 1);

    let pos = tok.get_pos(4);
    assert_eq!(pos.offset, 4);
    assert_eq!(pos.line, 1);
    assert_eq!(pos.column, 5);
}

#[test]
fn test_get_pos_multi_line() {
    let input = "line1\nline2\nline3";
    let cb = TestCallbacks::default();
    let mut tok = Tokenizer::new(input, cb);
    tok.tokenize();
    let pos = tok.get_pos(6);
    assert_eq!(pos.line, 2);
    assert_eq!(pos.column, 1);

    let pos = tok.get_pos(12);
    assert_eq!(pos.line, 3);
    assert_eq!(pos.column, 1);
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
    assert!(cb
        .events
        .contains(&TokenEvent::AttribEnd(QuoteType::Double, 12)));
}

#[test]
fn test_attribute_single_quote() {
    let cb = tokenize("<div id='foo'>");
    assert!(cb.events.contains(&TokenEvent::AttribName(5, 7)));
    assert!(cb.events.contains(&TokenEvent::AttribData(9, 12)));
    assert!(cb
        .events
        .contains(&TokenEvent::AttribEnd(QuoteType::Single, 12)));
}

#[test]
fn test_attribute_unquoted() {
    let cb = tokenize("<div id=foo>");
    assert!(cb.events.contains(&TokenEvent::AttribName(5, 7)));
    assert!(cb.events.contains(&TokenEvent::AttribData(8, 11)));
    assert!(cb
        .events
        .contains(&TokenEvent::AttribEnd(QuoteType::Unquoted, 11)));
}

#[test]
fn test_attribute_no_value() {
    let cb = tokenize("<input disabled>");
    assert!(cb.events.contains(&TokenEvent::AttribName(7, 15)));
    assert!(cb
        .events
        .contains(&TokenEvent::AttribEnd(QuoteType::NoValue, 15)));
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

// ========================================================================
// Comment tests
// ========================================================================

#[test]
fn test_comment() {
    let cb = tokenize("<!-- comment -->");
    assert!(cb.events.contains(&TokenEvent::Comment(4, 13)));
}

// ========================================================================
// Error tests
// ========================================================================

#[test]
fn test_error_eof_in_tag() {
    let cb = tokenize("<div");
    assert!(cb
        .errors
        .iter()
        .any(|(code, _)| *code == ErrorCode::EofInTag));
}

#[test]
fn test_error_eof_in_comment() {
    let cb = tokenize("<!-- unterminated");
    assert!(cb
        .errors
        .iter()
        .any(|(code, _)| *code == ErrorCode::EofInComment));
}

// ========================================================================
// HTML entity tests
// ========================================================================

#[test]
fn test_entity_attr_single_quote_and_text() {
    let cb = tokenize("<div data='&amp;'>>&amp;</div>");
    assert!(cb.events.contains(&TokenEvent::AttribEntity('&', 11, 16)));
    assert!(cb.events.contains(&TokenEvent::TextEntity('&', 19, 24)));
    println!("{:?}", cb.events);
}

#[test]
fn test_entity_in_double_quoted_attr_lt() {
    let cb = tokenize(r#"<div a="&lt;">"#);
    assert!(cb.events.contains(&TokenEvent::AttribEntity('<', 8, 12)));
    assert!(cb
        .events
        .contains(&TokenEvent::AttribEnd(QuoteType::Double, 12)));
}

#[test]
fn test_entity_in_double_quoted_attr_with_literal_suffix() {
    let cb = tokenize(r#"<div a="&amp;b">"#);
    assert!(cb.events.contains(&TokenEvent::AttribEntity('&', 8, 13)));
    assert!(cb.events.contains(&TokenEvent::AttribData(13, 14)));
    assert!(cb
        .events
        .contains(&TokenEvent::AttribEnd(QuoteType::Double, 14)));
}

#[test]
fn test_entity_in_single_quoted_attr() {
    let cb = tokenize("<div a='&#38;'>");
    assert!(cb.events.contains(&TokenEvent::AttribEntity('&', 8, 13)));
    assert!(cb
        .events
        .contains(&TokenEvent::AttribEnd(QuoteType::Single, 13)));
}

#[test]
fn test_entity_text_named_amp() {
    let cb = tokenize("a&amp;b");
    assert!(cb.events.contains(&TokenEvent::Text(0, 1)));
    assert!(cb.events.contains(&TokenEvent::TextEntity('&', 1, 6)));
    assert!(cb.events.contains(&TokenEvent::Text(6, 7)));
}

#[test]
fn test_entity_text_lt_semicolon() {
    let cb = tokenize("1&lt;2");
    assert!(cb.events.contains(&TokenEvent::Text(0, 1)));
    assert!(cb.events.contains(&TokenEvent::TextEntity('<', 1, 5)));
    assert!(cb.events.contains(&TokenEvent::Text(5, 6)));
}

#[test]
fn test_entity_text_numeric_dec() {
    let cb = tokenize("&#38;x");
    assert!(cb.events.contains(&TokenEvent::TextEntity('&', 0, 5)));
    assert!(cb.events.contains(&TokenEvent::Text(5, 6)));
}

#[test]
fn test_entity_double_ampersand_then_amp() {
    let cb = tokenize("&&amp;");
    assert!(cb.events.contains(&TokenEvent::Text(0, 1)));
    assert!(cb.events.contains(&TokenEvent::TextEntity('&', 1, 6)));
}

#[test]
fn test_entity_in_unquoted_attr_value() {
    let cb = tokenize("<div x=a&amp;b>");
    assert!(cb.events.contains(&TokenEvent::AttribName(5, 6)));
    assert!(cb.events.contains(&TokenEvent::AttribData(7, 8)));
    assert!(cb.events.contains(&TokenEvent::AttribEntity('&', 8, 13)));
    assert!(cb.events.contains(&TokenEvent::AttribData(13, 14)));
    assert!(cb
        .events
        .contains(&TokenEvent::AttribEnd(QuoteType::Unquoted, 14)));
}
