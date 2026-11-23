//! Byte bounds shared by prefix recovery and its source-frame checks.
#[cfg(creusot)]
use creusot_std::prelude::*;

#[cfg_attr(creusot, requires(old_start@ <= offset@))]
#[cfg_attr(creusot, requires(new_start@ + offset@ - old_start@ <= u32::MAX@))]
#[cfg_attr(creusot, ensures(result@ == new_start@ + offset@ - old_start@))]
pub(super) const fn rebase(offset: u32, old_start: u32, new_start: u32) -> u32 {
    new_start + (offset - old_start)
}

/// Upper bound on the bytes offered to the next parser after a failed cut.
#[cfg_attr(creusot, ensures(result@ <= first_label@))]
#[cfg_attr(creusot, ensures(2 * result@ <= failed_end@))]
#[cfg_attr(creusot, ensures(result == first_label || result@ == failed_end@ / 2))]
pub(super) fn retry_limit(failed_end: usize, first_label: usize) -> usize {
    first_label.min(failed_end / 2)
}
