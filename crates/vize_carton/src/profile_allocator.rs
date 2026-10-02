//! Host allocator selection for allocation profiling.

use std::alloc::System;

use vize_l0::profiler::ProfilingAllocator;

/// Select the host system allocator without another allocator wrapper.
///
/// The returned value is the exact L0 accounting wrapper around [`System`].
/// Native callers selecting another allocator continue to use
/// [`ProfilingAllocator::from_allocator`] directly.
///
/// ```
/// const ALLOCATOR: vize_l0::profiler::ProfilingAllocator<std::alloc::System> =
///     vize_carton::profile_allocator::system_allocator();
/// let _ = ALLOCATOR;
/// ```
pub const fn system_allocator() -> ProfilingAllocator<System> {
    ProfilingAllocator::from_allocator(System)
}
