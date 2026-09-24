//! Total char access for the output normalizers.

/// The char at `i`, or NUL past the end (every caller bounds-checks `i`).
pub(crate) fn ch_at(chars: &[char], i: usize) -> char {
    chars.get(i).copied().unwrap_or('\0')
}
