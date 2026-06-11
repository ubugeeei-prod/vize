//! CLI profile helpers whose behavior depends on binary build features.

use vize_carton::profiler::AllocationSnapshot;

#[inline]
pub(crate) fn allocation_snapshot() -> Option<AllocationSnapshot> {
    #[cfg(feature = "profiling")]
    {
        Some(vize_carton::profiler::allocation_snapshot())
    }

    #[cfg(not(feature = "profiling"))]
    {
        None
    }
}
