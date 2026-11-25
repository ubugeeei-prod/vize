//! LSP line starts following a break in a source segment.
//! CR and LF each end a line. A CRLF pair ends one line after its LF.

/// Byte offset immediately after the first line ending, relative to the segment.
#[inline]
pub fn first_start(bytes: &[u8]) -> Option<usize> {
    let at = memchr::memchr2(b'\r', b'\n', bytes)?;
    Some(at + 1 + usize::from(bytes.get(at..at + 2) == Some(b"\r\n")))
}

/// Byte offset immediately after the final line ending, relative to the segment.
#[inline]
pub fn last_start(bytes: &[u8]) -> Option<usize> {
    memchr::memrchr2(b'\r', b'\n', bytes).map(|at| at + 1)
}

#[cfg(test)]
mod tests {
    use super::{first_start, last_start};
    use crate::line_index::LineBreaks;

    #[test]
    fn both_ends_match_the_complete_lsp_line_iterator() {
        let parts = ["", "x", "\r", "\n", "\r\n", "🌸", "\u{2028}", "\u{2029}"];
        for a in parts {
            for b in parts {
                for c in parts {
                    let text = [a, b, c].concat();
                    let mut starts = LineBreaks::Lsp.line_starts(&text).skip(1);
                    let first = starts.next();
                    assert_eq!(first_start(text.as_bytes()), first);
                    assert_eq!(last_start(text.as_bytes()), starts.last().or(first));
                }
            }
        }
    }
}

#[cfg(test)]
mod kani;
