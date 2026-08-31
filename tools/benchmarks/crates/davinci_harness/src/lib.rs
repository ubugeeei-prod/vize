//! Davinci bench harness (P0-1).
//!
//! Wraps criterion with the memory metric families the Davinci budgets gate
//! on - allocation pressure and peak RSS - and exports every bench run as
//! schema-validated JSON under `tools/benchmarks/results/davinci/`.
//!
//! The pieces:
//!
//! - [`alloc::CountingAllocator`]: `GlobalAlloc` wrapper counting
//!   allocation-like calls plus live/peak heap bytes, installed by [`main!`].
//!   Wraps mimalloc so benches run on the allocator the shipped `vize`
//!   binary uses. Reports stamp the target OS so exact peak-byte gates can
//!   select a platform-specific contract.
//! - [`rss`]: `ru_maxrss` peak-RSS sampling, normalized to bytes and always
//!   reported as a baseline-subtracted delta.
//! - [`report`]: the JSON exporter, checked against
//!   `schema/davinci-bench.schema.json` by a strict subset validator.
//! - [`bench_with_metrics`]: the per-bench entry point - criterion drives
//!   sampling, the harness computes percentiles and collects the memory
//!   metrics, and the report lands on disk.
//!
//! A bench binary opts in with:
//!
//! ```ignore
//! criterion::criterion_group!(groups, my_bench);
//! davinci_harness::main!(groups);
//! ```

// Same test-only relaxation vize_croquis uses: assertion messages render
// errors with `to_string`, which the production ban list disallows.
#![cfg_attr(test, allow(clippy::disallowed_methods))]

pub mod alloc;
pub mod fixtures;
pub mod report;
pub mod rss;
pub mod stage;

use core::cell::RefCell;
use std::time::Instant;

use criterion::Criterion;

use crate::report::{BenchReport, WallNs};

/// Re-exported for [`main!`]; bench crates still depend on criterion directly
/// for `criterion_group!`.
pub use criterion;

/// Criterion measurement-phase sample count used by [`bench_with_metrics`].
///
/// The wall percentiles are computed over exactly this many samples (fewer
/// only in criterion's reduced modes such as `--test`), so two runs of the
/// same bench always summarize the same sample shape.
pub const SAMPLE_SIZE: usize = 60;

/// Nearest-rank percentile over ascending-sorted per-iteration samples.
///
/// `q` is the percentile rank in `[0, 1]`; `q = 0.0` selects the minimum.
/// The result is rounded to whole nanoseconds.
pub fn percentile_ns(sorted_ns: &[f64], q: f64) -> u64 {
    assert!(
        !sorted_ns.is_empty(),
        "percentile over an empty sample set is undefined"
    );
    assert!(
        (0.0..=1.0).contains(&q),
        "percentile rank must be within [0, 1]"
    );
    let rank = (q * sorted_ns.len() as f64).ceil() as usize;
    let index = rank.max(1) - 1;
    sorted_ns[index.min(sorted_ns.len() - 1)].round() as u64
}

