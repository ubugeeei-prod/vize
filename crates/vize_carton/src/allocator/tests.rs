use std::sync::atomic::{AtomicUsize, Ordering};

use super::Allocator;
use crate::{Box, Vec};

#[test]
fn test_allocator_new() {
    let allocator = Allocator::new();
    assert_eq!(allocator.allocated_bytes(), 0);
}

#[test]
fn test_allocator_default() {
    let allocator = Allocator::default();
    assert_eq!(allocator.allocated_bytes(), 0);
}

#[test]
fn test_alloc_str() {
    let allocator = Allocator::new();
    let s = allocator.alloc_str("hello world");
    assert_eq!(s, "hello world");
}

#[test]
fn test_oxc_pool_allocates_and_counts() {
    let allocator = Allocator::new();
    let s = allocator.as_oxc().alloc_str("retained");
    assert_eq!(s, "retained");
    assert_ne!(allocator.allocated_bytes(), 0);
}

#[test]
fn test_containers_and_strings_share_the_handle_lifetime() {
    let allocator = Allocator::new();
    let allocator = &allocator;
    let from_str = allocator.alloc_str("string side");
    let from_oxc = allocator.as_oxc().alloc_str("retained side");
    let boxed = Box::new_in(7_u32, &allocator);
    // All three references are alive together under the same borrow of
    // `allocator` — the single-lifetime contract P1-5 builds on.
    assert_eq!(
        (from_str, from_oxc, *boxed),
        ("string side", "retained side", 7)
    );
}

#[test]
fn test_reset_clears_the_pool() {
    let mut allocator = Allocator::new();
    {
        let alloc = &allocator;
        let _ = alloc.alloc_str("hello");
        let mut v: Vec<'_, u32> = Vec::new_in(&alloc);
        v.push(1);
    }
    assert_ne!(allocator.allocated_bytes(), 0);
    allocator.reset();
    assert_eq!(allocator.allocated_bytes(), 0);
}

/// Drop witness for the parked-value tests: a park must run the real
/// destructor at reset, which is exactly what `ManuallyDrop` parking did not.
/// Each test counts into its own static, so the tests stay independent under
/// libtest's parallel runner.
struct DropWitness {
    drops: &'static AtomicUsize,
    payload: u32,
}

impl Drop for DropWitness {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::Relaxed);
    }
}

/// P1-11: a parked owned value is readable for the whole compile and its
/// destructor runs at reset — the per-compile leak the arena park left behind.
#[test]
fn test_alloc_owned_borrows_for_the_compile_and_drops_at_reset() {
    static DROPS: AtomicUsize = AtomicUsize::new(0);

    let mut allocator = Allocator::new();
    {
        let parked = allocator.alloc_owned(DropWitness {
            drops: &DROPS,
            payload: 42,
        });
        assert_eq!(parked.payload, 42);
        assert_eq!(DROPS.load(Ordering::Relaxed), 0);
    }
    allocator.reset();
    assert_eq!(DROPS.load(Ordering::Relaxed), 1);
}

/// Dropping the allocator runs the parked destructors too, so a compile that
/// never resets does not leak either.
#[test]
fn test_alloc_owned_drops_with_the_allocator() {
    static DROPS: AtomicUsize = AtomicUsize::new(0);

    {
        let allocator = Allocator::new();
        let parked = allocator.alloc_owned(DropWitness {
            drops: &DROPS,
            payload: 7,
        });
        assert_eq!(parked.payload, 7);
        assert_eq!(DROPS.load(Ordering::Relaxed), 0);
    }
    assert_eq!(DROPS.load(Ordering::Relaxed), 1);
}

/// Parking twice keeps both values readable: the vector of boxes grows, and
/// growth moves the box handles, never the values they point at.
#[test]
fn test_alloc_owned_survives_further_parks() {
    let allocator = Allocator::new();
    let first: &crate::String = allocator.alloc_owned(crate::String::from("first"));
    let mut parked = std::vec::Vec::new();
    for index in 0..32_usize {
        let name = match index % 2 {
            0 => "even",
            _ => "odd",
        };
        parked.push(allocator.alloc_owned(crate::String::from(name)));
    }
    assert_eq!(first.as_str(), "first");
    assert_eq!(parked[31].as_str(), "odd");
}

/// P1-11 escape check: a stamp taken before a reset is stale afterwards, and
/// validating it panics instead of reading recycled bytes.
#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "arena-backed value outlived its compile")]
fn test_stale_stamp_panics_after_reset() {
    let mut allocator = Allocator::new();
    let stamp = allocator.stamp();
    allocator.assert_stamp_current(stamp);
    allocator.reset();
    allocator.assert_stamp_current(stamp);
}

/// The counterpart: without a reset the stamp stays current, so the check
/// cannot be satisfied by panicking on everything.
#[test]
fn test_stamp_stays_current_without_a_reset() {
    let allocator = Allocator::new();
    let stamp = allocator.stamp();
    let _ = allocator.alloc_str("allocation alone is not a generation change");
    allocator.assert_stamp_current(stamp);
    assert_eq!(stamp, allocator.stamp());
}

/// Two arenas are distinguishable, so a stamp cannot be validated against the
/// wrong arena of the same generation.
#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "arena-backed value outlived its compile")]
fn test_stamp_from_another_arena_panics() {
    let first = Allocator::new();
    let second = Allocator::new();
    second.assert_stamp_current(first.stamp());
}
