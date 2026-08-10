//! Line-scoped lint suppressions and the source ranges they pin to one line.
//!
//! `eslint-disable-next-line` is *physical line* scoped, exactly as in ESLint:
//! the suppression covers the line that follows the comment. When the formatter
//! breaks that line, the suppressed code moves out from under its own pragma and
//! the finding comes back — with nothing in the formatting diff that looks like
//! a cause. So a line covered by a line-scoped suppression is unsplittable: the
//! template formatter re-joins the chunks it would otherwise have emitted as
//! separate lines, keeping the suppression line-based and identical to ESLint's.
//! (#3343)
//!
//! Only the line breaks this layer *chooses* are suppressed. An attribute value
//! that is already multiline, or an interpolation the JS printer wraps, still
//! spans lines; both keep the element's opening tag — where template diagnostics
//! anchor (#3252, #3270) — on the suppressed line.

use std::cmp::Ordering;

use super::TemplateFormatter;
use crate::template::helpers::is_whitespace;

/// Pragmas whose suppression covers the line *after* the comment.
const NEXT_LINE_PRAGMAS: [&[u8]; 4] = [
    b"eslint-disable-next-line",
    b"vize-disable-next-line",
    b"@vize:expected",
    b"@vize:level(",
];

/// Pragmas whose suppression covers the line the comment itself sits on.
const SAME_LINE_PRAGMAS: [&[u8]; 2] = [b"eslint-disable-line", b"vize-disable-line"];

/// Substrings shared by every pragma above, used to skip the line scan for the
/// overwhelming majority of templates that carry no suppression at all.
const PRAGMA_MARKERS: [&[u8]; 2] = [b"-disable-", b"@vize:"];

/// Tracks which output line the formatter is currently filling, so chunks that
/// share an unsplittable source line share an output line too.
pub(super) struct LineJoiner<'s> {
    source: &'s [u8],
    /// Sorted, disjoint source byte ranges that must stay on one output line.
    locked: Vec<(usize, usize)>,
    /// Index into `locked` of the range the previously emitted chunk belonged
    /// to, or `None` when that chunk was free to end its line.
    current: Option<usize>,
    /// End of the last emitted source chunk. Adjacent authored chunks must
    /// remain adjacent; inserting formatter layout whitespace would create a
    /// runtime Vue text node.
    previous_end: Option<usize>,
}

impl<'s> LineJoiner<'s> {
    pub(super) fn new(source: &'s [u8]) -> Self {
        Self {
            source,
            locked: locked_line_ranges(source),
            current: None,
            previous_end: None,
        }
    }

    /// Decide how to start the chunk that begins at `start` in the source.
    ///
    /// `None` starts a fresh line at the caller's indent — the ordinary case.
    /// `Some(spaced)` continues the line the previous chunk opened, inserting a
    /// single separating space when the source separated the two chunks with
    /// whitespace. The gap between two chunks is whitespace by construction:
    /// an empty gap stays adjacent, horizontal whitespace becomes one space,
    /// and any authored line break keeps a fresh output line.
    pub(super) fn open(&mut self, start: usize) -> Option<bool> {
        let previous = self.current;
        self.current = self.locked_index(start);
        if let Some(end) = self.previous_end.filter(|end| *end <= start) {
            let gap = &self.source[end..start];
            if gap.is_empty() {
                return Some(false);
            }
            if !gap.iter().any(|byte| matches!(byte, b'\n' | b'\r'))
                && gap.iter().copied().all(is_whitespace)
            {
                return Some(true);
            }
        }
        if self.current.is_none() || self.current != previous {
            return None;
        }
        Some(start > 0 && is_whitespace(self.source[start - 1]))
    }

    /// Record the source end of the chunk just emitted.
    pub(super) fn finish(&mut self, end: usize) {
        self.previous_end = Some(end);
    }

    fn locked_index(&self, pos: usize) -> Option<usize> {
        self.locked
            .binary_search_by(|&(start, end)| {
                if pos < start {
                    Ordering::Greater
                } else if pos >= end {
                    Ordering::Less
                } else {
                    Ordering::Equal
                }
            })
            .ok()
    }
}

impl TemplateFormatter<'_> {
    /// Start a chunk of output, either on a fresh indented line or continuing
    /// the current one. See [`LineJoiner::open`] for how `join` is decided.
    pub(super) fn open_chunk(&self, output: &mut Vec<u8>, depth: usize, join: Option<bool>) {
        let Some(spaced) = join else {
            self.write_indent(output, depth);
            return;
        };
        // The previous chunk already closed its line; take that newline back so
        // both chunks stay on the line their suppression comment covers.
        while output
            .last()
            .is_some_and(|&byte| byte == b'\n' || byte == b'\r')
        {
            output.pop();
        }
        if spaced {
            output.push(b' ');
        }
    }
}

