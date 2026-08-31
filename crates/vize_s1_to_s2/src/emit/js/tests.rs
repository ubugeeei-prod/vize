use vize_s0::Span;
use vize_s2::expr::{ExprRef, JsExpr, OpaqueExpr, OpaqueReason};

use super::{convert_line_comments_to_block, js_expr_source, parse_rejected_raw_js};

fn rejected<'a>(allocator: &'a vize_s0::Allocator, source: &'a str) -> ExprRef<'a> {
    ExprRef::Opaque(allocator.alloc(OpaqueExpr {
        reason: OpaqueReason::ParseRejected,
        source,
        span: Span::new(0, source.len() as u32),
    }))
}

#[test]
fn rewrites_parsed_js_line_comments() {
    let allocator = vize_s0::Allocator::new();
    let source = "{ value, // note\nnext }";
    let js = JsExpr::parse_in(&allocator, source, Span::new(0, source.len() as u32))
        .expect("object expression with a line comment parses");

    assert_eq!(js_expr_source(js).as_str(), "{ value, /*  note */\nnext }");
}

#[test]
fn rewrites_line_comments_only_when_rewritten_source_is_js() {
    let allocator = vize_s0::Allocator::new();
    let raw = parse_rejected_raw_js(&rejected(&allocator, "ok // comment"), false)
        .expect("line comment can be rewritten");
    assert_eq!(raw.as_str(), "ok /*  comment */");
    assert!(parse_rejected_raw_js(&rejected(&allocator, "ok. // comment"), false).is_none());
}

#[test]
fn preserves_regex_strings_and_blocks_while_rewriting_line_comments() {
    assert_eq!(
        convert_line_comments_to_block("url.replace(/https?:\\/\\/[^/]+\\//, '//')"),
        "url.replace(/https?:\\/\\/[^/]+\\//, '//')"
    );
    assert_eq!(
        convert_line_comments_to_block("x /* // */ + y // */"),
        "x /* // */ + y /*  * / */"
    );
}
