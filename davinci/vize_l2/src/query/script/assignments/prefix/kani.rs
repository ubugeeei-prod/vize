//! Standalone Kani crate for the actual checked source rebase operation.
mod frame;
use frame::rebase;

#[kani::proof]
fn moving_a_checked_source_interval_is_lossless() {
    let old: u32 = kani::any();
    let new: u32 = kani::any();
    let length: u32 = kani::any();
    let a: u32 = kani::any();
    let b: u32 = kani::any();
    kani::assume(length <= u32::MAX - old && length <= u32::MAX - new);
    kani::assume(old <= a && a <= b && b <= old + length);
    let left = rebase(a, old, new);
    let right = rebase(b, old, new);
    assert!(new <= left && left <= right && right <= new + length);
    assert_eq!(right - left, b - a);
    assert_eq!((rebase(left, new, old), rebase(right, new, old)), (a, b));
}
