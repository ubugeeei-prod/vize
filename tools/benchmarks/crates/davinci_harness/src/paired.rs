//! Paired wall-clock sampling for optimized differential test binaries.
use std::time::Instant;

const PAIRS: usize = 31;

/// Measure both implementations on alternating sides of each pair. A sustained
/// slowdown needs 23 of 31 pairs on two runs with opposite starting order.
/// Comparing whole batches also preserves differences below one nanosecond per call.
pub fn assert_paired_time(label: &str, iterations: u32, mut run: impl FnMut(bool)) {
    assert!(iterations > 0);
    let mut measure = |count: u32, reverse: bool| {
        let mut sample = |candidate| {
            let start = Instant::now();
            for _ in 0..count {
                run(candidate);
            }
            start.elapsed().as_nanos()
        };
        let mut pair = |round: usize| {
            let first = round.is_multiple_of(2) != reverse;
            let a = sample(first);
            let b = sample(!first);
            if first { (b, a) } else { (a, b) }
        };
        for warmup in 0..4 {
            pair(warmup);
        }
        let mut baseline = [0; PAIRS];
        let mut candidate = [0; PAIRS];
        let mut slower = 0;
        for (round, (old, new)) in baseline.iter_mut().zip(&mut candidate).enumerate() {
            (*old, *new) = pair(round);
            slower += usize::from(*new > *old);
        }
        baseline.sort_unstable();
        candidate.sort_unstable();
        println!(
            "{label} baseline_ns={} candidate_ns={} slower={slower}/{PAIRS} iterations={count}",
            baseline[PAIRS / 2] / u128::from(count),
            candidate[PAIRS / 2] / u128::from(count)
        );
        slower >= 23
    };
    assert!(
        !(measure(iterations, false) && measure(iterations.saturating_mul(4), true)),
        "reproducible paired slowdown for {label}"
    );
}

/// [`assert_paired_time`] for one small input whose two reads can cost the same.
///
/// Each round times the baseline twice and the candidate once, rotating the
/// three positions, so an A/A control runs in the same loop. A slowdown needs
/// both the sign test (23 of 31 pairs, reproduced in the opposite rotation at
/// 4× iterations) and a median slowdown larger than the A/A spread: the
/// median of `|A₂/A₁ − 1|` over the 31 baseline pairs.
///
/// Why: equal-cost reads (both sides parse the same grammar, or neither
/// parses) move by about 1% when unrelated code moves, and the sign test alone
/// reproduces that shift. On CI runners, with the compared code unchanged, an
/// empty-block read measured the candidate 3.3% faster, 0.8% faster and then
/// 0.8% slower (31 of 31 pairs) on three consecutive heads, and a TSX read
/// 0.7% faster and then 0.8% slower (28 of 31). The A/A spread from the same
/// runner, input and loop is the size below which such a shift is noise.
/// Aggregate corpus passes keep the strict [`assert_paired_time`], and heap
/// counts stay exact.
pub fn assert_calibrated_paired_time(label: &str, iterations: u32, mut run: impl FnMut(bool)) {
    assert!(iterations > 0);
    let mut measure = |count: u32, reverse: bool| {
        let mut sample = |candidate| {
            let start = Instant::now();
            for _ in 0..count {
                run(candidate);
            }
            start.elapsed().as_nanos()
        };
        // Positions of (first baseline, candidate, second baseline) in a round.
        let mut round = |index: usize| {
            let rotation = if reverse { 2 - index % 3 } else { index % 3 };
            let (mut first, mut candidate, mut second) = (0, 0, 0);
            for slot in 0..3 {
                match (slot + rotation) % 3 {
                    0 => first = sample(false),
                    1 => candidate = sample(true),
                    _ => second = sample(false),
                }
            }
            (first, candidate, second)
        };
        for warmup in 0..4 {
            round(warmup);
        }
        let mut baseline = [0; PAIRS];
        let mut candidate = [0; PAIRS];
        let mut control = [0.0_f64; PAIRS];
        let mut slower = 0;
        for (index, ((old, new), aa)) in baseline
            .iter_mut()
            .zip(&mut candidate)
            .zip(&mut control)
            .enumerate()
        {
            let (first, next, second) = round(index);
            (*old, *new) = (first, next);
            *aa = (second as f64 / first.max(1) as f64 - 1.0).abs();
            slower += usize::from(next > first);
        }
        baseline.sort_unstable();
        candidate.sort_unstable();
        control.sort_unstable_by(f64::total_cmp);
        let (old, new, spread) = (
            baseline[PAIRS / 2],
            candidate[PAIRS / 2],
            control[PAIRS / 2],
        );
        let slowdown = new as f64 / old.max(1) as f64 - 1.0;
        println!(
            "{label} baseline_ns={} candidate_ns={} slower={slower}/{PAIRS} slowdown={:+.3}% aa_spread={:.3}% iterations={count}",
            old / u128::from(count),
            new / u128::from(count),
            slowdown * 100.0,
            spread * 100.0
        );
        slower >= 23 && slowdown > spread
    };
    assert!(
        !(measure(iterations, false) && measure(iterations.saturating_mul(4), true)),
        "reproducible paired slowdown beyond the A/A spread for {label}"
    );
}
