//! Preserve raw separators while rebasing formatter-owned template layout.

use super::super::raw_mask::compute_raw_line_mask;
use super::write_line;

/// The mask describes the state at each line's start. A separator entering a
/// raw continuation is therefore raw, including the opening line's separator;
/// after the closing line, indentation and configured layout resume (#7745).
#[cold]
#[inline(never)]
pub(super) fn write(output: &mut Vec<u8>, source: &str, indent: &[u8], newline: &[u8]) {
    let bytes = source.as_bytes();
    let mut lines = Vec::new();
    let mut endings = Vec::new();
    let mut start = 0;
    while let Some(offset) = memchr::memchr2(b'\r', b'\n', bytes.get(start..).unwrap_or_default()) {
        let end = start + offset;
        let width =
            usize::from(bytes.get(end) == Some(&b'\r') && bytes.get(end + 1) == Some(&b'\n')) + 1;
        lines.push(bytes.get(start..end).unwrap_or_default());
        endings.push(bytes.get(end..end + width).unwrap_or_default());
        start = end + width;
    }
    lines.push(bytes.get(start..).unwrap_or_default());
    let raw = compute_raw_line_mask(&lines);
    for (index, line) in lines.into_iter().enumerate() {
        let separator = if raw.get(index + 1) == Some(&true) {
            endings.get(index).copied().unwrap_or(newline)
        } else {
            newline
        };
        write_line(
            output,
            line,
            indent,
            separator,
            raw.get(index) == Some(&true),
        );
    }
}
