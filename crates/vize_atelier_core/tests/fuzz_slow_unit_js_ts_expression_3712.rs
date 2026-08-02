//! Replay of the `js_ts_expression` slow-unit reproducer (#3712).
//!
//! The reproducer is 10,729 bytes of identifier soup holding 837 `<` and not a
//! single `>`. Every angle is followed by an identifier, so OXC speculates that
//! each one opens another type-argument list (`f<T>`), recurses into the next,
//! and — because nothing ever closes — rewinds the whole cascade. Parsing it
//! costs ~1.47s in release mode; libFuzzer reported it as a slow unit.
//!
//! The guard already classified `<{`, `<[` (#2944), `<!` (#3213) and `<(`
//! (#3277/#3279/#3281) as speculative type-angle opens. `<Identifier` — the
//! plainest type-argument list of the four — was the hole, so none of the 837
//! angles counted toward the depth budget and the input scored depth 4.
//!
//! This is not merely a fuzz-harness shape: `steps::v_slot::extract_slot_prop_names`
//! parses `v-slot="..."` with TypeScript enabled behind the same guard, so the
//! same bytes in a template attribute stalled the compiler.
//!
//! The bytes live in `fixtures/js_ts_expression_slow_unit_3712.txt` so the
//! workflow artifact does not have to be re-downloaded to re-verify the fix; the
//! file name is libFuzzer's, and its SHA-1 is the `slow-unit-<sha>` suffix.

use std::hint::black_box;
use std::time::{Duration, Instant};

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_core::steps::expression::{
    MAX_EXPRESSION_NESTING_DEPTH, expression_has_balanced_delimiters, expression_is_safe_to_parse,
    expression_nesting_depth, prefix_identifiers_in_expression, strip_typescript_from_expression,
};
use vize_atelier_core::steps::v_slot::extract_slot_prop_names;

/// The exact bytes `cargo fuzz` saved as
/// `slow-unit-f3e3f6dd93baa2a605765668b0d34ba243cc7bb9` in the
/// `fuzz-reproducers-js_ts_expression` artifact of run 30736557966, which opened
/// #3712.
const REPRODUCER: &str = include_str!("fixtures/js_ts_expression_slow_unit_3712.txt");

/// What Vize's guarded expression-parsing path does with one input.
#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    /// Rejected by Vize's own guard, so OXC never sees it.
    RejectedByGuard,
    /// Handed to OXC, which returned exactly these diagnostic messages.
    Diagnostics(Vec<String>),
}

/// Runs one input through the same guard-then-parse path as the
/// `js_ts_expression` fuzz target.
fn classify(source: &str) -> Outcome {
    if !expression_is_safe_to_parse(source) {
        return Outcome::RejectedByGuard;
    }

    let allocator = Allocator::default();
    let parser = Parser::new(
        &allocator,
        source,
        SourceType::default()
            .with_module(true)
            .with_typescript(true),
    );
    Outcome::Diagnostics(match parser.parse_expression() {
        Ok(_) => Vec::new(),
        Err(errors) => errors
            .iter()
            .map(std::string::ToString::to_string)
            .collect(),
    })
}

/// The prefix of the reproducer ending at its `n`-th `<`, inclusive.
fn prefix_through_angle(n: usize) -> &'static str {
    let (offset, _) = REPRODUCER
        .char_indices()
        .filter(|&(_, c)| c == '<')
        .nth(n - 1)
        .expect("reproducer has fewer angles than requested");
    &REPRODUCER[..=offset]
}

#[test]
fn slow_unit_reproducer_is_rejected_by_the_expression_guard() {
    // Pin the artifact so a truncated or re-encoded fixture fails loudly here
    // rather than silently weakening every assertion below.
    assert_eq!(REPRODUCER.len(), 10729);
    assert_eq!(REPRODUCER.matches('<').count(), 837);
    assert_eq!(REPRODUCER.matches('>').count(), 0);

    // Delimiters are balanced, so the guard's other rejection reason is not what
    // is doing the work: the verdict rests entirely on the angle budget. Every
    // one of the 837 unclosed angles now counts.
    assert!(expression_has_balanced_delimiters(REPRODUCER));
    assert_eq!(expression_nesting_depth(REPRODUCER), 837);
    assert!(!expression_is_safe_to_parse(REPRODUCER));
    assert_eq!(classify(REPRODUCER), Outcome::RejectedByGuard);

    // Rejected input is returned untouched by both public rewriters, and the
    // production `v-slot` path — which parses with TypeScript enabled — extracts
    // nothing instead of stalling.
    assert_eq!(
        prefix_identifiers_in_expression(REPRODUCER).as_str(),
        REPRODUCER
    );
    assert_eq!(
        strip_typescript_from_expression(REPRODUCER).as_str(),
        REPRODUCER
    );
    assert_eq!(format!("{:?}", extract_slot_prop_names(REPRODUCER)), "[]");
}

