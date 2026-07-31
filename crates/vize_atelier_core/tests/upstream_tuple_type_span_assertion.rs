//! Boundary matrix for the upstream OXC tuple-type span assertion (#3307).
//!
//! Inside a TS tuple type that already has an optional member, OXC's tuple
//! element recovery builds the element span as `Span::new(start_span(),
//! prev_token_end)`. When the element parse consumes *nothing* — the token after
//! the comma cannot start a type — `start_span()` has already skipped the trivia
//! past `prev_token_end`, so the span comes out inverted (`start > end`).
//! `TS1257 required element cannot follow an optional element` then labeled that
//! span and `oxc_span::Span::size` tripped `debug_assert!(self.start <= self.end)`.
//!
//! Two properties fully determined the class, and the matrix below still pins
//! both:
//! - the tuple needs a preceding optional member (`r?`, `a?:`), and
//! - the element after a comma must consume nothing, which needs at least one
//!   byte of trivia after that comma (`<[r?,\x07]` parses cleanly, `<[r?, \x07]`
//!   did not) followed by a token that cannot start a type.
//!
//! `#3296` minimized this to `<[r?, \x07]` with `cargo fuzz tmin`, which made
//! the class look specific to `<[` type arguments and to C0 control bytes. It is
//! neither: `f<[a?, , b]>(x)` is printable, balanced ASCII and panicked
//! identically, and `,`, `;` and `@` all reach the same recovery.
//!
//! **The panic is out of reach as of the pin bump in #3405.** Replay evidence
//! (dev profile, debug assertions on): it reproduced on the previously pinned
//! rev `8265ed94` (0.127.0) through 0.140.0 and stops at oxc 0.141.0, where the
//! lazy `ParserDiagnostic` refactor leaves TS1257 unmaterialized so the inverted
//! span is never labeled. The workspace now pins `crates_v0.142.0`
//! (`fc702c1f`), and every row that used to panic reports the recovery
//! diagnostic instead — `Unexpected token`, or the lexer's `Invalid Character`
//! when the offending byte is a C0 control byte.
//!
//! TS1257 itself is *not* gone: the two well-formed rows at the end still report
//! it, because their span is built normally. Only the inverted-span path stopped
//! materializing it.
//!
//! The inverted span is still constructed upstream, so oxc-project/oxc#23670
//! stays open. The matrix is kept — and `UpstreamSpanAssertion` is kept as a
//! reachable `Outcome` variant — so that a future pin which re-materializes
//! TS1257 over the inverted span fails here, in a workspace test, instead of
//! only in the nightly fuzz job.

use std::panic::{self, AssertUnwindSafe};

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_core::steps::expression::expression_is_safe_to_parse;

/// What Vize's guarded expression-parsing path does with one input.
#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    /// Rejected by Vize's own guard, so OXC never sees it.
    RejectedByGuard,
    /// Handed to OXC, which returned exactly these diagnostic messages.
    Diagnostics(Vec<String>),
    /// Handed to OXC, which tripped the upstream span assertion with exactly
    /// this panic payload. No row produces this on the pinned revision; the
    /// variant stays so a regressing pin is reported as a value mismatch rather
    /// than aborting the test binary.
    UpstreamSpanAssertion(String),
}

/// The exact bytes `cargo fuzz` saved as
/// `crash-f2b2934c4b4dd19d1257bbcc46b998bc2f1e8d69` in the
/// `fuzz-reproducers-js_ts_expression` artifact of run 30335797526 — the crash
/// that opened #3296. `cargo fuzz tmin` reduced it to `<[r?, \x07]`, which is
/// the first row of [`matrix`]; this is the unreduced input, replayed here so
/// the artifact does not have to be re-downloaded to re-verify the fix.
const REPRODUCER_3296: &[u8] = b"dpf<[number, number?, \x07\x00\x00\x00numbr]?XXXX,XX$4:e `=(p)";

/// Runs one input through the same guard-then-parse path as the
/// `js_ts_expression` fuzz target, catching the upstream assertion instead of
/// aborting the test binary.
fn classify(source: &str) -> Outcome {
    if !expression_is_safe_to_parse(source) {
        return Outcome::RejectedByGuard;
    }

    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let parsed = panic::catch_unwind(AssertUnwindSafe(|| {
        let allocator = Allocator::default();
        let parser = Parser::new(
            &allocator,
            source,
            SourceType::default()
                .with_module(true)
                .with_typescript(true),
        );
        match parser.parse_expression() {
            Ok(_) => Vec::new(),
            Err(errors) => errors
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
        }
    }));
    panic::set_hook(previous_hook);

    match parsed {
        Ok(messages) => Outcome::Diagnostics(messages),
        Err(payload) => Outcome::UpstreamSpanAssertion(
            payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| {
                    payload
                        .downcast_ref::<&str>()
                        .map(|text| (*text).to_string())
                })
                .unwrap_or_else(|| "<non-string panic payload>".to_string()),
        ),
    }
}

