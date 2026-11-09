//! Native CLI binary for Vize.

#[cfg(not(feature = "profiling"))]
#[global_allocator]
static GLOBAL_ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(feature = "profiling")]
#[global_allocator]
static GLOBAL_ALLOCATOR: vize_l0::profiler::ProfilingAllocator<mimalloc::MiMalloc> =
    vize_l0::profiler::ProfilingAllocator::from_allocator(mimalloc::MiMalloc);

// Linked through `libmimalloc-sys`, whose bindings only declare it behind a
// feature that would add a dependency.
#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn mi_option_set_default(option: std::ffi::c_int, value: std::ffi::c_long);
}

/// mimalloc commits whole arenas up front where the OS overcommits (Linux,
/// WSL2), which multiplied peak RSS without making anything faster. It
/// reserves its first arena before `main`, so the default changes in a
/// constructor ordered between std's (99) and mimalloc's (101 or later).
/// `MIMALLOC_ARENA_EAGER_COMMIT` still wins.
#[cfg(target_os = "linux")]
#[used]
#[unsafe(link_section = ".init_array.00100")]
static COMMIT_ARENAS_ON_DEMAND: extern "C" fn() = {
    extern "C" fn commit_arenas_on_demand() {
        // `mi_option_arena_eager_commit` in mimalloc v2 and v3.
        const ARENA_EAGER_COMMIT: std::ffi::c_int = 4;
        // SAFETY: runs single-threaded, before mimalloc reads its options.
        unsafe { mi_option_set_default(ARENA_EAGER_COMMIT, 0) };
    }
    commit_arenas_on_demand
};

fn main() {
    vize::cli::run_from_env();
}