/// Run one bench through criterion and export the Davinci metric report.
///
/// Criterion owns the sampling strategy (via `iter_custom`); the harness
/// records per-iteration wall times, reduces the measurement-phase samples to
/// p50/p95, runs one instrumented pass for allocation metrics, reads the
/// RSS delta, and writes `tools/benchmarks/results/davinci/<bench_id>.json`.
///
/// The routine should return the value it computes so the harness can
/// `black_box` it; wall timings include the counting allocator's overhead,
/// which is constant across baseline and candidate runs.
///
/// Panics if the report cannot be written or `bench_id` is not filename-safe:
/// a bench that cannot record its result is a failed bench, not a warning.
pub fn bench_with_metrics<T>(
    criterion: &mut Criterion,
    bench_id: &str,
    fixture: &str,
    mut routine: impl FnMut() -> T,
) {
    report::validate_bench_id(bench_id).expect("bench_id must be filename-safe");

    let samples = RefCell::new(Vec::<f64>::new());
    let mut group = criterion.benchmark_group(bench_id);
    group.sample_size(SAMPLE_SIZE);
    group.bench_function("run", |bencher| {
        bencher.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                core::hint::black_box(routine());
            }
            let elapsed = start.elapsed();
            let per_iter_ns = elapsed.as_nanos() as f64 / iters as f64;
            samples.borrow_mut().push(per_iter_ns);
            elapsed
        });
    });
    group.finish();

    let mut all_samples = samples.into_inner();
    if all_samples.is_empty() {
        // Criterion modes that never execute the routine (`--list`) have
        // nothing to report.
        return;
    }
    // Criterion also invokes the routine during warmup; only the trailing
    // measurement-phase samples enter the percentiles.
    let tail_start = all_samples.len().saturating_sub(SAMPLE_SIZE);
    let mut measured = all_samples.split_off(tail_start);
    measured.sort_by(|left, right| left.total_cmp(right));

    let wall_ns = WallNs {
        p50: percentile_ns(&measured, 0.50),
        p95: percentile_ns(&measured, 0.95),
    };
    let alloc_metrics = alloc::measure(|| core::hint::black_box(routine()));
    let report = BenchReport {
        bench_id,
        fixture,
        platform: std::env::consts::OS,
        wall_ns,
        allocs: alloc_metrics.map(|metrics| metrics.calls),
        alloc_bytes_peak: alloc_metrics.map(|metrics| metrics.peak_bytes_over_start),
        rss_peak_bytes: rss::delta_since_baseline(),
        harness_version: report::HARNESS_VERSION,
    };
    let path = report::write(&report).expect("davinci bench report must be written");
    eprintln!("davinci-harness: wrote {}", path.display());
}

/// Define the bench binary entry point with the Davinci metric setup.
///
/// Expands to the counting global allocator (over mimalloc), the RSS baseline
/// capture, and a criterion main that runs each listed
/// `criterion_group!`-defined group - the moral equivalent of
/// `criterion_main!` with the harness installed.
#[macro_export]
macro_rules! main {
    ($($group:path),+ $(,)?) => {
        #[global_allocator]
        static DAVINCI_HARNESS_GLOBAL_ALLOC: $crate::alloc::CountingAllocator =
            $crate::alloc::CountingAllocator::mimalloc();

        fn main() {
            $crate::alloc::mark_installed();
            $crate::rss::capture_baseline();
            $($group();)+
            $crate::criterion::Criterion::default()
                .configure_from_args()
                .final_summary();
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentiles_are_nearest_rank_exact() {
        let samples: Vec<f64> = (1..=100).map(f64::from).collect();
        assert_eq!(percentile_ns(&samples, 0.50), 50);
        assert_eq!(percentile_ns(&samples, 0.95), 95);
        assert_eq!(percentile_ns(&samples, 0.0), 1);
        assert_eq!(percentile_ns(&samples, 1.0), 100);

        let single = [42.0];
        assert_eq!(percentile_ns(&single, 0.50), 42);
        assert_eq!(percentile_ns(&single, 0.95), 42);

        // Sixty samples - the harness sample size - with rank arithmetic on
        // the boundary: ceil(0.5 * 60) = 30 -> index 29; ceil(0.95 * 60) = 57.
        let sixty: Vec<f64> = (1..=60).map(f64::from).collect();
        assert_eq!(percentile_ns(&sixty, 0.50), 30);
        assert_eq!(percentile_ns(&sixty, 0.95), 57);
    }

    #[test]
    fn percentile_rounds_to_whole_nanoseconds() {
        let samples = [1.4, 2.5, 3.6];
        assert_eq!(percentile_ns(&samples, 0.0), 1);
        assert_eq!(percentile_ns(&samples, 0.5), 3);
        assert_eq!(percentile_ns(&samples, 1.0), 4);
    }
}
