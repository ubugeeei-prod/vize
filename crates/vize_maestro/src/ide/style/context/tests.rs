#![expect(
    clippy::disallowed_macros,
    reason = "bounded CSS fixtures use std strings"
)]

use super::{CssContext, MAX_LOOKBACK, at};

fn marked(source: &str) -> (std::string::String, usize) {
    let offset = source.find('|').unwrap();
    (source.replacen('|', "", 1), offset)
}

#[test]
fn property_replaces_whole_token_and_keeps_an_existing_colon() {
    let (source, offset) = marked(".a {\r\n  back|ground-color /* comment */ : red }");
    let span = (
        source.find("background").unwrap(),
        source.find(" /*").unwrap(),
    );
    assert_eq!(
        at(&source, offset),
        CssContext::Property {
            prefix: "back",
            span,
            has_colon: true,
        }
    );
    let (source, offset) = marked(".a { col| }");
    assert_eq!(
        at(&source, offset),
        CssContext::Property {
            prefix: "col",
            span: (5, 8),
            has_colon: false,
        }
    );
}

#[test]
fn multiline_values_keep_the_owning_property_and_exact_span() {
    let (source, offset) = marked(".a {\n display /* : ; } */ :\r\n   in|line-flex; }");
    let start = source.find("inline-flex").unwrap();
    assert_eq!(
        at(&source, offset),
        CssContext::Value {
            property: "display",
            prefix: "in",
            span: (start, start + "inline-flex".len()),
        }
    );
    let (source, offset) = marked(".a { color: red\n display| }");
    assert!(matches!(
        at(&source, offset),
        CssContext::Value {
            property: "color",
            prefix: "display",
            ..
        }
    ));
}

#[test]
fn empty_insertion_points_belong_to_their_declaration() {
    let (source, offset) = marked(".a {\n  |}");
    assert_eq!(
        at(&source, offset),
        CssContext::Property {
            prefix: "",
            span: (offset, offset),
            has_colon: false,
        }
    );
    let (source, offset) = marked(".a { display: |}");
    assert_eq!(
        at(&source, offset),
        CssContext::Value {
            property: "display",
            prefix: "",
            span: (offset, offset),
        }
    );
}

#[test]
fn selector_pseudos_include_both_colons() {
    for (marked_source, prefix, token) in [
        (".a:hov|er { color: red }", ":hov", ":hover"),
        (".a::bef|ore {}", "::bef", "::before"),
        (".a { &:fo|cus {} }", ":fo", ":focus"),
        (".a { button:hov|er { color: red } }", ":hov", ":hover"),
        (".a :|", ":", ":"),
    ] {
        let (source, offset) = marked(marked_source);
        let start = source.find(token).unwrap();
        assert_eq!(
            at(&source, offset),
            CssContext::Pseudo {
                prefix,
                span: (start, start + token.len()),
            },
            "{marked_source}"
        );
    }
}

#[test]
fn at_rule_names_are_distinct_from_parameters_and_rule_list_bodies() {
    let (source, offset) = marked("@med|ia screen { .a { color: red } }");
    assert_eq!(
        at(&source, offset),
        CssContext::AtRule {
            prefix: "@med",
            span: (0, 6),
        }
    );
    for source in [
        "@media scr|een {}",
        "@media screen { col| }",
        "@supports (display: fl|ex) {}",
    ] {
        let (source, offset) = marked(source);
        assert_eq!(at(&source, offset), CssContext::Unknown);
    }
    let (source, offset) = marked("@media screen { @sup|ports (display: flex) {} }");
    assert!(matches!(
        at(&source, offset),
        CssContext::AtRule { prefix: "@sup", .. }
    ));
}

#[test]
fn nested_rules_restore_the_enclosing_declaration_context() {
    for source in [
        "@media screen { @supports (display: flex) { .a { dis|play } } }",
        ".a { &:hover { color: red; } dis|play }",
        "@font-face { font-f|amily: serif }",
        "@FONT-FACE { font-f|amily: serif }",
        "@keyframes pulse { from { op|acity: 0 } }",
    ] {
        let (source, offset) = marked(source);
        assert!(
            matches!(at(&source, offset), CssContext::Property { .. }),
            "{source}"
        );
    }
}

#[test]
fn comments_strings_and_function_arguments_do_not_offer_catalog_tokens() {
    for source in [
        ".a { /* color: re|d */ }",
        ".a { // dis|play\n color: red }",
        ".a { content: 'dis|play'; }",
        ".a { content: \"red\\\"; dis|play\"; }",
        ".a { background: url(re|d); }",
        ".a { color: var(--re|d); }",
        ".a { color: var(--x, re|d); }",
        ".a { width: calc(1px + in|herit); }",
    ] {
        let (source, offset) = marked(source);
        assert_eq!(at(&source, offset), CssContext::Unknown, "{source}");
    }
    let (source, offset) = marked(".a { /* : ; } */ // } :\n dis|play }");
    assert!(matches!(
        at(&source, offset),
        CssContext::Property { prefix: "dis", .. }
    ));
    let (source, offset) = marked(".a { width: calc(1px); dis|play }");
    assert!(matches!(
        at(&source, offset),
        CssContext::Property { prefix: "dis", .. }
    ));
}

#[test]
fn custom_properties_preprocessor_variables_and_hex_colors_fall_back() {
    for source in [
        ".a { --re|d: 1 }",
        ".a { |--red: 1 }",
        ".a { --x: re|d }",
        ".a { $re|d: 1 }",
        ".a { $x: re|d }",
        ".a { color: $re|d }",
        ".a { color: #re|d }",
        ".a { color: --re|d }",
        ".a { color: ére|d }",
        ".a { color: re|dé }",
        ".a { color: @re|d }",
        ".a { color::re|d }",
        ".a { dis|playé }",
        ".a { color: #{$re|d} }",
    ] {
        let (source, offset) = marked(source);
        assert_eq!(at(&source, offset), CssContext::Unknown, "{source}");
    }
}

