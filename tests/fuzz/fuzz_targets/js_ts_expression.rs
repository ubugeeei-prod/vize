#![no_main]

// JS/TS expression parser fuzz target.
//
// Drives the same OXC expression parsing path used by template expression
// transforms and import-usage checks. Invalid expressions are expected and
// reported as parser errors; panics are not.
//
// One upstream crash class is skip-listed until the pinned OXC revision carries
// a fix (#3296; retirement tracked in #3307): a TS tuple type with an optional
// member, whose next element consumes no tokens, builds that element's span as
// `Span::new(start_span(), prev_token_end)`. `start_span()` has already skipped
// the trivia after the comma, so the span comes out inverted (`start > end`),
// and labeling it for `TS1257 required element cannot follow an optional
// element` trips `debug_assert!(self.start <= self.end)` in
// `oxc_span::Span::size`.
//
// The shape is neither a depth nor a balance property, so the expression
// nesting guard cannot express it; the skip costs fuzz coverage only. The
// predicate lives in `upstream_span_assertion_skip.rs` and its boundary matrix
// is pinned by `crates/vize_atelier_core/tests/upstream_tuple_type_span_assertion.rs`.
use libfuzzer_sys::fuzz_target;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use upstream_span_assertion_skip::hits_known_upstream_span_assertion_shape;
use vize_atelier_core::steps::expression::expression_is_safe_to_parse;

#[path = "upstream_span_assertion_skip.rs"]
mod upstream_span_assertion_skip;

fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };
    if !expression_is_safe_to_parse(source) {
        return;
    }
    if hits_known_upstream_span_assertion_shape(source.as_bytes()) {
        return;
    }

    let allocator = Allocator::default();
    let parser = Parser::new(
        &allocator,
        source,
        SourceType::default()
            .with_module(true)
            .with_typescript(true),
    );
    let _ = parser.parse_expression();
});
