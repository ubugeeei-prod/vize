#![no_main]

// JS/TS expression parser fuzz target.
//
// Drives the same OXC expression parsing path used by template expression
// transforms and import-usage checks. Invalid expressions are expected and
// reported as parser errors; panics are not.
//
// This target used to skip one upstream crash class (#3296, tracked to
// retirement in #3307): a TS tuple type with an optional member, whose next
// element consumes no tokens, built that element's span as
// `Span::new(start_span(), prev_token_end)` with `start > end`, and labeling it
// for `TS1257 required element cannot follow an optional element` tripped
// `debug_assert!(self.start <= self.end)` in `oxc_span::Span::size`.
//
// The skip was removed when the pinned OXC revision moved to 0.142.0 (#3405):
// from 0.141.0 onward the lazy `ParserDiagnostic` refactor leaves TS1257
// unmaterialized, so the inverted span is never labeled and the class parses
// cleanly. The whole class is replayed as a workspace test by
// `crates/vize_atelier_core/tests/upstream_tuple_type_span_assertion.rs`, so a
// pin that reintroduces the panic fails CI rather than only the fuzz job.
// A second upstream crash class is skipped: a `#__PURE__` / `@__PURE__`
// annotation whose recorded index goes stale across a type-argument backtrack
// trips `debug_assert!(comment.is_pure())` in OXC's `TriviaBuilder`
// (`oxc_parser/src/lexer/trivia_builder.rs:62`). Minimized to
// `f<>/((\nd=//#__PURE__0`.
//
// It is a `debug_assert!`, so it compiles out of the release profile the
// shipped binaries use and no vize user can reach it; `cargo fuzz` turns debug
// assertions on, which is why only this job sees it. The stale index is built
// entirely inside OXC's lexer from input vize hands over verbatim, so there is
// nothing here to fix.
//
// The skip is the necessary ingredient rather than the full three-part class
// (type arguments, an unterminated bracketing construct, an assignment whose
// right-hand side opens with an unapplied annotation), because a precise
// predicate would have to re-implement the lexer state that goes stale. It
// costs coverage only for sources carrying a pure annotation.
//
// `crates/vize_atelier_core/tests/upstream_pure_comment_assertion.rs` replays
// the whole class as a workspace test, so a pin bump that fixes it upstream
// fails there — visibly — instead of leaving this skip to widen forever.
use libfuzzer_sys::fuzz_target;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_core::steps::expression::expression_is_safe_to_parse;

fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };
    if !expression_is_safe_to_parse(source) {
        return;
    }
    if carries_pure_annotation(source) {
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

/// Whether the source carries a `#__PURE__` / `@__PURE__` annotation, the one
/// ingredient the upstream `TriviaBuilder` assertion cannot fire without.
fn carries_pure_annotation(source: &str) -> bool {
    source.contains("#__PURE__") || source.contains("@__PURE__")
}
