#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "tests assert admission boundaries by panicking"
)]

use core::ptr;

use vize_s0::{Allocator, Span};
use vize_s2::expr::{ExprRef, JsExpr, OpaqueReason};

use super::ExpressionCache;

fn admitted<'a>(expression: ExprRef<'a>) -> &'a JsExpr<'a> {
    match expression {
        ExprRef::Js(js) => js,
        _ => panic!("expected an admitted JS expression"),
    }
}

#[test]
fn equal_bytes_keep_distinct_authored_wrappers() {
    let allocator = Allocator::new();
    let cache = ExpressionCache::new();
    let authored = "item.value | item.value";
    let (first_source, second_source) = authored.split_once(" | ").expect("two positions");
    let first = admitted(cache.parse(&allocator, first_source, Span::new(0, 10)));
    let second = admitted(cache.parse(&allocator, second_source, Span::new(13, 23)));
    assert!(ptr::eq(first.ast, second.ast));
    assert!(!ptr::eq(first, second));
    assert!(ptr::eq(second.source.as_ptr(), second_source.as_ptr()));
    assert_eq!(first.span, Span::new(0, 10));
    assert_eq!(second.span, Span::new(13, 23));
}

#[test]
fn exact_text_identity_includes_typescript_and_trailing_trivia() {
    let allocator = Allocator::new();
    let cache = ExpressionCache::new();
    let first = admitted(cache.parse(&allocator, "value!", Span::new(0, 6)));
    let padded = admitted(cache.parse(&allocator, "value! /* note */", Span::new(10, 27)));
    let repeated = admitted(cache.parse(&allocator, "value!", Span::new(30, 36)));
    assert!(!ptr::eq(first.ast, padded.ast));
    assert!(ptr::eq(first.ast, repeated.ast));
    assert_eq!(padded.source, "value! /* note */");
    assert_eq!(repeated.span, Span::new(30, 36));
}

#[test]
fn eviction_reenters_the_canonical_parser() {
    let allocator = Allocator::new();
    let cache = ExpressionCache::new();
    let first = admitted(cache.parse(&allocator, "a", Span::new(0, 1)));
    for source in ["b", "c", "d", "e"] {
        admitted(cache.parse(&allocator, source, Span::new(0, 1)));
    }
    let repeated = admitted(cache.parse(&allocator, "a", Span::new(20, 21)));
    assert!(!ptr::eq(first.ast, repeated.ast));
    assert_eq!(repeated.source, "a");
    assert_eq!(repeated.span, Span::new(20, 21));
}

#[test]
fn rejected_text_is_never_admitted_from_a_successful_prefix() {
    let allocator = Allocator::new();
    let cache = ExpressionCache::new();
    admitted(cache.parse(&allocator, "value", Span::new(0, 5)));
    for source in ["value; next()", "value // unfinished", "%"] {
        for span in [Span::new(0, 1), Span::new(40, 41)] {
            match cache.parse(&allocator, source, span) {
                ExprRef::Opaque(opaque) => {
                    assert_eq!(opaque.reason, OpaqueReason::ParseRejected);
                    assert_eq!(opaque.source, source);
                    assert_eq!(opaque.span, span);
                }
                _ => panic!("incomplete text was admitted: {source}"),
            }
        }
    }
}
