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
