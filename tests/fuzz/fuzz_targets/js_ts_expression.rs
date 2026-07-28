#![no_main]

// JS/TS expression parser fuzz target.
//
// Drives the same OXC expression parsing path used by template expression
// transforms and import-usage checks. Invalid expressions are expected and
// reported as parser errors; panics are not.
//
// One upstream crash class is skip-listed below until the pinned OXC revision
// carries a fix (#3296; retirement tracked in #3307): a speculative tuple type
// with an optional member recovering over a C0 control byte builds a span
// with `start > end`, tripping a `debug_assert` in `oxc_span::Span::size`
// (`<[r?, \x07]` after `cargo fuzz tmin`). The shape is not a depth or
// balance property, so the expression nesting guard cannot express it; the
// skip costs fuzz coverage only.
use libfuzzer_sys::fuzz_target;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_core::steps::expression::expression_is_safe_to_parse;

/// Returns true when a `<[` span holds both a `?` and a C0 control byte
/// (excluding tab/newline/carriage return) before its closing `]` — the
/// empirical requirements of the upstream tuple-type span assertion (#3307):
/// dropping either the optional member or the control byte parses cleanly.
fn hits_known_upstream_span_assertion_shape(bytes: &[u8]) -> bool {
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'<' && bytes[i + 1] == b'[' {
            let (mut has_question, mut has_control) = (false, false);
            let mut j = i + 2;
            while j < bytes.len() && bytes[j] != b']' {
                match bytes[j] {
                    b'?' => has_question = true,
                    b'\t' | b'\n' | b'\r' => {}
                    b if b < 0x20 => has_control = true,
                    _ => {}
                }
                j += 1;
            }
            if has_question && has_control {
                return true;
            }
            i = j;
        } else {
            i += 1;
        }
    }
    false
}

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
