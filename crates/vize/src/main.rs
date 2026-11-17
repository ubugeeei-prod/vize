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

/// `mi_option_destroy_on_exit` in mimalloc v3 (`mimalloc.h`, 0.1.52).
const DESTROY_ON_EXIT: std::ffi::c_int = 22;

fn main() {
    // mimalloc's exit hook (`mi_process_done`) collects the main thread's heap
    // and purges every arena with `madvise` while the process is ending: 1.8 %
    // of a content-mapper session's CPU samples on macOS and 0.7 % on Linux,
    // after the last reply, freeing memory the OS reclaims anyway. A value
    // above 1 makes the hook return at once (`_mi_auto_process_done`).
    // `MIMALLOC_DESTROY_ON_EXIT` still wins.
    // SAFETY: an option store before any other thread exists.
    unsafe { mi_option_set_default(DESTROY_ON_EXIT, 2) };
    vize::cli::run_from_env();
}
