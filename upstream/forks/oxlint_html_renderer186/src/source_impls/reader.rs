use super::*;
use oxc_span::Span;
#[derive(Default)]
pub(super) struct TrailingContext {
    pub(super) active: bool,
    pub(super) saw_newline: bool,
    pub(super) line_count: usize,
}

impl TrailingContext {
    pub(super) fn activate(&mut self) {
        self.active = true;
    }

    pub(super) fn record_break(&mut self) {
        if self.saw_newline {
            self.line_count += 1;
        } else {
            self.saw_newline = true;
        }
    }

    pub(super) fn is_complete(&self, requested_lines: usize) -> bool {
        self.line_count >= requested_lines
    }
}

/// A single span read. Keeping its counters together makes the phase
/// transitions and the final payload invariants explicit.
pub(super) struct SpanReader<'a> {
    pub(super) input: &'a [u8],
    pub(super) request: SpanRequest,
    pub(super) context: ContextLines,
    pub(super) leading: LeadingContext,
    pub(super) trailing: TrailingContext,
    pub(super) line_count: usize,
    pub(super) current_line_start: usize,
    pub(super) start_column: usize,
    pub(super) offset: usize,
}

impl<'a> SpanReader<'a> {
    #[cfg(test)]
    pub(super) fn from_span(
        input: &'a [u8],
        span: Span,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Self {
        Self::new(
            input,
            SpanRequest::new(span),
            ContextLines::new(context_lines_before, context_lines_after),
        )
    }

    pub(super) fn new(input: &'a [u8], request: SpanRequest, context: ContextLines) -> Self {
        let offset = request.prefix_end(input);
        let PrefixScan {
            line_count,
            leading,
            current_line_start,
        } = PrefixScan::new(input, offset, context.before);
        Self {
            input,
            request,
            context,
            leading,
            trailing: TrailingContext::default(),
            line_count,
            current_line_start,
            start_column: offset - current_line_start,
            offset,
        }
    }

    pub(super) fn read(mut self) -> Option<SpanContents<'a>> {
        while self.offset < self.input.len() {
            let byte = self.input[self.offset];
            if matches!(byte, b'\r' | b'\n') {
                let end = if byte == b'\r' && self.input.get(self.offset + 1) == Some(&b'\n') {
                    self.offset + 1
                } else {
                    self.offset
                };
                if self.consume_line_break(end) {
                    self.offset = end + 1;
                    break;
                }
                self.offset = end;
            } else if self.offset < self.request.offset {
                self.start_column += 1;
            }

            if self.offset >= self.request.end_threshold() {
                self.trailing.activate();
                if self.trailing.is_complete(self.context.after) {
                    self.offset += 1;
                    break;
                }
            }
            self.offset += 1;
        }
        self.finish()
    }

    /// Returns whether the requested trailing context is complete.
    pub(super) fn consume_line_break(&mut self, end: usize) -> bool {
        self.line_count += 1;
        if end < self.request.offset {
            self.start_column = 0;
            self.leading.push(self.current_line_start);
        } else if end >= self.request.trailing_break_threshold() && self.trailing.active {
            self.start_column = 0;
            self.trailing.record_break();
            if self.trailing.is_complete(self.context.after) {
                return true;
            }
        }
        self.current_line_start = end + 1;
        false
    }

    pub(super) fn finish(self) -> Option<SpanContents<'a>> {
        if self.offset < self.request.end_threshold() {
            return None;
        }

        let start = self.leading.starting_offset(self.request.offset);
        // A zero-length span starting just past the end of the input reaches
        // the threshold but has no content to slice.
        let Some(data) = self.input.get(start..self.offset) else {
            return None;
        };
        let span_start = u32::try_from(start).ok()?;
        let span_len = u32::try_from(self.offset - start).ok()?;
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

impl SpanContents<'_> {
    /// The 0-indexed line and column of an absolute source `offset` that lies
    /// within this payload, derived without re-reading the source.
    ///
    /// Equivalent to the `line()`/`column()` a fresh zero-context span read
    /// would report, but obtained by scanning only this payload's prefix up to
    /// `offset`. Returns `None` when `offset` is past this payload. Newline
    /// handling mirrors [`SpanReader`] (a `\r\n` pair
    /// and a lone `\r`/`\n` each count once), scanned with the same `memchr2`
    /// primitive so a long (e.g. minified) line stays cheap.
    // Only the `fancy` graphical renderer consumes this today; without it the
    // method is dead in a lib-only build (CI lints `-D warnings`).
    pub(crate) fn line_column_at(&self, offset: usize) -> Option<(usize, usize)> {
        let data = self.data();
        let base = self.span().start as usize;
        let mut rel = offset.saturating_sub(base);
        // A label past the end of the payload is out of bounds; reject it rather
        // than clamping to a misleading end-of-snippet position.
        if rel > data.len() {
            return None;
        }
        // An offset landing on the `\n` of a `\r\n` pair sits inside an
        // unfinished break: `SpanReader` has not advanced the line at that
        // byte and reports the preceding `\r`. Normalize to that byte.
        if rel > 0 && rel < data.len() && data[rel - 1] == b'\r' && data[rel] == b'\n' {
            rel -= 1;
        }
        let mut line = self.line();
        // Byte index just past the most recent line break — the start of the
        // line `offset` falls on. `None` until the first break is seen, meaning
        // `offset` is still on this payload's first line.
        let mut line_start: Option<usize> = None;
        for line_break in LineBreaks::new(&data[..rel]) {
            line += 1;
            line_start = Some(line_break.next_line_start());
        }
        Some(match line_start {
            // A later line, which by definition starts at column 0.
            Some(start) => (line, rel - start),
            // Still on the first line: offset from this payload's start column.
            None => (line, self.column() + rel),
        })
    }
}
