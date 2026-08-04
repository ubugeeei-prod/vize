//! Boundary matrix for the upstream OXC pure-comment assertion.
//!
//! `oxc_parser`'s `TriviaBuilder` records the index of the most recent
//! `#__PURE__` / `@__PURE__` annotation in `pure_comment`. When the annotated
//! expression turns out not to be a call, the parser calls
//! `mark_current_pure_comment_not_applied`, which asserts the comment at that
//! index is still pure:
//!
//! ```text
//! oxc_parser/src/lexer/trivia_builder.rs:62
//! debug_assert!(comment.is_pure());
//! ```
//!
//! Type arguments make the parser backtrack and re-lex. When the re-lex passes
//! over a *different* comment, the recorded index is stale and the assertion
//! fires. Found by the `js_ts_expression` fuzz target on the v0.324.0 release
//! candidate; minimized with `cargo fuzz tmin` to 21 bytes.
//!
//! ## The class needs all three ingredients
//!
//! Established by the matrix below, each row differing from the reproducer in
//! one element:
//!
//! 1. **Type arguments** (`f<>`, `f<a>`) — without them nothing backtracks.
//! 2. **An unterminated bracketing construct left open at end of line** —
//!    `/((` (regex) or `` `${( `` (template). One unbalanced `(` is not enough
//!    (`/(` parses cleanly) and neither is a balanced one (`/((()`); the
//!    re-lex has to still be inside it when it reaches the comment.
//! 3. **An assignment whose right-hand side opens with the annotation**, where
//!    the annotation does not go on to attach to a call. `d=//#__PURE__0`
//!    panics, `d+//#__PURE__0` does not (not an assignment), `(d)=…` does not,
//!    and `d=/*@__PURE__*/x()` does not — there the annotation *is* applied, so
//!    the not-applied path never runs.
//!
//! Both spellings reach it (`#__PURE__` and `@__PURE__`) and both comment forms
//! (`//` and `/* */`). `#__NO_SIDE_EFFECTS__` does not: it is tracked in a
//! separate field with no such assertion.
//!
//! ## Why this is skipped in the fuzz target rather than fixed here
//!
//! It is a `debug_assert!`, so it compiles out of the release profile the
//! shipped binaries are built with — no vize user can reach it. `cargo fuzz`
//! turns debug assertions on, which is why only the fuzz job sees it. There is
//! nothing for vize to fix: the stale index is built entirely inside oxc's
//! lexer, from input vize hands over verbatim.
//!
//! This matrix is the durable half of that trade. `js_ts_expression` skips
//! sources carrying a pure annotation, so the class cannot fail the release
//! gate again; this test keeps every row executable, so a pin bump that fixes
//! the assertion upstream fails *here* — visibly, in a workspace test — instead
//! of silently widening the fuzz skip forever.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    /// Parsed, with or without diagnostics. No assertion tripped.
    Parsed,
    /// `debug_assert!(comment.is_pure())` in OXC's `TriviaBuilder`.
    UpstreamPureCommentAssertion,
}

/// The panic hook is deliberately left alone: swapping it out around
/// `catch_unwind` races with the other tests in this file, which run in
/// parallel on the same process-wide hook. The expected panics therefore print
/// their message, which `cargo test` captures per test and only shows on
/// failure.
fn parse(source: &str) -> Outcome {
    let result = std::panic::catch_unwind(|| {
        let allocator = Allocator::default();
        let _ = Parser::new(
            &allocator,
            source,
            SourceType::default()
                .with_module(true)
                .with_typescript(true),
        )
        .parse_expression();
    });

    let payload = match result {
        Ok(()) => return Outcome::Parsed,
        Err(payload) => payload,
    };

    // Only the pure-comment assertion counts. Anything else is a real parser
    // bug and has to keep failing loudly instead of satisfying a row here.
    let message = payload
        .downcast_ref::<&str>()
        .map(|message| (*message).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned());
    match message {
        Some(message) if message.contains("comment.is_pure()") => {
            Outcome::UpstreamPureCommentAssertion
        }
        _ => std::panic::resume_unwind(payload),
    }
}

/// The minimized reproducer, byte for byte as `cargo fuzz tmin` produced it.
const REPRODUCER: &str = "f<>/((\nd=//#__PURE__0";

#[test]
fn the_minimized_reproducer_still_reaches_the_upstream_assertion() {
    assert_eq!(
        parse(REPRODUCER),
        Outcome::UpstreamPureCommentAssertion,
        "if this now parses, OXC fixed the assertion: drop the pure-annotation \
         skip in tests/fuzz/fuzz_targets/js_ts_expression.rs and this file with it"
    );
}

#[test]
fn every_ingredient_is_required() {
    // Each row drops or alters exactly one ingredient of the reproducer.
    let rows: &[(&str, Outcome)] = &[
        (REPRODUCER, Outcome::UpstreamPureCommentAssertion),
        // 1. Type arguments — the backtrack that staleness needs.
        ("/((\nd=//#__PURE__0", Outcome::Parsed),
        (
            "f<a>/((\nd=//#__PURE__0",
            Outcome::UpstreamPureCommentAssertion,
        ),
        // 2. An unterminated construct, still open when the re-lex hits the
        //    comment. Depth matters: one `(` or a balanced pair is not enough.
        ("f<>/(\nd=//#__PURE__0", Outcome::Parsed),
        ("f<>/((()\nd=//#__PURE__0", Outcome::Parsed),
        ("f<>/[\nd=//#__PURE__0", Outcome::Parsed),
        ("f<>\nd=//#__PURE__0", Outcome::Parsed),
        (
            "f<>`${(\nd=//#__PURE__0",
            Outcome::UpstreamPureCommentAssertion,
        ),
        // 3. An assignment whose RHS opens with an unapplied annotation.
        ("f<>/((\n//#__PURE__0", Outcome::Parsed),
        ("f<>/((\nd+//#__PURE__0", Outcome::Parsed),
        ("f<>/((\n(d)=//#__PURE__0", Outcome::Parsed),
        ("f<>/((\nd=/*@__PURE__*/x()", Outcome::Parsed),
    ];

    for (source, expected) in rows {
        assert_eq!(&parse(source), expected, "{source:?}");
    }
}

#[test]
fn both_annotation_spellings_and_comment_forms_reach_it() {
    for source in [
        "f<>/((\nd=//#__PURE__0",
        "f<>/((\nd=//@__PURE__0",
        "f<>/((\nd=/*#__PURE__*/0",
        "f<>/((\nd=//#__PURE__",
        "f<>/((\nd=//#__PURE__ 0",
    ] {
        assert_eq!(
            parse(source),
            Outcome::UpstreamPureCommentAssertion,
            "{source:?} should still reach the upstream assertion",
        );
    }
}

/// A different annotation tracked in a different field, with no such assertion.
#[test]
fn the_no_side_effects_annotation_is_unaffected() {
    assert_eq!(parse("f<>/((\nd=//#__NO_SIDE_EFFECTS__0"), Outcome::Parsed,);
}
