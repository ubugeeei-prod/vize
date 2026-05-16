//! Native CLI binary for Vize.

#[global_allocator]
static GLOBAL_ALLOCATOR: vize_carton::profiler::ProfilingAllocator =
    vize_carton::profiler::ProfilingAllocator::new();

fn main() {
    vize::cli::run_from_env();
}
