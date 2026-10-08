use super::{CssContext, MAX_LOOKBACK, at, marked};

#[test]
fn clean_empty_declaration_prefixes_end_before_following_whitespace() {
    for source in [
        ".a { | }",
        ".a { color:red; | }",
        ".a {\r\n\t | \r\n}",
        ".a { /* closed */ | }",
        ".a { // closed\n | }",
    ] {
        let (source, offset) = marked(source);
        assert_eq!(
            at(&source, offset),
            CssContext::Property {
                prefix: "",
                span: (offset, offset),
                has_colon: false,
            },
            "{source}"
        );
    }
}

#[test]
fn empty_prefixes_keep_incomplete_lexical_and_actual_budget_fallbacks() {
    for source in [
        ".a { /| }",
        ".a { /*| }",
        ".a { //| }",
        ".a { content: '| }",
        ".a { content: \"| }",
        ".a { | /* incomplete",
    ] {
        let (source, offset) = marked(source);
        assert_eq!(at(&source, offset), CssContext::Unknown, "{source}");
    }
    let mut source = ".a {".to_owned();
    source.push_str(&" ".repeat(MAX_LOOKBACK - source.len()));
    source.push_str(" }");
    assert_eq!(
        at(&source, MAX_LOOKBACK),
        CssContext::Property {
            prefix: "",
            span: (MAX_LOOKBACK, MAX_LOOKBACK),
            has_colon: false,
        }
    );
    assert_eq!(at(&source, MAX_LOOKBACK + 1), CssContext::Unknown);
    let mut source = ".a { ".to_owned();
    let offset = source.len();
    source.push_str(&" ".repeat(1025));
    source.push('}');
    assert_eq!(at(&source, offset), CssContext::Unknown);
}
