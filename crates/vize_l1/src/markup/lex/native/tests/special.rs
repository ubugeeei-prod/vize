use super::{TokenEvent, tokenize};

// Special tag tests.

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
fn test_self_closing_textarea_returns_to_text() {
    let cb = tokenize("<textarea /><span>ok</span>");
    assert!(cb.errors.is_empty());
    assert!(cb.events.contains(&TokenEvent::OpenTagName(1, 9)));
    assert!(cb.events.contains(&TokenEvent::SelfClosingTag(11)));
    assert!(cb.events.contains(&TokenEvent::OpenTagName(13, 17)));
    assert!(cb.events.contains(&TokenEvent::Text(18, 20)));
    assert!(cb.events.contains(&TokenEvent::CloseTag(22, 26)));
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
    assert!(
        cb.events
            .iter()
            .any(|e| matches!(e, TokenEvent::CloseTag(_, _)))
    );
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
    assert!(
        !cb.events
            .iter()
            .any(|e| matches!(e, TokenEvent::Interpolation(_, _)))
    );
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
