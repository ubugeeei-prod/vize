//! Actual pinned-parser regressions for PURE annotation recovery and rewind.
//!
//! The original parser marked the same annotation twice on error recovery and
//! pointed duplicate PURE comments at a later ordinary comment during rewind.
//! The exact local parser correction admits the terminal not-applied state and
//! restores the matching authored annotation index. Every boundary case runs
//! the real parser directly: no catch, input rewrite or PURE skip.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    Parsed,
}

fn parse(source: &str) -> Outcome {
    let allocator = Allocator::default();
    let parsed = Parser::new(
        &allocator,
        source,
        SourceType::default()
            .with_module(true)
            .with_typescript(true),
    )
    .parse_expression();
    // Successful or malformed expression diagnostics drop by ordinary ownership.
    drop(parsed);
    Outcome::Parsed
}

/// The minimized reproducer, byte for byte as `cargo fuzz tmin` produced it.
const REPRODUCER: &str = "f<>/((\nd=//#__PURE__0";

#[test]
fn the_minimized_reproducer_returns_normally_owned_parser_observations() {
    assert_eq!(
        parse(REPRODUCER),
        Outcome::Parsed,
        "the repaired pinned parser must parse the exact formerly skipped input"
    );
}

#[test]
fn all_original_boundary_variants_return_without_panic() {
    // Each row drops or alters exactly one ingredient of the reproducer.
    let rows: &[(&str, Outcome)] = &[
        (REPRODUCER, Outcome::Parsed),
        // 1. Type arguments — the backtrack that staleness needs.
        ("/((\nd=//#__PURE__0", Outcome::Parsed),
        ("f<a>/((\nd=//#__PURE__0", Outcome::Parsed),
        // 2. An unterminated construct, still open when the re-lex hits the
        //    comment. Depth matters: one `(` or a balanced pair is not enough.
        ("f<>/(\nd=//#__PURE__0", Outcome::Parsed),
        ("f<>/((()\nd=//#__PURE__0", Outcome::Parsed),
        ("f<>/[\nd=//#__PURE__0", Outcome::Parsed),
        ("f<>\nd=//#__PURE__0", Outcome::Parsed),
        ("f<>`${(\nd=//#__PURE__0", Outcome::Parsed),
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
            Outcome::Parsed,
            "{source:?} must not reach the repaired upstream assertion",
        );
    }
}

/// A different annotation tracked in a different field, with no such assertion.
#[test]
fn the_no_side_effects_annotation_is_unaffected() {
    assert_eq!(parse("f<>/((\nd=//#__NO_SIDE_EFFECTS__0"), Outcome::Parsed,);
}

#[test]
fn program_recovery_retags_only_the_matching_authored_pure_comment() {
    use oxc_ast::ast::CommentContent;

    for source in [
        "(/*#__PURE__*/\n/* other */\nf",
        "(/*@__PURE__*/ /* ordinary */ value)",
    ] {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
        assert_eq!(parsed.program.source_text, source);
        assert_eq!(parsed.program.comments.len(), 2);
        assert_eq!(
            parsed.program.comments[0].content,
            CommentContent::PureNotApplied
        );
        assert_eq!(parsed.program.comments[1].content, CommentContent::None);
        drop(parsed);
    }
}
