//! Stack headroom for the compiler's recursive AST passes.
//!
//! Vize's middle end walks the template AST with the machine stack: the
//! transform lane recurses through `traverse_node`, static hoisting recurses
//! through its subtree predicates and `hoist_static_inner`, and codegen recurses
//! through `generate_node`. Nesting depth in the template is therefore depth on
//! the call stack.
//!
//! That matters because a Rust stack overflow is not a recoverable error. The
//! guard page faults, the runtime prints `fatal runtime error: stack overflow`,
//! and the process is aborted with `SIGABRT` — taking down a `vize` CLI run, an
//! LSP session, or a test binary with it. There is no `catch_unwind` for it and
//! no way to turn it into a diagnostic after the fact. A depth limit chosen to
//! stay under the stack is not a fix either: it only moves the abort to a
//! different input, and it caps the templates Vize accepts far below what
//! `@vue/compiler-dom` accepts (measured: ~1092 levels of `<div>` on a default
//! Node stack, after which Vue raises a *catchable* `RangeError`).
//!
//! [`ensure_sufficient_stack`] closes that gap. Called at the point where a
//! recursive pass steps from a node to its children, it checks the remaining
//! stack and, when the headroom drops below [`RED_ZONE`], moves the rest of the
//! recursion onto a freshly allocated [`SEGMENT_SIZE`] stack. Depth then costs
//! heap, which is recoverable and bounded by the parser's nesting limit, instead
//! of stack, which is not.
//!
//! # Why not an explicit work stack
//!
//! An iterative traversal with a heap-allocated worklist is the cleaner shape
//! and remains the long-term direction for the transform lane, whose recursion
//! is confined to one function pair. It is not viable for the whole middle end
//! today: codegen interleaves emission with recursion at ~20 call sites across
//! ten modules (`generate_element` emits an opening call, recurses into
//! children, then emits the closing arguments), so making it iterative means a
//! continuation-passing rewrite of every one of those sites. Static hoisting
//! adds four more mutually recursive walks. Guarding the recursion covers all
//! three passes with one mechanism and no behavioural risk, and it is the same
//! answer `rustc` reached for the same problem
//! (`rustc_data_structures::stack::ensure_sufficient_stack`).
//!
//! # Cost
//!
//! On the shallow templates that dominate real projects the guard never grows
//! anything: it is one thread-local read plus a pointer comparison per guarded
//! call, and no segment is ever allocated. Growth only happens once per
//! [`SEGMENT_SIZE`] of consumed stack, so a pathologically deep template pays a
//! handful of stack-segment allocations in total.

/// Stack headroom that must remain before a guarded call proceeds in place.
///
/// This has to exceed the stack a single guarded step can consume before it
/// reaches the next guard. The deepest such step is one template nesting level
/// of the full pipeline, measured at ~8 KiB in a debug build (the profile with
/// the largest frames, since it keeps every temporary live); 512 KiB leaves two
/// orders of magnitude of slack for the unguarded helper frames in between.
const RED_ZONE: usize = 512 * 1024;

/// Size of the stack allocated when [`RED_ZONE`] is breached.
///
/// Large enough that growth is rare (~64 debug-build nesting levels per
/// allocation at the measured 8 KiB/level, ~500 in release), small enough that
/// the allocation is never a meaningful cost next to the compile it enables.
const SEGMENT_SIZE: usize = 4 * 1024 * 1024;

/// Run `f`, first moving to a fresh stack segment if the current stack is close
/// to exhausted.
///
/// Call this where a recursive pass descends one level — the point whose
/// repetition is bounded by input nesting depth rather than by the code. Adding
/// it to a leaf helper only costs a check; omitting it from a recursion step is
/// what reintroduces the abort.
#[inline]
pub fn ensure_sufficient_stack<R>(f: impl FnOnce() -> R) -> R {
    stacker::maybe_grow(RED_ZONE, SEGMENT_SIZE, f)
}

#[cfg(test)]
mod tests {
    use super::ensure_sufficient_stack;

    /// A frame big enough that unguarded recursion overflows a small stack long
    /// before the recursion count below is reached.
    fn deep(n: usize) -> usize {
        ensure_sufficient_stack(|| {
            let padding = [0u64; 512];
            if n == 0 {
                return std::hint::black_box(&padding).len();
            }
            std::hint::black_box(&padding);
            deep(n - 1)
        })
    }

    #[test]
    fn recursion_far_past_the_thread_stack_completes() {
        // 20_000 frames of >=4 KiB each need >=80 MiB; the thread below has
        // 256 KiB, so this can only pass by growing onto the heap.
        let handle = std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(|| deep(20_000))
            .expect("spawn");
        assert_eq!(handle.join().expect("join"), 512);
    }
}
