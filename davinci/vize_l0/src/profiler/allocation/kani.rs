//! Model-check that the profiling `GlobalAlloc` wrapper forwards the
//! allocator contract unchanged.
//!
//! Law: with tracking on or off, the wrapper returns the wrapped allocator's
//! block for `alloc`/`alloc_zeroed`, keeps a reallocated block's prefix, and
//! releases it with the caller's layout; counters move only after the inner
//! call returned. Kani checks every write/read of the returned blocks, that
//! `alloc_zeroed` blocks read as zero, and that `dealloc` releases a live block
//! exactly once.
//!
//! Bounds: sizes 1..=8 bytes, alignments 1, 2, 4 and 8 (the `System`
//! malloc path), reallocation to 1..=12 bytes, tracking flag symbolic.
#![allow(
    unexpected_cfgs,
    reason = "Kani selects its symbolic harnesses with cfg(kani)"
)]

#[cfg(kani)]
use super::{ProfilingAllocator, allocation_snapshot, set_allocation_tracking_enabled};
#[cfg(kani)]
use std::alloc::{GlobalAlloc, Layout, System};

#[cfg(kani)]
fn any_layout() -> Layout {
    let size: usize = kani::any();
    let shift: u8 = kani::any();
    kani::assume((1..=8).contains(&size) && shift <= 3);
    Layout::from_size_align(size, 1 << shift).expect("valid bounded layout")
}

#[cfg(kani)]
#[kani::proof]
#[kani::unwind(13)]
fn profiling_allocator_forwards_alloc_realloc_dealloc() {
    let tracking: bool = kani::any();
    set_allocation_tracking_enabled(tracking);
    let allocator = ProfilingAllocator::from_allocator(System);
    let layout = any_layout();
    let fill: u8 = kani::any();
    let new_size: usize = kani::any();
    kani::assume((1..=12).contains(&new_size));
    let before = allocation_snapshot();
    // SAFETY: `layout` has a non-zero size; the block is written only within
    // its size, reallocated with the layout it was allocated with, and freed
    // once with its current layout.
    unsafe {
        let block = allocator.alloc(layout);
        assert!(!block.is_null());
        assert_eq!(block as usize % layout.align(), 0);
        for offset in 0..layout.size() {
            block.add(offset).write(fill);
        }
        let grown = allocator.realloc(block, layout, new_size);
        assert!(!grown.is_null());
        for offset in 0..layout.size().min(new_size) {
            assert_eq!(grown.add(offset).read(), fill);
        }
        for offset in 0..new_size {
            grown.add(offset).write(fill);
        }
        let current = Layout::from_size_align(new_size, layout.align()).expect("valid layout");
        allocator.dealloc(grown, current);
    }
    let after = allocation_snapshot();
    if tracking {
        assert_eq!(after.alloc_calls, before.alloc_calls + 1);
        assert_eq!(after.realloc_calls, before.realloc_calls + 1);
        assert_eq!(after.dealloc_calls, before.dealloc_calls + 1);
    } else {
        assert_eq!(after.alloc_calls, before.alloc_calls);
    }
    kani::cover!(tracking && new_size > layout.size(), "tracked growth");
    kani::cover!(!tracking && new_size < layout.size(), "untracked shrink");
}

#[cfg(kani)]
#[kani::proof]
#[kani::unwind(9)]
fn profiling_allocator_forwards_zeroed_blocks() {
    let tracking: bool = kani::any();
    set_allocation_tracking_enabled(tracking);
    let allocator = ProfilingAllocator::from_allocator(System);
    let layout = any_layout();
    // SAFETY: non-zero layout; reads stay within the block, freed once.
    unsafe {
        let block = allocator.alloc_zeroed(layout);
        assert!(!block.is_null());
        for offset in 0..layout.size() {
            assert_eq!(block.add(offset).read(), 0);
        }
        allocator.dealloc(block, layout);
    }
    kani::cover!(tracking && layout.align() == 8, "tracked aligned zeroed block");
}