#[test]
fn escaped_delimiters_do_not_begin_a_new_declaration() {
    for source in [
        ".a { color: red\\; dis|play }",
        ".a { co\\:lor: re|d }",
        ".a\\{ dis|play",
    ] {
        let (source, offset) = marked(source);
        assert_eq!(at(&source, offset), CssContext::Unknown, "{source}");
    }
    let (source, offset) = marked(".a { color: red\\;; dis|play }");
    assert!(matches!(
        at(&source, offset),
        CssContext::Property { prefix: "dis", .. }
    ));
}

#[test]
fn unicode_crlf_and_end_of_buffer_keep_relative_byte_ranges() {
    let (source, offset) = marked("/* 😀 */\r\n.日本 {\r\n  dis|play");
    let start = source.find("display").unwrap();
    assert_eq!(
        at(&source, offset),
        CssContext::Property {
            prefix: "dis",
            span: (start, start + 7),
            has_colon: false,
        }
    );
    let source = ".a { display";
    assert!(matches!(
        at(source, source.len()),
        CssContext::Property {
            prefix: "display",
            ..
        }
    ));
    assert_eq!(at("😀", 1), CssContext::Unknown);
    assert_eq!(at("a", 2), CssContext::Unknown);
}

#[test]
fn incomplete_or_unbalanced_lexical_regions_fail_conservatively() {
    for source in [
        ".a { /* |",
        ".a { content: '|",
        ".a { color: var(|",
        ".a } dis|",
        ".a { color: ) re|d",
        ".a { //|",
    ] {
        let (source, offset) = marked(source);
        assert_eq!(at(&source, offset), CssContext::Unknown, "{source}");
    }
    assert_eq!(at(".a { /", 6), CssContext::Unknown);
}

#[test]
fn prefix_and_replacement_budgets_are_explicit_not_clipped() {
    let mut source = std::string::String::from(".a {");
    source.push_str(&" ".repeat(MAX_LOOKBACK - source.len() - 3));
    source.push_str("dis");
    assert_eq!(source.len(), MAX_LOOKBACK);
    assert!(matches!(
        at(&source, source.len()),
        CssContext::Property { prefix: "dis", .. }
    ));
    source.push('p');
    assert_eq!(at(&source, source.len()), CssContext::Unknown);
    let source = format!(".a {{ {}", "d".repeat(1025));
    assert_eq!(at(&source, 5), CssContext::Unknown);
    let source = format!(".a {{ dis{}:", " ".repeat(1025));
    assert_eq!(at(&source, 8), CssContext::Unknown);
    let source = format!("{}dis", "a{".repeat(65));
    assert_eq!(at(&source, source.len()), CssContext::Unknown);
}

#[test]
fn every_utf8_boundary_produces_valid_borrowed_spans() {
    // Walk malformed and complete editor fragments, including cursor positions
    // between escape/comment delimiter bytes. The classifier must never panic
    // or produce a replacement outside its borrowed content/token boundary.
    for source in [
        ".a { /",
        ".a { //",
        ".a { /*",
        ".a { /*a*/color: \"a\\\"b\"; }",
        "@media (x:y) { .😀:hover { color: var(--x); display:block } }",
        ".a { &:hover { color: red } --x: #{$y}; /* } */ color: éred; }",
        ".a { color: red\\; display: block; content: '😀\\'😀'; }",
    ] {
        for offset in (0..=source.len()).filter(|&i| source.is_char_boundary(i)) {
            let result = at(source, offset);
            let (prefix, (start, end)) = match result {
                CssContext::Property { prefix, span, .. }
                | CssContext::Value { prefix, span, .. }
                | CssContext::Pseudo { prefix, span }
                | CssContext::AtRule { prefix, span } => (prefix, span),
                CssContext::Unknown => continue,
            };
            assert!(start <= offset && offset <= end && end <= source.len());
            assert_eq!(source.get(start..offset), Some(prefix));
            assert!(source.get(start..end).is_some());
            assert_eq!(Some(prefix.as_ptr()), source.get(start..).map(str::as_ptr));
        }
    }
}

#[test]
fn urls_keep_literal_slashes_and_resume_the_next_declaration() {
    for source in [
        ".a { background: url(https://example.com/a); col| }",
        ".a { background: url(https://example.com/*path); col| }",
        ".a { background: URL(https://example.com/a\\)b); col| }",
        ".a { background: url('https://example.com/a)b'); col| }",
        ".a { background: url(\"https://example.com/a\\\")b\"); col| }",
        ".a { background: image-set(url(https://example.com/a) 1x); col| }",
    ] {
        let (source, offset) = marked(source);
        assert!(
            matches!(
                at(&source, offset),
                CssContext::Property { prefix: "col", .. }
            ),
            "{source}"
        );
    }
    for source in [
        ".a { background: url(https://example.com/re|d); }",
        ".a { background: url(https://example.com/*re|d); }",
        ".a { background: url(https://example.com/a\\)| }",
        ".a { background: url('https://example.com/a)| }",
        ".a { background: url(https://example.com/a); /* col| */ }",
        ".a { background: url(https://example.com/a); // col|\n }",
    ] {
        let (source, offset) = marked(source);
        assert_eq!(at(&source, offset), CssContext::Unknown, "{source}");
    }
}
