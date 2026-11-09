use super::{QuoteType, TokenEvent, tokenize};

// HTML entity tests.

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
    assert!(
        cb.events
            .contains(&TokenEvent::AttribEnd(QuoteType::Double, 12))
    );
}

#[test]
fn test_entity_in_double_quoted_attr_with_literal_suffix() {
    let cb = tokenize(r#"<div a="&amp;b">"#);
    assert!(cb.events.contains(&TokenEvent::AttribEntity('&', 8, 13)));
    assert!(cb.events.contains(&TokenEvent::AttribData(13, 14)));
    assert!(
        cb.events
            .contains(&TokenEvent::AttribEnd(QuoteType::Double, 14))
    );
}

#[test]
fn test_entity_in_single_quoted_attr() {
    let cb = tokenize("<div a='&#38;'>");
    assert!(cb.events.contains(&TokenEvent::AttribEntity('&', 8, 13)));
    assert!(
        cb.events
            .contains(&TokenEvent::AttribEnd(QuoteType::Single, 13))
    );
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
    assert!(
        cb.events
            .contains(&TokenEvent::AttribEnd(QuoteType::Unquoted, 14))
    );
}