fn diagnostic(message: &str) -> Outcome {
    Outcome::Diagnostics(vec![message.to_string()])
}

/// Every input in the class, with the outcome the pinned OXC revision produces.
fn matrix() -> Vec<(&'static str, Outcome)> {
    vec![
        // The #3296 `cargo fuzz tmin` reproducer and its neighbours. These three
        // used to panic (except the no-trivia `<[r?,\x07]`); the lexer's
        // invalid-character diagnostic is what survives now.
        ("<[r?, \u{7}]", diagnostic("Invalid Character `\u{7}`")),
        ("<[r?,\u{7}]", diagnostic("Invalid Character `\u{7}`")),
        ("<[r?, \u{1f}]", diagnostic("Invalid Character `\u{1f}`")),
        ("f<[a?, \u{7}]>(x)", diagnostic("Unexpected token")),
        ("a<[b?, \u{7}]", diagnostic("Unexpected token")),
        // Documented clean neighbours: printable element, no optional member,
        // no second element. Unchanged by the bump.
        ("<[r?, x]", diagnostic("Expected `>` but found `EOF`")),
        ("<[\u{7}]", diagnostic("Expected `]` but found `Unknown`")),
        ("<[r?]", diagnostic("Expected `>` but found `EOF`")),
        // oxc-project/oxc#23670's own reproducer: the trivia is a newline and
        // the tuple never closes, so Vize's balance guard rejects it first.
        ("<[m?,\n\n", Outcome::RejectedByGuard),
        ("<[m?,", Outcome::RejectedByGuard),
        // Printable members of the same class: the byte after the comma only
        // has to be unable to start a type. All of these used to panic except
        // `<[r?,,x]`, and all now collapse onto the same recovery diagnostic.
        ("<[r?, , x]", diagnostic("Unexpected token")),
        ("f<[a?, , b]>(x)", diagnostic("Unexpected token")),
        ("<[r?,,x]", diagnostic("Unexpected token")),
        ("<[r?, ; ]", diagnostic("Unexpected token")),
        ("<[r?, @ ]", diagnostic("Unexpected token")),
        ("<[r? , , x]", diagnostic("Unexpected token")),
        ("<[a?: T, , b]", diagnostic("Unexpected token")),
        ("f<[a?: T, , b]>(x)", diagnostic("Unexpected token")),
        // Tuple types reached without `<[` type arguments at all. These are the
        // rows that made the class reachable from `strip_typescript_from_expression`.
        ("(x as [a?, , b])", diagnostic("Unexpected token")),
        ("f(y as [a?, , b])", diagnostic("Unexpected token")),
        ("(x satisfies [a?, , b])", diagnostic("Unexpected token")),
        ("<[a?, , b]>x", diagnostic("Unexpected token")),
        // TS1257 with a well-formed span: still reported, never inverted.
        (
            "f<[a?, b]>(x)",
            diagnostic("A required element cannot follow an optional element."),
        ),
        (
            "(x as [a?, b])",
            diagnostic("A required element cannot follow an optional element."),
        ),
    ]
}

#[test]
fn upstream_tuple_type_span_assertion_boundary_matrix() {
    let expected = matrix();
    let actual: Vec<(&str, Outcome)> = expected
        .iter()
        .map(|(source, _)| (*source, classify(source)))
        .collect();

    assert_eq!(actual, expected);
}

/// The whole point of the pin bump: no input in the class aborts a
/// debug-assertions build any more, so the `js_ts_expression` fuzz target no
/// longer needs a skip predicate.
#[test]
fn no_input_in_the_class_trips_the_upstream_span_assertion() {
    let actual: Vec<(&str, bool)> = matrix()
        .iter()
        .map(|(source, _)| {
            (
                *source,
                matches!(classify(source), Outcome::UpstreamSpanAssertion(_)),
            )
        })
        .collect();

    let expected: Vec<(&str, bool)> = matrix()
        .iter()
        .map(|(source, _)| (*source, false))
        .collect();

    assert_eq!(actual, expected);
}

/// Replay of the unreduced `js_ts_expression` reproducer from #3296 through the
/// same guard-then-parse path the fuzz target runs, now that the target no
/// longer skips this class.
#[test]
fn issue_3296_fuzz_reproducer_parses_without_panicking() {
    let source = std::str::from_utf8(REPRODUCER_3296).expect("the reproducer is valid UTF-8");

    assert_eq!(classify(source), diagnostic("Unexpected token"));
}
