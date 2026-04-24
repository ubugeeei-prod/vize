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
    Cdata(usize, usize),
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
    fn on_cdata(&mut self, start: usize, end: usize) {
        self.events.push(TokenEvent::Cdata(start, end));
    }
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

#[test]
fn test_error_eof_in_empty_comment() {
    let cb = tokenize("<!--");
    assert!(cb
        .errors
        .iter()
        .any(|(code, _)| *code == ErrorCode::EofInComment));
    assert!(cb.events.contains(&TokenEvent::Comment(4, 4)));
}

#[test]
fn test_error_eof_in_empty_cdata() {
    let cb = tokenize("<![CDATA[");
    assert!(cb
        .errors
        .iter()
        .any(|(code, _)| *code == ErrorCode::EofInCdata));
    assert!(cb.events.contains(&TokenEvent::Cdata(9, 9)));
}

// ========================================================================
// HTML entity tests
// ========================================================================

#[test]
fn test_entity_attr_single_quote_and_text() {
    let cb = tokenize("<div data='&amp;'>>&amp;</div>");
    assert!(cb.events.contains(&TokenEvent::AttribEntity('&', 11, 16)));
    assert!(cb.events.contains(&TokenEvent::TextEntity('&', 19, 24)));
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

// ========================================================================
// Special tags tests
// ========================================================================

#[test]
fn test_special_opening_script_text_and_close() {
    let cb = tokenize("<script>a</script>");
    assert!(cb.errors.is_empty());
    assert!(cb.events.contains(&TokenEvent::OpenTagName(1, 7)));
    assert!(cb.events.contains(&TokenEvent::OpenTagEnd(7)));
    assert!(cb.events.contains(&TokenEvent::Text(8, 9)));
    assert!(cb.events.contains(&TokenEvent::CloseTag(11, 17)));
    assert!(cb.events.contains(&TokenEvent::End));
}

#[test]
fn test_special_opening_script_close_tag_uppercase() {
    let cb = tokenize("<script>a</SCRIPT>");
    assert!(cb.errors.is_empty());
    assert!(cb.events.contains(&TokenEvent::Text(8, 9)));
    assert!(cb.events.contains(&TokenEvent::CloseTag(11, 17)));
}

#[test]
fn test_special_opening_style_and_close() {
    let cb = tokenize("<style>.a{}</style>");
    assert!(cb.errors.is_empty());
    assert!(cb.events.contains(&TokenEvent::OpenTagName(1, 6)));
    // `</style` is 7 bytes; `>` at 18, closing name starts at 13
    assert!(cb.events.contains(&TokenEvent::CloseTag(13, 18)));
}

#[test]
fn test_special_opening_textarea_text_and_close() {
    let cb = tokenize("<textarea>hi</textarea>");
    assert!(cb.errors.is_empty());
    assert!(cb.events.contains(&TokenEvent::Text(10, 12)));
    assert!(cb.events.contains(&TokenEvent::CloseTag(14, 22)));
}

#[test]
fn test_special_opening_title_entity_in_plain_text() {
    let cb = tokenize("<title>a&amp;b</title>");
    assert!(cb.errors.is_empty());
    assert!(cb.events.contains(&TokenEvent::Text(7, 8)));
    assert!(cb.events.contains(&TokenEvent::TextEntity('&', 8, 13)));
    assert!(cb.events.contains(&TokenEvent::Text(13, 14)));
    assert!(cb.events.contains(&TokenEvent::CloseTag(16, 21)));
}

#[test]
fn test_before_special_s_falls_back_to_in_tag_name() {
    let cb = tokenize("<sfoo x></sfoo>");
    assert!(cb.errors.is_empty());
    assert!(cb.events.contains(&TokenEvent::OpenTagName(1, 5)));
    assert!(cb
        .events
        .iter()
        .any(|e| matches!(e, TokenEvent::CloseTag(_, _))));
}

#[test]
fn test_before_special_t_falls_back_to_in_tag_name() {
    let cb = tokenize("<tfoo></tfoo>");
    assert!(cb.errors.is_empty());
    assert!(cb.events.contains(&TokenEvent::OpenTagName(1, 5)));
}

#[test]
fn test_opening_tag_span_past_before_special_s() {
    let cb = tokenize("<span>a</span>");
    assert!(cb.errors.is_empty());
    assert!(cb.events.contains(&TokenEvent::OpenTagName(1, 5)));
    assert!(cb.events.contains(&TokenEvent::Text(6, 7)));
    // `</span` is 6 bytes; `>` at 13
    assert!(cb.events.contains(&TokenEvent::CloseTag(9, 13)));
}

#[test]
fn test_closing_tag_name_scr_prefix() {
    let cb = tokenize("</scrfoo>");
    assert!(cb.errors.is_empty());
    assert!(cb.events.contains(&TokenEvent::CloseTag(2, 8)));
}

#[test]
fn test_textarea_with_embedded_element() {
    let cb = tokenize("<textarea><h1>hi</h1></textarea>");
    assert!(cb.errors.is_empty());
    assert!(cb.events.contains(&TokenEvent::OpenTagName(1, 9)));
    assert!(cb.events.contains(&TokenEvent::OpenTagEnd(9)));
    assert!(cb.events.contains(&TokenEvent::Text(10, 21)));
    assert!(cb.events.contains(&TokenEvent::CloseTag(23, 31)));
}

#[test]
fn test_script_with_less_than_sign() {
    let cb = tokenize("<script>if(a<b)</script>");
    assert!(cb.errors.is_empty());
    assert!(cb.events.contains(&TokenEvent::OpenTagName(1, 7)));
    assert!(cb.events.contains(&TokenEvent::OpenTagEnd(7)));
    assert!(cb.events.contains(&TokenEvent::Text(8, 15)));
    assert!(cb.events.contains(&TokenEvent::CloseTag(17, 23)));
}

#[test]
fn test_textarea_interpolation_returns_to_rcdata() {
    let cb = tokenize("<textarea>a{{x}}b</textarea>");
    assert!(cb.errors.is_empty());
    assert!(cb.events.contains(&TokenEvent::OpenTagName(1, 9)));
    assert!(cb.events.contains(&TokenEvent::OpenTagEnd(9)));
    assert!(cb.events.contains(&TokenEvent::Text(10, 11)));
    assert!(cb.events.contains(&TokenEvent::Interpolation(13, 14)));
    assert!(cb.events.contains(&TokenEvent::Text(16, 17)));
    assert!(cb.events.contains(&TokenEvent::CloseTag(19, 27)));
}

#[test]
fn test_textarea_partial_interpolation_open_falls_back_to_rcdata() {
    let cb = tokenize("<textarea>a{b</textarea>");
    assert!(cb.errors.is_empty());
    assert!(cb.events.contains(&TokenEvent::OpenTagName(1, 9)));
    assert!(cb.events.contains(&TokenEvent::OpenTagEnd(9)));
    assert!(cb.events.contains(&TokenEvent::Text(10, 13)));
    assert!(cb.events.contains(&TokenEvent::CloseTag(15, 23)));
    assert!(!cb
        .events
        .iter()
        .any(|e| matches!(e, TokenEvent::Interpolation(_, _))));
}

#[test]
fn test_scriptx_is_not_treated_as_special_script_tag() {
    let cb = tokenize("<scriptx>1</scriptx>");
    assert!(cb.errors.is_empty());
    assert!(cb.events.contains(&TokenEvent::OpenTagName(1, 8)));
    assert!(cb.events.contains(&TokenEvent::OpenTagEnd(8)));
    assert!(cb.events.contains(&TokenEvent::Text(9, 10)));
    assert!(cb.events.contains(&TokenEvent::CloseTag(12, 19)));
}

#[test]
fn test_titlex_is_not_treated_as_special_title_tag() {
    let cb = tokenize("<titlex>1</titlex>");
    assert!(cb.errors.is_empty());
    assert!(cb.events.contains(&TokenEvent::OpenTagName(1, 7)));
    assert!(cb.events.contains(&TokenEvent::OpenTagEnd(7)));
    assert!(cb.events.contains(&TokenEvent::Text(8, 9)));
    assert!(cb.events.contains(&TokenEvent::CloseTag(11, 17)));
}

#[test]
fn test_script_close_tag_with_whitespace_before_gt() {
    let cb = tokenize("<script>a</script   >");
    assert!(cb.errors.is_empty());
    assert!(cb.events.contains(&TokenEvent::OpenTagName(1, 7)));
    assert!(cb.events.contains(&TokenEvent::Text(8, 9)));
    assert!(cb.events.contains(&TokenEvent::CloseTag(11, 17)));
}

#[test]
fn test_script_pseudo_close_kept_as_text_until_real_close() {
    let cb = tokenize("<script>a</scriptx>b</script>");
    assert!(cb.errors.is_empty());
    assert!(cb.events.contains(&TokenEvent::OpenTagName(1, 7)));
    assert!(cb.events.contains(&TokenEvent::OpenTagEnd(7)));
    assert!(cb.events.contains(&TokenEvent::Text(8, 20)));
    assert!(cb.events.contains(&TokenEvent::CloseTag(22, 28)));
}
