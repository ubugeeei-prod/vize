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
// The pinned parser's PURE error-recovery/idempotency and duplicate-comment
// rewind defects are repaired by the local exact-source parser patch. PURE
// annotations are no longer skipped. The retained boundary matrix in
// `crates/vize_atelier_core/tests/upstream_pure_comment_assertion.rs` now
// parses every formerly skipped class directly and requires no panic.
use libfuzzer_sys::fuzz_target;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_core::steps::expression::expression_is_safe_to_parse;

const MAX_FUZZ_RAW_ARRAY_DEPTH: usize = 256;

/// Keep malformed delimiter streams from reaching the recursive parser when
/// the product guard interprets an earlier token as an unterminated literal.
/// This intentionally over-counts brackets inside invalid literals: the fuzz
/// target should keep exploring their smaller forms without spending seconds
/// on thousands of recursive array productions hidden behind that recovery.
fn has_excessive_raw_array_depth(source: &str) -> bool {
    let mut depth = 0usize;
    for byte in source.bytes() {
        match byte {
            b'[' => {
                depth += 1;
                if depth > MAX_FUZZ_RAW_ARRAY_DEPTH {
                    return true;
                }
            }
            b']' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    false
}

fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };
    if !expression_is_safe_to_parse(source) || has_excessive_raw_array_depth(source) {
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
