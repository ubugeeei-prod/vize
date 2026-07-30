//! Skip predicate for the upstream OXC tuple-type span assertion (#3307).
//!
//! Included both by the `js_ts_expression` fuzz target and by
//! `crates/vize_atelier_core/tests/upstream_tuple_type_span_assertion.rs`, so
//! the predicate the fuzzer applies is the one CI asserts against the boundary
//! matrix. It is not a fuzz-only heuristic living outside test coverage.

/// Returns true when `bytes` can reach the upstream tuple-type span assertion.
///
/// The panic needs two things inside one TS tuple type, and nothing else:
/// 1. an optional member before the offending comma — `[a?, …]`, `[a?]` or the
///    named form `[a?: T, …]` — so the `?` is followed, after optional
///    whitespace, by `,`, `]` or `:`; and
/// 2. a later comma whose element consumes no tokens, which requires at least
///    one byte of trivia after that comma: `<[r?,\x07]` parses cleanly while
///    `<[r?, \x07]` panics.
///
/// Neither `<[` type arguments nor a control byte is part of the class:
/// `f<[a?, , b]>(x)`, `(x as [a?, , b])` and `(x satisfies [a?, , b])` are
/// printable, balanced ASCII and panic identically. The predicate therefore
/// keys on the tuple shape rather than on the `cargo fuzz tmin` reproducer's
/// `<[` prefix.
///
/// Over-skipping only costs fuzz coverage, while under-skipping re-reports a
/// known upstream crash, so the match deliberately ignores bracket nesting and
/// quoting, and searches for the trailing comma to the end of the input.
pub fn hits_known_upstream_span_assertion_shape(bytes: &[u8]) -> bool {
    let Some(first_bracket) = bytes.iter().position(|byte| *byte == b'[') else {
        return false;
    };

    bytes
        .iter()
        .enumerate()
        .skip(first_bracket + 1)
        .filter(|(_, byte)| **byte == b'?')
        .any(|(index, _)| {
            let after_marker = &bytes[index + 1..];
            let Some(offset) = after_marker
                .iter()
                .position(|byte| !byte.is_ascii_whitespace())
            else {
                return false;
            };
            let tail = &after_marker[offset..];
            matches!(tail.first(), Some(b',' | b']' | b':')) && has_comma_followed_by_trivia(tail)
        })
}

/// Returns true when `bytes` holds a comma followed by trivia — ASCII
/// whitespace, a C0 control byte, or end of input — that is, a comma whose
/// element can start past `prev_token_end`.
fn has_comma_followed_by_trivia(bytes: &[u8]) -> bool {
    bytes.iter().enumerate().any(|(index, byte)| {
        *byte == b','
            && match bytes.get(index + 1) {
                None => true,
                Some(next) => next.is_ascii_whitespace() || *next < 0x20,
            }
    })
}
