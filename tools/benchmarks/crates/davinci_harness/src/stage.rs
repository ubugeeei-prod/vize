//! Stage-scoped measurement (P0-3).
//!
//! Several pipeline stages mutate their input in place (`transform` takes
//! `&mut RootNode` and the AST has no `Clone`), so a bench iteration must
//! rebuild its input every time - but the rebuild must stay outside the
//! numbers. [`bench_stage_with_metrics`] runs a whole iteration closure per
//! criterion iteration and only accounts the section the closure wraps in
//! [`StageWindow::measure`]:
//!
//! ```ignore
//! davinci_harness::stage::bench_stage_with_metrics(c, "core_transform_small", path, |window| {
//!     let allocator = Allocator::new();                  // setup: not measured
//!     let (mut root, _) = parse(&allocator, template);   // setup: not measured
//!     window.measure(|| transform(&allocator, &mut root, options, None))
//! });
//! ```
//!
//! Exactly one `measure` call per iteration is enforced (a second call
//! panics, and an iteration that never measured panics at the end of the
//! criterion pass): a silent zero-width window would report a fantasy
//! number, and summing disjoint windows would make the peak-bytes metric
//! meaningless.
//!
//! Criterion's own reported time equals the exported wall metric here - the
//! harness returns the summed window durations to `iter_custom`, so the
//! console output and the JSON never diverge.

use core::cell::RefCell;
use std::time::{Duration, Instant};

use criterion::Criterion;

use crate::report::{BenchReport, WallNs};
use crate::{SAMPLE_SIZE, alloc, percentile_ns, report, rss};

enum WindowMode {
    Timing {
        elapsed: Duration,
    },
    Alloc {
        metrics: Option<alloc::AllocMetrics>,
    },
}

/// The measured-section handle passed to a stage iteration closure.
pub struct StageWindow {
    mode: WindowMode,
    uses: u32,
}

impl StageWindow {
    fn timing() -> Self {
        Self {
            mode: WindowMode::Timing {
                elapsed: Duration::ZERO,
            },
            uses: 0,
        }
    }

    fn alloc_probe() -> Self {
        Self {
            mode: WindowMode::Alloc { metrics: None },
            uses: 0,
        }
    }

    /// Run the stage under measurement and hand its value back.
    ///
    /// Panics on a second call within the same iteration.
    pub fn measure<T>(&mut self, stage: impl FnOnce() -> T) -> T {
        self.uses += 1;
        assert_eq!(
            self.uses, 1,
            "a stage iteration must contain exactly one measured section"
        );
        match &mut self.mode {
            WindowMode::Timing { elapsed } => {
                let start = Instant::now();
                let value = stage();
                *elapsed = start.elapsed();
                value
            }
            WindowMode::Alloc { metrics } => {
                let (value, measured) = alloc::measure_returning(stage);
                *metrics = measured;
                value
            }
        }
    }

    fn finish_timing(self) -> Duration {
        assert_eq!(
            self.uses, 1,
            "a stage iteration must call StageWindow::measure exactly once"
        );
        match self.mode {
            WindowMode::Timing { elapsed } => elapsed,
            WindowMode::Alloc { .. } => unreachable!("timing pass uses timing windows"),
        }
    }

    fn finish_alloc(self) -> Option<alloc::AllocMetrics> {
        assert_eq!(
            self.uses, 1,
            "a stage iteration must call StageWindow::measure exactly once"
        );
        match self.mode {
            WindowMode::Alloc { metrics } => metrics,
            WindowMode::Timing { .. } => unreachable!("alloc pass uses alloc windows"),
        }
    }
}

/// [`crate::bench_with_metrics`] for stages that need per-iteration setup:
/// only the section wrapped by [`StageWindow::measure`] enters the wall and
/// allocation metrics. The RSS delta stays process-scoped as everywhere else.
pub fn bench_stage_with_metrics<T>(
    criterion: &mut Criterion,
    bench_id: &str,
    fixture: &str,
    mut iteration: impl FnMut(&mut StageWindow) -> T,
) {
    report::validate_bench_id(bench_id).expect("bench_id must be filename-safe");

    let samples = RefCell::new(Vec::<f64>::new());
    let mut group = criterion.benchmark_group(bench_id);
    group.sample_size(SAMPLE_SIZE);
    group.bench_function("run", |bencher| {
        bencher.iter_custom(|iters| {
            let mut total = Duration::ZERO;
            for _ in 0..iters {
                let mut window = StageWindow::timing();
                core::hint::black_box(iteration(&mut window));
                total += window.finish_timing();
            }
            let per_iter_ns = total.as_nanos() as f64 / iters as f64;
            samples.borrow_mut().push(per_iter_ns);
            total
        });
    });
    group.finish();

    let mut all_samples = samples.into_inner();
    if all_samples.is_empty() {
        // Criterion modes that never execute the routine (`--list`).
        return;
    }
    let tail_start = all_samples.len().saturating_sub(SAMPLE_SIZE);
    let mut measured = all_samples.split_off(tail_start);
    measured.sort_by(|left, right| left.total_cmp(right));

    let wall_ns = WallNs {
        p50: percentile_ns(&measured, 0.50),
        p95: percentile_ns(&measured, 0.95),
    };
    let mut window = StageWindow::alloc_probe();
    core::hint::black_box(iteration(&mut window));
    let alloc_metrics = window.finish_alloc();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timing_window_accounts_only_the_measured_section() {
        let mut window = StageWindow::timing();
        // Setup outside the window: sleep-free but observable ordering - the
        // measured section is the only part that can contribute time.
        let value = window.measure(|| {
            let mut acc = 0u64;
            for i in 0..1000u64 {
                acc = acc.wrapping_add(core::hint::black_box(i));
            }
            acc
        });
        assert_eq!(value, 499_500);
        let elapsed = window.finish_timing();
        assert!(elapsed > Duration::ZERO);
    }

    #[test]
    #[should_panic(expected = "exactly one measured section")]
    fn second_measure_call_panics() {
        let mut window = StageWindow::timing();
        window.measure(|| ());
        window.measure(|| ());
    }

    #[test]
    #[should_panic(expected = "exactly once")]
    fn unmeasured_iteration_panics_at_finish() {
        let window = StageWindow::timing();
        let _ = window.finish_timing();
    }
}
