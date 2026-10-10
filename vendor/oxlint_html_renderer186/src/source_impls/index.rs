use super::*;
use oxc_span::Span;
pub(super) struct LineIndex<'a> {
    pub(super) input: &'a [u8],
    /// Starts (byte offsets) of consecutive lines, the first of which is line
    /// number `base_line`; covers every line whose start lies in
    /// `[line_starts[0], frontier]`. Empty until the first query seeds it.
    pub(super) line_starts: Vec<usize>,
    /// 0-indexed line number of `line_starts[0]`.
    pub(super) base_line: usize,
    /// Bytes in `[0, frontier)` have been scanned: every line break there is
    /// either recorded in `line_starts` or (before `line_starts[0]`) summed
    /// into `base_line`. Never splits a `\r\n` pair.
    pub(super) frontier: usize,
}

impl<'a> LineIndex<'a> {
    pub(super) fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            line_starts: Vec::new(),
            base_line: 0,
            frontier: 0,
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.line_starts.is_empty()
    }

    pub(super) fn origin(&self) -> Option<usize> {
        self.line_starts.first().copied()
    }

    /// First query: retain the line starts in its leading context window and
    /// use them as the origin of the reusable index.
    pub(super) fn init(&mut self, cut: usize, context_lines_before: usize) {
        let PrefixScan {
            line_count,
            leading,
            current_line_start,
        } = PrefixScan::new(self.input, cut, context_lines_before);
        debug_assert_eq!(leading.start_line + leading.len(), line_count);
        self.base_line = leading.start_line;
        self.line_starts.reserve(leading.len() + 8);
        leading.append_to(&mut self.line_starts);
        self.line_starts.push(current_line_start);
        self.frontier = cut;
    }

    /// Extend the index so every break in `[0, target)` is recorded, with one
    /// bulk `memchr` pass. (A `read_span`-shaped `target` never splits a
    /// `\r\n` pair, but hold the pair back a byte if one would be.)
    pub(super) fn cover(&mut self, mut target: usize) {
        if target > self.frontier
            && self.input[target - 1] == b'\r'
            && self.input.get(target) == Some(&b'\n')
        {
            target -= 1;
        }
        if target <= self.frontier {
            return;
        }
        for line_break in LineBreaks::new(&self.input[self.frontier..target]) {
            self.line_starts
                .push(line_break.shifted(self.frontier).next_line_start());
        }
        self.frontier = target;
    }

    /// Scan forward from `frontier` to the next line break, recording the
    /// line start after it. `None` at end of input (with `frontier` advanced
    /// there so the probe isn't repeated).
    pub(super) fn extend(&mut self) -> Option<LineBreak> {
        if let Some(line_break) = LineBreaks::new(&self.input[self.frontier..]).next() {
            let line_break = line_break.shifted(self.frontier);
            self.line_starts.push(line_break.next_line_start());
            self.frontier = line_break.next_line_start();
            Some(line_break)
        } else {
            self.frontier = self.input.len();
            None
        }
    }

    /// 0-indexed line number of the line containing `offset`. Requires
    /// `line_starts[0] <= offset` and `offset <= frontier`.
    pub(super) fn line_index_of(&self, offset: usize) -> usize {
        self.base_line + self.line_starts.partition_point(|&start| start <= offset) - 1
    }

    /// Byte offset where line number `line` starts. Requires the line to be
    /// indexed.
    pub(super) fn line_start_of(&self, line: usize) -> usize {
        self.line_starts[line - self.base_line]
    }

    /// The break terminating line number `line` (which contains `pos`),
    /// extending the scan on demand.
    /// `None` when the line runs to end of input.
    pub(super) fn break_ending_line(&mut self, line: usize, pos: usize) -> Option<LineBreak> {
        if let Some(&next_start) = self.line_starts.get(line + 1 - self.base_line) {
            let line_break = LineBreak::ending_at(self.input, next_start - 1);
            debug_assert!(line_break.start >= pos);
            return Some(line_break);
        }
        // `pos` is on the last indexed line; its terminator (if any) is at or
        // past the frontier.
        debug_assert!(pos <= self.frontier);
        self.extend()
    }
}