#[test]
fn slow_unit_reproducer_boundary_sits_exactly_on_the_depth_budget() {
    // The budget is enforced on the reproducer's own bytes, not just on
    // synthesised chains: its first parenthesis is at offset 6938 and its first
    // `&`/`|` at 2057, so both prefixes below are pure unclosed-angle runs.
    let at_limit = prefix_through_angle(MAX_EXPRESSION_NESTING_DEPTH);
    assert_eq!(
        expression_nesting_depth(at_limit),
        MAX_EXPRESSION_NESTING_DEPTH
    );
    assert!(expression_is_safe_to_parse(at_limit));
    // Accepted, handed to OXC, and cheap: the prefix ends on its 31st `<`, so
    // OXC exhausts the input mid-speculation and recovers with one diagnostic.
    assert_eq!(
        classify(at_limit),
        Outcome::Diagnostics(vec!["Unexpected token".to_string()])
    );

    let over_limit = prefix_through_angle(MAX_EXPRESSION_NESTING_DEPTH + 1);
    assert_eq!(
        expression_nesting_depth(over_limit),
        MAX_EXPRESSION_NESTING_DEPTH + 1
    );
    assert!(!expression_is_safe_to_parse(over_limit));
    assert_eq!(classify(over_limit), Outcome::RejectedByGuard);
}

/// Reps for the control input. Chosen so the budget lands in the tens of
/// milliseconds — far above scheduler jitter, still trivial to run.
const CONTROL_REPS: usize = 1000;
/// Reps for the reproducer. Kept small so a regression fails in seconds rather
/// than minutes, but above one so a single scheduling hiccup cannot decide the
/// verdict.
const REPRODUCER_REPS: usize = 3;

/// The reproducer with `&&z` injected after every tenth `<`.
///
/// Same alphabet, same `<` density, slightly longer — but `&&` cannot appear
/// inside a type-argument list, so it ends OXC's speculation exactly like the
/// guard's own reset does. The guard accepts it, which makes it a control that
/// measures a *full TypeScript parse* of an equivalent input.
fn control_input() -> String {
    let mut control = String::with_capacity(REPRODUCER.len() * 2);
    let mut angles = 0usize;
    for c in REPRODUCER.chars() {
        control.push(c);
        if c == '<' {
            angles += 1;
            if angles.is_multiple_of(10) {
                control.push_str("&&z");
            }
        }
    }
    control
}

fn time_guarded(reps: usize, source: &str) -> Duration {
    let start = Instant::now();
    for _ in 0..reps {
        if !expression_is_safe_to_parse(black_box(source)) {
            continue;
        }
        let allocator = Allocator::default();
        let parser = Parser::new(
            &allocator,
            source,
            SourceType::default()
                .with_module(true)
                .with_typescript(true),
        );
        black_box(parser.parse_expression().is_ok());
    }
    start.elapsed()
}

#[test]
fn slow_unit_reproducer_costs_less_than_a_guard_accepted_control() {
    // A wall-clock ceiling would be flaky under parallel load, so the budget is
    // measured on the same machine in the same run: three guarded passes over
    // the reproducer must cost less than a thousand full parses of a same-size,
    // same-alphabet input the guard accepts.
    //
    // Before the fix the reproducer reached OXC and took ~1.47s per pass, some
    // 200x the whole budget. After it, the guard rejects in ~17us and the margin
    // is roughly 400x the other way.
    let control = control_input();
    assert!(
        expression_is_safe_to_parse(&control),
        "control must reach OXC or it measures a rejection, not a parse"
    );

    time_guarded(CONTROL_REPS / 10, &control);
    let budget = time_guarded(CONTROL_REPS, &control);
    let spent = time_guarded(REPRODUCER_REPS, REPRODUCER);

    assert!(
        spent <= budget,
        "{REPRODUCER_REPS} guarded passes over the reproducer took {spent:?}, \
         over the {budget:?} budget of {CONTROL_REPS} guard-accepted parses"
    );
}
