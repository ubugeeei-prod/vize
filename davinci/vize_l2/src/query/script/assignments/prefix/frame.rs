//! Byte bounds shared by prefix recovery and its source-frame checks.
pub(super) const fn rebase(offset: u32, old_start: u32, new_start: u32) -> u32 {
    new_start + (offset - old_start)
}

/// Upper bound on the bytes offered to the next parser after a failed cut.
pub(super) fn retry_limit(failed_end: usize, first_label: usize) -> usize {
    first_label.min(failed_end / 2)
}
