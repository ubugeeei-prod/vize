//! Boundary matrix for the upstream OXC tuple-type span assertion (#3307).
//!
//! Inside a TS tuple type that already has an optional member, OXC's tuple
//! element recovery builds the element span as `Span::new(start_span(),
//! prev_token_end)`. When the element parse consumes *nothing* — the token after
//! the comma cannot start a type — `start_span()` has already skipped the trivia
//! past `prev_token_end`, so the span comes out inverted (`start > end`).
//! `TS1257 required element cannot follow an optional element` then labels that
//! span and `oxc_span::Span::size` trips `debug_assert!(self.start <= self.end)`.
//!
//! Two properties fully determine the class, and the matrix below pins both:
//! - the tuple needs a preceding optional member (`r?`, `a?:`), and
//! - the element after a comma must consume nothing, which needs at least one
//!   byte of trivia after that comma (`<[r?,\x07]` parses cleanly, `<[r?, \x07]`
//!   does not) followed by a token that cannot start a type.
//!
//! `#3296` minimized this to `<[r?, \x07]` with `cargo fuzz tmin`, which made
//! the class look specific to `<[` type arguments and to C0 control bytes. It is
//! neither: `f<[a?, , b]>(x)` is printable, balanced ASCII and panics
//! identically, and `,`, `;` and `@` all reach the same recovery.
//!
//! Replay evidence (dev profile, debug assertions on): the panic reproduces on
//! the pinned rev `8265ed94` (0.127.0) through 0.140.0 and stops at oxc 0.141.0,
//! where the lazy `ParserDiagnostic` refactor leaves TS1257 unmaterialized so
//! the inverted span is never labeled. The inverted span is still constructed
//! upstream, so oxc-project/oxc#23670 stays open; only the panic is out of
//! reach.
//!
//! When the pinned revision moves to 0.141.0 or later, every
//! `UpstreamSpanAssertion` row here fails. That is the signal to flip those rows
//! to the diagnostics they then produce and to delete the `js_ts_expression`
//! fuzz-target skip.

use std::panic::{self, AssertUnwindSafe};

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use upstream_span_assertion_skip::hits_known_upstream_span_assertion_shape;
use vize_atelier_core::steps::expression::expression_is_safe_to_parse;

// The predicate the `js_ts_expression` fuzz target uses to skip this class, so
// the matrix below asserts the shipped predicate rather than a copy of it.
#[path = "../../../tests/fuzz/fuzz_targets/upstream_span_assertion_skip.rs"]
mod upstream_span_assertion_skip;

/// What Vize's guarded expression-parsing path does with one input.
#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    /// Rejected by Vize's own guard, so OXC never sees it.
    RejectedByGuard,
    /// Handed to OXC, which returned exactly these diagnostic messages.
    Diagnostics(Vec<String>),
    /// Handed to OXC, which tripped the upstream span assertion with exactly
    /// this panic payload.
    UpstreamSpanAssertion(String),
}

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

fn span_assertion() -> Outcome {
    Outcome::UpstreamSpanAssertion("assertion failed: self.start <= self.end".to_string())
}