/// Leading-context state reconstructed from a [`LineIndex`].
pub(super) struct IndexedLeadingContext {
    pub(super) limit: usize,
    pub(super) start_line: usize,
    pub(super) len: usize,
}

impl IndexedLeadingContext {
    pub(super) fn new(current_line: usize, limit: usize) -> Self {
        let start_line = current_line - limit.min(current_line);
        Self {
            limit,
            start_line,
            len: current_line - start_line,
        }
    }

    pub(super) fn push(&mut self) {
        self.len += 1;
        if self.len > self.limit {
            self.start_line += 1;
            self.len -= 1;
        }
    }

    pub(super) fn starting_offset(&self, index: &LineIndex<'_>, span_offset: usize) -> usize {
        if self.len > 0 {
            index.line_start_of(self.start_line)
        } else if self.limit == 0 {
            span_offset
        } else {
            0
        }
    }
}

/// One span query replayed against a reusable [`LineIndex`].
pub(super) struct IndexedReader<'index, 'source> {
    pub(super) index: &'index mut LineIndex<'source>,
    pub(super) request: SpanRequest,
    pub(super) context: ContextLines,
    pub(super) leading: IndexedLeadingContext,
    pub(super) trailing: TrailingContext,
    pub(super) line_count: usize,
    pub(super) start_column: usize,
    pub(super) position: usize,
}

impl<'index, 'source> IndexedReader<'index, 'source> {
    pub(super) fn new(
        index: &'index mut LineIndex<'source>,
        request: SpanRequest,
        context: ContextLines,
        position: usize,
    ) -> Self {
        let line_count = index.line_index_of(position);
        Self {
            start_column: position - index.line_start_of(line_count),
            index,
            request,
            context,
            leading: IndexedLeadingContext::new(line_count, context.before),
            trailing: TrailingContext::default(),
            line_count,
            position,
        }
    }

    /// Jump line break to line break while preserving [`SpanReader`]'s exact
    /// edge-case behavior.
    pub(super) fn read(mut self) -> Option<SpanContents<'source>> {
        let input = self.index.input;
        let window_end = loop {
            let line_break = self.index.break_ending_line(self.line_count, self.position);
            let run_end = line_break.map_or(input.len(), |line_break| line_break.start);
            if self.position < self.request.offset {
                self.start_column += self.request.offset.min(run_end) - self.position;
            }
            if run_end > self.request.end_threshold() && run_end > self.position {
                self.trailing.activate();
                if self.trailing.is_complete(self.context.after) {
                    break self.request.end_threshold().max(self.position) + 1;
                }
            }
            let Some(line_break) = line_break else {
                // No more breaks: the scan runs off the end of the input.
                break input.len();
            };
            if self.consume_line_break(line_break.end) {
                break line_break.next_line_start();
            }
            self.position = line_break.next_line_start();
        };
        self.finish(window_end)
    }

    pub(super) fn consume_line_break(&mut self, end: usize) -> bool {
        self.line_count += 1;
        if end < self.request.offset {
            self.start_column = 0;
            self.leading.push();
        } else if end >= self.request.trailing_break_threshold() && self.trailing.active {
            self.start_column = 0;
            self.trailing.record_break();
            if self.trailing.is_complete(self.context.after) {
                return true;
            }
        }
        if end >= self.request.end_threshold() {
            self.trailing.activate();
            return self.trailing.is_complete(self.context.after);
        }
        false
    }

    pub(super) fn finish(self, window_end: usize) -> Option<SpanContents<'source>> {
        if window_end < self.request.end_threshold() {
            return None;
        }
        let start = self
            .leading
            .starting_offset(self.index, self.request.offset);
        let Some(data) = self.index.input.get(start..window_end) else {
            return None;
        };
        let span_start = u32::try_from(start).ok()?;
        let span_len = u32::try_from(window_end - start).ok()?;
        Some(SpanContents::new(
            data,
            Span::sized(span_start, span_len),
            self.leading.start_line,
            if self.context.before == 0 {
                self.start_column
            } else {
                0
            },
            self.line_count,
        ))
    }
}
