//! Lightweight profiling utilities for performance monitoring.
//!
//! Provides simple timing and metrics collection for tracking
//! type checking and compilation performance.

mod allocation;
mod attribution;
mod cache;
mod core;
mod counters;
mod export;
mod metrics;
mod report;
mod snapshot;

pub use allocation::{
    AllocationSnapshot, ProfilingAllocator, allocation_snapshot, reset_allocation_counters,
    set_allocation_tracking_enabled,
};
pub use attribution::{SpanAttribution, SpanRange};
pub use cache::CacheStats;
pub use core::{ProfileGuard, Profiler, Timer, global_profiler};
pub use export::{
    PROFILE_EXPORT_SCHEMA_VERSION, ProfileExport, ProfileExportAllocCounts,
    ProfileExportAllocation, ProfileExportAttribution, ProfileExportBudget, ProfileExportCounter,
    ProfileExportOptions, ProfileExportSpan, ProfileExportSpanRange, ProfileExportTruncation,
    ProfileExportWallNs,
};
pub use metrics::{CounterMetrics, Metrics};
pub use report::{CounterEntry, CounterSummary, ProfileEntry, ProfileSummary};

/// Macro for profiling a block of code.
///
/// The two-argument form records under the plain dotted key. The `attr:` form
/// additionally carries a [`SpanAttribution`](crate::profiler::SpanAttribution),
/// recording under its own `key × attribution` bucket:
///
/// ```
/// use vize_carton::profile;
/// use vize_carton::profiler::SpanAttribution;
///
/// let plain = profile!("davinci.demo.plain", 1 + 1);
/// let attributed = profile!(
///     "davinci.demo.pass",
///     attr: SpanAttribution::new().with_stage("s1").with_pass("demo"),
///     1 + 1
/// );
/// assert_eq!(plain + attributed, 4);
/// ```
#[macro_export]
macro_rules! profile {
    ($name:expr, $block:expr) => {{
        let name: &'static str = $name;
        let profiler = $crate::profiler::global_profiler();
        // Keep disabled profiling cheap enough to leave at fine-grained call
        // sites: one relaxed atomic check, then the original block executes.
        if profiler.is_enabled() {
            let _profile_guard = profiler.global_span(name);
            $block
        } else {
            $block
        }
    }};
    ($name:expr, attr: $attribution:expr, $block:expr) => {{
        let name: &'static str = $name;
        let profiler = $crate::profiler::global_profiler();
        // Same disabled-cost contract as the unattributed arm: one relaxed
        // atomic check, then the original block executes.
        if profiler.is_enabled() {
            let _profile_guard = profiler.global_span_attributed(name, $attribution);
            $block
        } else {
            $block
        }
    }};
}

#[cfg(test)]
mod tests;
