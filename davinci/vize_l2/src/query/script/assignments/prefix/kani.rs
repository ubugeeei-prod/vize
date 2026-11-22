//! Standalone Kani crate for source relocation and the first-open retry budget.
mod frame;
use frame::rebase;

#[kani::proof]
fn cold_retries_fit_the_remaining_byte_budget() {
    let end: u32 = kani::any();
    let first_label: u32 = kani::any();
    let next = frame::retry_limit(end as usize, first_label as usize) as u64;
    assert!(next <= u64::from(first_label));
    assert!(end == 0 || next < u64::from(end));
    // Spend 'end' bytes on this attempt and reserve twice the next limit
    // for every descendant attempt. The invariant holds for any sequence.
    assert!(u64::from(end) + 2 * next <= 2 * u64::from(end));
}

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
