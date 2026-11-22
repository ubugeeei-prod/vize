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
            let first = (round % 2 == 0) != reverse;
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
