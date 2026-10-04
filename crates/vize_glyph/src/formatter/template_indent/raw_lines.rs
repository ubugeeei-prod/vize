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

#[cfg(test)]
mod tests {
    use super::write;

    #[test]
    fn raw_boundaries_keep_authored_separators_then_resume_layout() {
        let raw = "\nfirst\r\n  second\r\n\r  third\n";
        for newline in [b"\n".as_slice(), b"\r\n", b"\r"] {
            for (open, close) in [
                ("<pre>", "</pre>"),
                ("<textarea>", "</textarea>"),
                ("<listing>", "</listing>"),
                ("<div v-pre>", "</div>"),
                ("<!--", "-->"),
                ("<div title=\"", "\">x</div>"),
                ("<div>{{ `", "` }}</div>"),
            ] {
                let source = [
                    open.as_bytes(),
                    raw.as_bytes(),
                    close.as_bytes(),
                    newline,
                    b"<p>x</p>",
                ]
                .concat();
                let expected = [
                    b"  ".as_slice(),
                    open.as_bytes(),
                    raw.as_bytes(),
                    close.as_bytes(),
                    newline,
                    b"  <p>x</p>",
                    newline,
                ]
                .concat();
                let mut output = Vec::new();
                write(
                    &mut output,
                    std::str::from_utf8(&source).unwrap(),
                    b"  ",
                    newline,
                );
                assert_eq!(
                    output, expected,
                    "raw opening/closing separator and following layout boundary"
                );
            }
        }
    }
}
