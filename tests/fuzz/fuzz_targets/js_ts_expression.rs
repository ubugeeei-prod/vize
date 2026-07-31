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
