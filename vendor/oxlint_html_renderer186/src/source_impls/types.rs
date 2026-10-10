use oxc_span::Span;
use std::collections::VecDeque;
#[derive(Debug)]
pub struct SpanContents<'a> {
    pub(super) data: &'a [u8],
    pub(super) span: Span,
    pub(super) line: usize,
    pub(super) column: usize,
    pub(super) line_count: usize,
}

impl<'a> SpanContents<'a> {
    pub const fn new(
        data: &'a [u8],
        span: Span,
        line: usize,
        column: usize,
        line_count: usize,
    ) -> Self {
        Self {
            data,
            span,
            line,
            column,
            line_count,
        }
    }

    pub const fn data(&self) -> &'a [u8] {
        self.data
    }

    pub const fn span(&self) -> &Span {
        &self.span
    }

    pub const fn line(&self) -> usize {
        self.line
    }

    pub const fn column(&self) -> usize {
        self.column
    }

    pub const fn line_count(&self) -> usize {
        self.line_count
    }
}

#[derive(Clone, Copy)]
pub(super) struct ContextLines {
    pub(super) before: usize,
    pub(super) after: usize,
}

impl ContextLines {
    pub(super) const fn new(before: usize, after: usize) -> Self {
        Self { before, after }
    }
}

/// The normalized, integer-only form of a [`Span`] query.
#[derive(Clone, Copy)]
pub(super) struct SpanRequest {
    pub(super) offset: usize,
    pub(super) len: usize,
}

impl SpanRequest {
    pub(super) fn new(span: Span) -> Self {
        Self {
            offset: span.start as usize,
            len: span.size() as usize,
        }
    }

    /// Boundary between the bulk prefix scan and the detailed span scan.
    /// Never splits a CRLF pair.
    pub(super) fn prefix_end(self, input: &[u8]) -> usize {
        let mut end = self.offset.saturating_sub(1).min(input.len());
        if end > 0 && input[end - 1] == b'\r' {
            end -= 1;
        }
        end
    }

    /// First byte at which the detailed scan has consumed the requested span.
    pub(super) fn end_threshold(self) -> usize {
        self.offset.saturating_add(self.len).saturating_sub(1)
    }

    /// First line break that can belong to the trailing context.
    pub(super) fn trailing_break_threshold(self) -> usize {
        self.offset.saturating_add(self.len.saturating_sub(1))
    }
}

/// A logical line break. `start == end` for LF or CR and differs by one for
/// CRLF, allowing all scanners to share the same newline handling.
#[derive(Clone, Copy)]
pub(super) struct LineBreak {
    pub(super) start: usize,
    pub(super) end: usize,
}

impl LineBreak {
    pub(super) fn ending_at(input: &[u8], end: usize) -> Self {
        let start = if end > 0 && input[end] == b'\n' && input[end - 1] == b'\r' {
            end - 1
        } else {
            end
        };
        Self { start, end }
    }

    pub(super) const fn next_line_start(self) -> usize {
        self.end + 1
    }

    pub(super) const fn shifted(self, offset: usize) -> Self {
        Self {
            start: self.start + offset,
            end: self.end + offset,
        }
    }
}

/// Iterator over logical line breaks in a slice. It consumes CRLF as one item,
/// so callers do not need their own "skip the LF" branches.
pub(super) struct LineBreaks<'a> {
    pub(super) input: &'a [u8],
    pub(super) positions: memchr::Memchr2<'a>,
}

impl<'a> LineBreaks<'a> {
    pub(super) fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            positions: memchr::memchr2_iter(b'\r', b'\n', input),
        }
    }
}

impl Iterator for LineBreaks<'_> {
    type Item = LineBreak;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let start = self.positions.next()?;
            // The CR already represents this CRLF pair.
            if start > 0 && self.input[start] == b'\n' && self.input[start - 1] == b'\r' {
                continue;
            }
            let end = if self.input[start] == b'\r' && self.input.get(start + 1) == Some(&b'\n') {
                start + 1
            } else {
                start
            };
            return Some(LineBreak { start, end });
        }
    }
}

/// The retained leading-context lines and the absolute line number of the
/// first one. The size invariant lives here instead of being repeated by each
/// scanner.
pub(super) struct LeadingContext {
    pub(super) limit: usize,
    pub(super) start_line: usize,
    pub(super) line_starts: RetainedLineStarts,
}

