//! Verify the arithmetic used to parse a checked prefix of a damaged script.
//! This is the library root of the separate Creusot workspace.
mod frame;

use creusot_std::prelude::*;

/// The caller has two ordered endpoints in a source block that fits in its
/// new file. Relocating both preserves length, order and the inverse mapping.
#[requires(old_start@ <= left@ && left@ <= right@)]
#[requires(new_start@ + right@ - old_start@ <= u32::MAX@)]
#[ensures(new_start@ <= result.0@ && result.0@ <= result.1@)]
#[ensures(result.1@ - result.0@ == right@ - left@)]
pub fn relocate_interval(left: u32, right: u32, old_start: u32, new_start: u32) -> (u32, u32) {
    let start = frame::rebase(left, old_start, new_start);
    let end = frame::rebase(right, old_start, new_start);
    let original_left = frame::rebase(start, new_start, old_start);
    let original_right = frame::rebase(end, new_start, old_start);
    proof_assert!(original_left == left && original_right == right);
    (start, end)
}

/// Each pair supplies a proposed cut and the first diagnostic offset. The
/// cold-open loop can select any earlier separator and stop after any attempt.
/// Its initial full-file parse is charged separately.
#[ensures(result@ <= 2 * first_limit@)]
pub fn offered_prefix_bytes(first_limit: usize, attempts: &[(usize, usize)]) -> u128 {
    let mut limit = first_limit;
    let mut spent = 0_u128;
    let mut i = 0;
    #[invariant(i@ <= attempts@.len())]
    #[invariant(spent@ + 2 * limit@ <= 2 * first_limit@)]
    #[variant(attempts@.len() - i@)]
    while i < attempts.len() {
        let (cut, first_label) = attempts[i];
        let end = cut.min(limit);
        spent += end as u128;
        limit = frame::retry_limit(end, first_label);
        i += 1;
    }
    spent
}

/// A cold retry fits the remaining byte budget: the next limit never exceeds
/// the first diagnostic offset, is below a nonzero failed end, and spending
/// `end` bytes now while reserving twice the next limit for every later
/// attempt stays within twice `end`. The invariant of `offered_prefix_bytes`
/// is this step, for any sequence of attempts.
pub fn cold_retries_fit_the_remaining_byte_budget(end: u32, first_label: u32) {
    let next = frame::retry_limit(end as usize, first_label as usize);
    proof_assert!(next@ <= first_label@);
    proof_assert!(end@ == 0 || next@ < end@);
    proof_assert!(end@ + 2 * next@ <= 2 * end@);
}

/// Moving a checked source interval into a new file that holds the whole
/// block keeps it inside the block, keeps its length, and moves back.
#[requires(length@ <= u32::MAX@ - old_start@ && length@ <= u32::MAX@ - new_start@)]
#[requires(old_start@ <= left@ && left@ <= right@ && right@ <= old_start@ + length@)]
pub fn moving_a_checked_source_interval_is_lossless(
    old_start: u32,
    new_start: u32,
    length: u32,
    left: u32,
    right: u32,
) {
    let start = frame::rebase(left, old_start, new_start);
    let end = frame::rebase(right, old_start, new_start);
    proof_assert!(new_start@ <= start@ && start@ <= end@ && end@ <= new_start@ + length@);
    proof_assert!(end@ - start@ == right@ - left@);
    let back_start = frame::rebase(start, new_start, old_start);
    let back_end = frame::rebase(end, new_start, old_start);
    proof_assert!(back_start == left && back_end == right);
}