/// Text accumulated for the current output line, together with where it started
/// in the source. The offset lets the joiner tell whether the run shares an
/// unsplittable line with the chunk that follows it.
pub(super) struct TextRun {
    bytes: Vec<u8>,
    start: usize,
    end: usize,
}

impl TextRun {
    pub(super) fn new() -> Self {
        Self {
            bytes: Vec::with_capacity(256),
            start: 0,
            end: 0,
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Source offset the buffered text begins at.
    pub(super) fn start(&self) -> usize {
        self.start
    }

    /// One-past-the-last source byte retained by the buffered run.
    pub(super) fn end(&self) -> usize {
        self.end
    }

    pub(super) fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes).unwrap_or("")
    }

    pub(super) fn clear(&mut self) {
        self.bytes.clear();
    }

    /// Append `source[start..end]`, separating it from already buffered text
    /// with a single space (an interpolation or a stray `<` split the runs).
    pub(super) fn push_source(&mut self, source: &[u8], start: usize, end: usize) {
        if self.bytes.is_empty() {
            self.start = start;
        } else {
            self.bytes.push(b' ');
        }
        self.bytes.extend_from_slice(&source[start..end]);
        self.end = end;
    }

    /// Append a single byte the tag scanner rejected as markup.
    pub(super) fn push_byte(&mut self, at: usize, byte: u8) {
        if self.bytes.is_empty() {
            self.start = at;
        }
        self.bytes.push(byte);
        self.end = at + 1;
    }
}

/// Source byte ranges (trimmed to their content) that a line-scoped suppression
/// covers, in ascending order and at most one per line.
fn locked_line_ranges(source: &[u8]) -> Vec<(usize, usize)> {
    let mut locked = Vec::new();
    if !contains_any(source, &PRAGMA_MARKERS) {
        return locked;
    }

    let mut line_start = 0;
    let mut pragma_above = false;
    loop {
        let line_end = memchr::memchr(b'\n', &source[line_start..])
            .map_or(source.len(), |offset| line_start + offset);
        let line = &source[line_start..line_end];
        // A blank line ends nothing and covers nothing: ESLint would apply the
        // suppression to it and suppress no code at all.
        if let Some((start, end)) = content_span(line)
            && (pragma_above || contains_any(line, &SAME_LINE_PRAGMAS))
        {
            locked.push((line_start + start, line_start + end));
        }
        pragma_above = contains_any(line, &NEXT_LINE_PRAGMAS);
        if line_end >= source.len() {
            break;
        }
        line_start = line_end + 1;
    }
    locked
}

/// Offsets of the first and one-past-the-last non-whitespace byte of `line`.
fn content_span(line: &[u8]) -> Option<(usize, usize)> {
    let start = line.iter().position(|&byte| !is_whitespace(byte))?;
    let end = line.iter().rposition(|&byte| !is_whitespace(byte))? + 1;
    Some((start, end))
}

/// `memmem` rather than a naive window scan: the marker check in
/// [`locked_line_ranges`] runs over every template the formatter sees, including
/// the overwhelming majority that carry no suppression at all.
fn contains_any(haystack: &[u8], needles: &[&[u8]]) -> bool {
    needles
        .iter()
        .any(|needle| memchr::memmem::find(haystack, needle).is_some())
}

#[cfg(test)]
mod tests {
    use super::locked_line_ranges;

    #[test]
    fn ranges_track_pragma_placement() {
        assert!(locked_line_ranges(b"<div>\n  <p>x</p>\n</div>").is_empty());
        // The range covers the trimmed content of the following line only, so
        // indentation and the line break stay outside it.
        let next_line = b"<!-- eslint-disable-next-line -->\n  a <b/>\n<p/>";
        assert_eq!(locked_line_ranges(next_line), [(36, 42)]);
        assert_eq!(&next_line[36..42], b"a <b/>");
        // A blank line after the pragma pins nothing.
        assert!(locked_line_ranges(b"<!-- eslint-disable-next-line -->\n\n<p/>").is_empty());
        // A same-line pragma pins the line it sits on, and stacking both forms
        // records one range per line rather than a duplicate.
        let both = b"<!-- eslint-disable-next-line -->\n<p/> <!-- eslint-disable-line -->";
        assert_eq!(locked_line_ranges(both), [(34, 67)]);
        assert_eq!(&both[34..67], b"<p/> <!-- eslint-disable-line -->");
    }
}
