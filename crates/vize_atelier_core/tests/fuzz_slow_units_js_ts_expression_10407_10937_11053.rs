//! Replay of the `js_ts_expression` slow units from #10407, #10937 and #11053.
//!
//! The three campaigns ran before the type-argument probe fixes from #10612
//! (impossible trailing probes) and #10945 (shared fatal subproblems) reached
//! `main`, and their artifacts belong to the same malformed tuple/type-argument
//! family as the retained #10401 and #10845 inputs. #10407 also carried the
//! exact #10401 artifact, which `issue-10401.input` already retains. Every
//! other artifact is kept byte for byte in the regression corpus, seeded into
//! the fuzz corpus by `tools/commands/ci/fuzz/seed_corpus.rs`, and replayed
//! here on the fuzz target's own path: the shared guard admits the input, and
//! the parser rejects it with a bounded arena instead of speculating for
//! seconds.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_core::steps::expression::expression_is_safe_to_parse;

/// Exact artifact bytes with the length each campaign recorded.
const SLOW_UNITS: [(&str, &str, usize); 6] = [
    (
        "issue-10407.input",
        include_str!("../../../tests/fuzz/regressions/js_ts_expression/issue-10407.input"),
        23_588,
    ),
    (
        "issue-10937.input",
        include_str!("../../../tests/fuzz/regressions/js_ts_expression/issue-10937.input"),
        23_857,
    ),
    (
        "issue-10937.slow-1.input",
        include_str!("../../../tests/fuzz/regressions/js_ts_expression/issue-10937.slow-1.input"),
        23_857,
    ),
    (
        "issue-10937.slow-2.input",
        include_str!("../../../tests/fuzz/regressions/js_ts_expression/issue-10937.slow-2.input"),
        23_857,
    ),
    (
        "issue-11053.input",
        include_str!("../../../tests/fuzz/regressions/js_ts_expression/issue-11053.input"),
        23_262,
    ),
    (
        "issue-11053.slow-1.input",
        include_str!("../../../tests/fuzz/regressions/js_ts_expression/issue-11053.slow-1.input"),
        23_261,
    ),
];

/// The pre-fix parse of the sibling #10845 input used about 132 MB of arena;
/// each retained input now stays below 2.5 MB.
const ARENA_CEILING_BYTES: usize = 4 * 1024 * 1024;

#[test]
fn retained_slow_units_are_admitted_and_parse_with_a_bounded_arena() {
    for (name, source, bytes) in SLOW_UNITS {
        assert_eq!(
            source.len(),
            bytes,
            "{name} must keep the campaign artifact bytes"
        );
        assert!(
            expression_is_safe_to_parse(source),
            "{name} must reach the parser the way the fuzz target replays it"
        );

        let allocator = Allocator::default();
        let parsed = Parser::new(
            &allocator,
            source,
            SourceType::default()
                .with_module(true)
                .with_typescript(true),
        )
        .parse_expression();

        assert!(parsed.is_err(), "{name} must remain a diagnostic");
        assert!(
            allocator.used_bytes() < ARENA_CEILING_BYTES,
            "{name} allocated {} arena bytes",
            allocator.used_bytes()
        );
    }
}
