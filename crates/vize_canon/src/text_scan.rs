//! Bounds-checked byte-scanning helpers shared by the hand-written scanners.
//!
//! The scanners walk source text by byte offset. These helpers answer the
//! common "what is at / before this offset" questions without indexing, so an
//! offset that runs past the end reads as "nothing there" instead of aborting.

/// Whether `text` continues with `prefix` at byte offset `at`.
///
/// Compares bytes, so an `at` that is not a character boundary simply fails
/// to match.
#[inline]
pub(crate) fn starts_with_at(text: &str, at: usize, prefix: &str) -> bool {
    text.as_bytes()
        .get(at..)
        .is_some_and(|rest| rest.starts_with(prefix.as_bytes()))
}

/// The byte immediately before offset `at`, if any.
#[inline]
pub(crate) fn byte_before(bytes: &[u8], at: usize) -> Option<u8> {
    at.checked_sub(1).and_then(|prev| bytes.get(prev)).copied()
}