/// Storage for the retained line starts. The renderer's default one-line
/// context stays inline instead of allocating a `VecDeque` for every span.
pub(super) enum RetainedLineStarts {
    One(Option<usize>),
    Many(VecDeque<usize>),
}

impl LeadingContext {
    pub(super) fn new(limit: usize) -> Self {
        let line_starts = if limit == 1 {
            RetainedLineStarts::One(None)
        } else {
            RetainedLineStarts::Many(VecDeque::new())
        };
        Self {
            limit,
            start_line: 0,
            line_starts,
        }
    }

    pub(super) fn one(start_line: usize, line_start: Option<usize>) -> Self {
        Self {
            limit: 1,
            start_line,
            line_starts: RetainedLineStarts::One(line_start),
        }
    }

    pub(super) fn len(&self) -> usize {
        match &self.line_starts {
            RetainedLineStarts::One(line_start) => usize::from(line_start.is_some()),
            RetainedLineStarts::Many(line_starts) => line_starts.len(),
        }
    }

    pub(super) fn first(&self) -> Option<usize> {
        match &self.line_starts {
            RetainedLineStarts::One(line_start) => *line_start,
            RetainedLineStarts::Many(line_starts) => line_starts.front().copied(),
        }
    }

    #[cfg(test)]
    pub(super) fn last(&self) -> Option<usize> {
        match &self.line_starts {
            RetainedLineStarts::One(line_start) => *line_start,
            RetainedLineStarts::Many(line_starts) => line_starts.back().copied(),
        }
    }

    pub(super) fn push(&mut self, line_start: usize) {
        match &mut self.line_starts {
            RetainedLineStarts::One(retained) => {
                if retained.replace(line_start).is_some() {
                    self.start_line += 1;
                }
            }
            RetainedLineStarts::Many(line_starts) => {
                line_starts.push_back(line_start);
                if line_starts.len() > self.limit {
                    self.start_line += 1;
                    line_starts.pop_front();
                }
            }
        }
    }

    pub(super) fn starting_offset(&self, span_offset: usize) -> usize {
        self.first()
            .unwrap_or(if self.limit == 0 { span_offset } else { 0 })
    }

    pub(super) fn append_to(self, target: &mut Vec<usize>) {
        match self.line_starts {
            RetainedLineStarts::One(line_start) => target.extend(line_start),
            RetainedLineStarts::Many(line_starts) => target.extend(line_starts),
        }
    }
}

/// State produced by scanning the source prefix before a span.
pub(super) struct PrefixScan {
    pub(super) line_count: usize,
    pub(super) leading: LeadingContext,
    pub(super) current_line_start: usize,
}

impl PrefixScan {
    /// Scan the prefix before a span, retaining only its leading context lines.
    #[inline]
    pub(super) fn new(input: &[u8], end: usize, context_lines_before: usize) -> Self {
        let prefix = &input[..end];
        if context_lines_before == 1 {
            return Self::one_context_line(prefix);
        }

        let mut scan = Self {
            line_count: 0,
            leading: LeadingContext::new(context_lines_before),
            current_line_start: 0,
        };
        for line_break in LineBreaks::new(prefix) {
            scan.line_count += 1;
            scan.leading.push(scan.current_line_start);
            scan.current_line_start = line_break.next_line_start();
        }
        scan
    }

    /// The graphical handler's default. Keep the one retained line start in a
    /// scalar while scanning instead of updating a `VecDeque` for every line.
    pub(super) fn one_context_line(prefix: &[u8]) -> Self {
        let mut line_count = 0;
        let mut current_line_start = 0;
        let mut previous_line_start = None;

        if memchr::memchr(b'\r', prefix).is_none() {
            // Most source files only use LF. Count all breaks in one SIMD pass,
            // then recover the two line starts the caller needs from the end.
            line_count = bytecount::count(prefix, b'\n');
            if let Some(last_break) = memchr::memrchr(b'\n', prefix) {
                current_line_start = last_break + 1;
                previous_line_start =
                    Some(memchr::memrchr(b'\n', &prefix[..last_break]).map_or(0, |pos| pos + 1));
            }
        } else {
            for line_break in LineBreaks::new(prefix) {
                line_count += 1;
                previous_line_start = Some(current_line_start);
                current_line_start = line_break.next_line_start();
            }
        }

        let leading = LeadingContext::one(line_count.saturating_sub(1), previous_line_start);
        Self {
            line_count,
            leading,
            current_line_start,
        }
    }
}
