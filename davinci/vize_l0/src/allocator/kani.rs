//! Model-check the arena-lifetime borrow `alloc_owned` hands out.
//!
//! Law: a value parked by `alloc_owned` stays readable, unchanged, through the
//! returned reference while the allocator is borrowed, even after later parks
//! grow (and relocate) the vector of boxes; `reset` then frees the parked
//! values and the allocator parks again. Kani checks every dereference of the
//! returned references for liveness and bounds, and the frees for double or
//! invalid release. (Kani does not model aliasing; the Miri lane
//! `cargo miri test -p vize_carton --lib -- allocator pool` covers Stacked
//! Borrows for the same entry point.)
//!
//! Bounds: three parks of distinct types with symbolic contents, then three
//! more parks, so the box vector grows past its first capacity and relocates
//! every handle while the first three references are live; one park after
//! `reset`.
#![allow(
    unexpected_cfgs,
    reason = "Kani selects its symbolic harnesses with cfg(kani)"
)]

#[cfg(kani)]
use super::Allocator;

#[cfg(kani)]
#[kani::proof]
#[kani::unwind(10)]
fn parked_values_outlive_vector_growth_until_reset() {
    let mut allocator = Allocator::new();
    let first: u64 = kani::any();
    let second: [u8; 3] = kani::any();
    let third: (u16, bool) = kani::any();
    {
        let a = allocator.alloc_owned(first);
        let b = allocator.alloc_owned(second);
        let c = allocator.alloc_owned(third);
        // Fill past the first capacity so the box handles relocate.
        for _ in 0..3 {
            let _ = allocator.alloc_owned(0u8);
        }
        assert_eq!(*a, first);
        assert_eq!(*b, second);
        assert_eq!(*c, third);
        kani::cover!(*a == u64::MAX && b[2] == 7, "symbolic parked contents");
    }
    allocator.reset();
    let again = allocator.alloc_owned(third);
    assert_eq!(*again, third);
}
