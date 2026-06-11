//! Native CLI binary for Vize.

#[cfg(not(feature = "profiling"))]
#[global_allocator]
static GLOBAL_ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(feature = "profiling")]
#[global_allocator]
static GLOBAL_ALLOCATOR: vize_carton::profiler::ProfilingAllocator<mimalloc::MiMalloc> =
    vize_carton::profiler::ProfilingAllocator::from_allocator(mimalloc::MiMalloc);

fn main() {
    vize::cli::run_from_env();
}
