//! The same offset map is used for names, type annotations and initializers.
pub(super) const fn rebase(offset: u32, old_start: u32, new_start: u32) -> u32 {
    new_start + (offset - old_start)
}
