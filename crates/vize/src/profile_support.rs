//! CLI profile helpers whose behavior depends on binary build features.

use vize_s0::profiler::AllocationSnapshot;

#[inline]
pub(crate) fn allocation_snapshot() -> Option<AllocationSnapshot> {
    #[cfg(feature = "profiling")]
    {
        Some(vize_s0::profiler::allocation_snapshot())
    }

    #[cfg(not(feature = "profiling"))]
    {
        None
    }
}
