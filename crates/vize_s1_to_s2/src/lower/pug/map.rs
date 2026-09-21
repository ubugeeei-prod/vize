//! The derived-HTML → authored-pug source map.
//!
//! Every byte of the derived Vue template is emitted from one pug token:
//! verbatim copies (tag names, text, unquoted keys) map byte for byte;
//! synthesized bytes (`<`, quotes, `</div>`, escaped entities, folded
//! literals) map to the start of the construct they came from. S2 spans
//! over the derived template reach the authored pug through [`to_pug`].
//!
//! [`to_pug`]: PugSourceMap::to_pug

use alloc::vec::Vec as StdVec;

use vize_s0::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Segment {
    html_start: u32,
    html_end: u32,
    pug_start: u32,
    pug_end: u32,
}

/// Ordered, contiguous segments covering the whole derived template.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PugSourceMap {
    segments: StdVec<Segment>,
    html_len: u32,
}

impl PugSourceMap {
    pub(crate) fn push(&mut self, html_len: usize, pug_start: u32, pug_end: u32) {
        if html_len == 0 {
            return;
        }
        let html_start = self.html_len;
        let html_end = html_start + html_len as u32;
        self.html_len = html_end;
        self.segments.push(Segment {
            html_start,
            html_end,
            pug_start,
            pug_end,
        });
    }

    /// The authored pug offset of derived-template offset `at`.
    pub fn offset_to_pug(&self, at: u32) -> u32 {
        let index = self
            .segments
            .partition_point(|segment| segment.html_end <= at);
        let Some(segment) = self.segments.get(index) else {
            return self.segments.last().map_or(0, |segment| segment.pug_end);
        };
        let verbatim = segment.html_end - segment.html_start == segment.pug_end - segment.pug_start;
        if verbatim {
            segment.pug_start + at.saturating_sub(segment.html_start)
        } else {
            segment.pug_start
        }
    }

    /// Map a derived-template span onto the authored pug.
    pub fn to_pug(&self, span: Span) -> Span {
        let start = self.offset_to_pug(span.start);
        if span.end <= span.start {
            return Span::new(start, start);
        }
        let end = self
            .offset_to_pug(span.end - 1)
            .saturating_add(1)
            .max(start);
        Span::new(start, end)
    }

    /// Byte length of the derived template this map covers.
    pub fn html_len(&self) -> u32 {
        self.html_len
    }
}