/// Every input in the class, with the outcome the pinned OXC revision produces.
fn matrix() -> Vec<(&'static str, Outcome)> {
    vec![
        // The #3296 `cargo fuzz tmin` reproducer and its neighbours.
        ("<[r?, \u{7}]", span_assertion()),
        ("<[r?,\u{7}]", diagnostic("Invalid Character `\u{7}`")),
        ("<[r?, \u{1f}]", span_assertion()),
        ("f<[a?, \u{7}]>(x)", span_assertion()),
        ("a<[b?, \u{7}]", span_assertion()),
        // Documented clean neighbours: printable element, no optional member,
        // no second element.
        ("<[r?, x]", diagnostic("Expected `>` but found `EOF`")),
        ("<[\u{7}]", diagnostic("Expected `]` but found `Unknown`")),
        ("<[r?]", diagnostic("Expected `>` but found `EOF`")),
        // oxc-project/oxc#23670's own reproducer: the trivia is a newline and
        // the tuple never closes, so Vize's balance guard rejects it first.
        ("<[m?,\n\n", Outcome::RejectedByGuard),
        ("<[m?,", Outcome::RejectedByGuard),
        // Printable members of the same class: the byte after the comma only
        // has to be unable to start a type.
        ("<[r?, , x]", span_assertion()),
        ("f<[a?, , b]>(x)", span_assertion()),
        ("<[r?,,x]", diagnostic("Unexpected token")),
        ("<[r?, ; ]", span_assertion()),
        ("<[r?, @ ]", span_assertion()),
        ("<[r? , , x]", span_assertion()),
        ("<[a?: T, , b]", span_assertion()),
        ("f<[a?: T, , b]>(x)", span_assertion()),
        // Tuple types reached without `<[` type arguments at all.
        ("(x as [a?, , b])", span_assertion()),
        ("f(y as [a?, , b])", span_assertion()),
        ("(x satisfies [a?, , b])", span_assertion()),
        ("<[a?, , b]>x", span_assertion()),
        // TS1257 with a well-formed span: reported, never inverted.
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

#[test]
fn fuzz_skip_predicate_covers_every_panicking_input() {
    // `false` rows are the ones the fuzzer keeps exercising. Clean inputs may be
    // skipped too: `<[r?,\x07]` parses cleanly but the predicate counts the C0
    // byte after the comma as trivia, and `<[r?, x]` / `f<[a?, b]>(x)` are the
    // well-formed neighbours the shape match cannot separate.
    let expected_skips: Vec<(&str, bool)> = vec![
        ("<[r?, \u{7}]", true),
        ("<[r?,\u{7}]", true),
        ("<[r?, \u{1f}]", true),
        ("f<[a?, \u{7}]>(x)", true),
        ("a<[b?, \u{7}]", true),
        ("<[r?, x]", true),
        ("<[\u{7}]", false),
        ("<[r?]", false),
        ("<[m?,\n\n", true),
        ("<[m?,", true),
        ("<[r?, , x]", true),
        ("f<[a?, , b]>(x)", true),
        ("<[r?,,x]", false),
        ("<[r?, ; ]", true),
        ("<[r?, @ ]", true),
        ("<[r? , , x]", true),
        ("<[a?: T, , b]", true),
        ("f<[a?: T, , b]>(x)", true),
        ("(x as [a?, , b])", true),
        ("f(y as [a?, , b])", true),
        ("(x satisfies [a?, , b])", true),
        ("<[a?, , b]>x", true),
        ("f<[a?, b]>(x)", true),
        ("(x as [a?, b])", true),
    ];
    let actual_skips: Vec<(&str, bool)> = matrix()
        .iter()
        .map(|(source, _)| {
            (
                *source,
                hits_known_upstream_span_assertion_shape(source.as_bytes()),
            )
        })
        .collect();

    assert_eq!(actual_skips, expected_skips);

    // The invariant the fuzz job depends on: over-skipping a clean input only
    // costs coverage, but leaving a panicking input unskipped re-reports the
    // known upstream crash.
    for (source, outcome) in matrix() {
        if matches!(outcome, Outcome::UpstreamSpanAssertion(_)) {
            assert!(
                hits_known_upstream_span_assertion_shape(source.as_bytes()),
                "panicking input {source:?} is not skipped by the fuzz predicate"
            );
        }
    }
}

/// Expressions that never reach the class stay fuzzable: the predicate must not
/// swallow ternaries, optional chaining or nullish coalescing near an array.
#[test]
fn fuzz_skip_predicate_keeps_ordinary_expressions() {
    let sources = [
        "a[0] ? f(1, 2) : g",
        "[cond ? a : b, c]",
        "[a?.b, c]",
        "a[x ?? y, z]",
        "[a, b, c]",
        "f<[a, b]>(x)",
        "x ? y : z",
    ];
    let actual: Vec<(&str, bool)> = sources
        .iter()
        .map(|source| {
            (
                *source,
                hits_known_upstream_span_assertion_shape(source.as_bytes()),
            )
        })
        .collect();

    assert_eq!(
        actual,
        vec![
            ("a[0] ? f(1, 2) : g", false),
            ("[cond ? a : b, c]", false),
            ("[a?.b, c]", false),
            ("a[x ?? y, z]", false),
            ("[a, b, c]", false),
            ("f<[a, b]>(x)", false),
            ("x ? y : z", false),
        ]
    );
}
