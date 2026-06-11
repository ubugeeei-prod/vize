//! Lightweight profiling utilities for performance monitoring.
//!
//! Provides simple timing and metrics collection for tracking
//! type checking and compilation performance.

mod allocation;
mod cache;
mod core;
mod metrics;
mod report;

pub use allocation::{
    AllocationSnapshot, ProfilingAllocator, allocation_snapshot, reset_allocation_counters,
    set_allocation_tracking_enabled,
};
pub use cache::CacheStats;
pub use core::{ProfileGuard, Profiler, Timer, global_profiler};
pub use metrics::{CounterMetrics, Metrics};
pub use report::{CounterEntry, CounterSummary, ProfileEntry, ProfileSummary};

/// Macro for profiling a block of code.
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
}

#[cfg(test)]
mod tests;
