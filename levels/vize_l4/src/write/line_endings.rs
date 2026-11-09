//! JavaScript line terminators, with CRLF counted as one boundary.

#[inline]
pub(crate) fn line_break_len(text: &str, offset: usize) -> usize {
    match text.as_bytes().get(offset..) {
        Some([b'\r', b'\n', ..]) => 2,
        Some([b'\r' | b'\n', ..]) => 1,
        Some([0xe2, 0x80, 0xa8 | 0xa9, ..]) => 3,
        _ => 0,
    }
}
