//! Total sub-slicing of the one source.
//!
//! Every S1 string is a slice of the parsed source, and spans are recovered
//! from pointers. Offsets the builders compute sit on ASCII delimiters, so
//! these ranges are always char-boundary ranges of `src`; should one not be,
//! the fallback is still a (zero-width) slice of `src` rather than a panic or
//! a foreign string.

/// `src[start..end]`, or the empty slice at `src`'s start.
pub(crate) fn range(src: &str, start: usize, end: usize) -> &str {
    src.get(start..end).unwrap_or_else(|| empty(src))
}

/// `src[start..]`, or the empty slice at `src`'s start.
pub(crate) fn from(src: &str, start: usize) -> &str {
    src.get(start..).unwrap_or_else(|| empty(src))
}

/// The zero-width slice at `src`'s start.
fn empty(src: &str) -> &str {
    src.split_at_checked(0).map_or(src, |(empty, _)| empty)
}
