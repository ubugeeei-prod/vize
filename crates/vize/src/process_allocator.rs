//! mimalloc defaults for the artifacts that install it as the process
//! allocator: the CLI binary and the Node addon.

/// Makes mimalloc commit arena memory on demand on Linux.
///
/// mimalloc commits whole arenas up front where the OS overcommits (Linux,
/// WSL2), which multiplied peak RSS without making anything faster (#12358).
/// It reserves its first arena while it initializes, before `main` or, in a
/// shared object, while the object loads, so the default changes in a
/// constructor ordered between std's (99) and mimalloc's (101 or later).
/// `MIMALLOC_ARENA_EAGER_COMMIT` still wins.
///
/// Expand it once, in the crate that links the artifact: a constructor in a
/// dependency's object file is dropped unless something else pulls that
/// object in.
#[macro_export]
#[doc(hidden)]
macro_rules! commit_mimalloc_arenas_on_demand {
    () => {
        // Linked through `libmimalloc-sys`, whose bindings only declare it
        // behind a feature that would add a dependency.
        #[cfg(target_os = "linux")]
        unsafe extern "C" {
            fn mi_option_set_default(option: ::std::ffi::c_int, value: ::std::ffi::c_long);
        }

        #[cfg(target_os = "linux")]
        #[used]
        #[unsafe(link_section = ".init_array.00100")]
        static COMMIT_ARENAS_ON_DEMAND: extern "C" fn() = {
            extern "C" fn commit_arenas_on_demand() {
                // `mi_option_arena_eager_commit` in mimalloc v2 and v3.
                const ARENA_EAGER_COMMIT: ::std::ffi::c_int = 4;
                // SAFETY: runs single-threaded, before mimalloc reads its options.
                unsafe { mi_option_set_default(ARENA_EAGER_COMMIT, 0) };
            }
            commit_arenas_on_demand
        };
    };
}
