//! The derived-HTML → authored-pug source map.
//!
//! Every byte of the derived Vue template is emitted from one pug token:
//! verbatim copies (tag names, text, unquoted keys) map byte for byte;
//! synthesized bytes (`<`, quotes, `</div>`, escaped entities, folded
//! literals) map to the construct they came from. S2 spans
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
    /// The derived bytes are a copy of `[pug_start, pug_end)`.
    verbatim: bool,
}

/// Ordered, contiguous segments covering the whole derived template.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PugSourceMap {
    segments: StdVec<Segment>,
    html_len: u32,
}

impl PugSourceMap {
    /// `len` derived bytes copied from the pug bytes at `pug_start`.
    pub(crate) fn push_verbatim(&mut self, len: usize, pug_start: u32) {
        self.push(len, pug_start, pug_start + len as u32, true);
    }

    /// `len` derived bytes synthesized for the construct `[start, end)`.
    pub(crate) fn push_synth(&mut self, len: usize, start: u32, end: u32) {
        self.push(len, start, end, false);
    }

    fn push(&mut self, html_len: usize, pug_start: u32, pug_end: u32, verbatim: bool) {
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
            verbatim,
        });
    }

    /// The segment holding derived byte `at`, and whether it is verbatim.
    fn segment(&self, at: u32) -> Option<(&Segment, bool)> {
        let index = self
            .segments
            .partition_point(|segment| segment.html_end <= at);
        self.segments
            .get(index)
            .map(|segment| (segment, segment.verbatim))
    }

    /// The authored pug offset of derived-template offset `at` (a start
    /// position: synthesized bytes resolve to their construct's start).
    pub fn offset_to_pug(&self, at: u32) -> u32 {
        match self.segment(at) {
            Some((segment, true)) => segment.pug_start + (at - segment.html_start),
            Some((segment, false)) => segment.pug_start,
            None => self.segments.last().map_or(0, |segment| segment.pug_end),
        }
    }

    /// Map a derived-template span onto the authored pug: verbatim bytes
    /// map exactly, and a span that starts or ends in synthesized bytes
    /// widens to the whole construct that produced them.
    pub fn to_pug(&self, span: Span) -> Span {
        let start = self.offset_to_pug(span.start);
        if span.end <= span.start {
            return Span::new(start, start);
        }
        let end = match self.segment(span.end - 1) {
            Some((segment, true)) => segment.pug_start + (span.end - segment.html_start),
            Some((segment, false)) => segment.pug_end,
            None => self.segments.last().map_or(0, |segment| segment.pug_end),
        };
        Span::new(start, end.max(start))
    }

    /// Byte length of the derived template this map covers.
    pub fn html_len(&self) -> u32 {
        self.html_len
    }
}
