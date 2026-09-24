//! Total byte and slice helpers for the petite-vue document scanner.

/// The offset past the run of bytes from `pos` that satisfy `keep`.
pub(super) fn skip_while(bytes: &[u8], pos: usize, keep: impl Fn(&u8) -> bool) -> usize {
    pos + bytes
        .get(pos..)
        .map_or(0, |tail| tail.iter().take_while(|byte| keep(byte)).count())
}

/// `text[start..end]`, or empty when the range is not a char-boundary slice.
/// Every caller's bounds come from ASCII delimiters, so the fallback is
/// never observed.
pub(super) fn slice(text: &str, start: usize, end: usize) -> &str {
    text.get(start..end).unwrap_or_default()
}
