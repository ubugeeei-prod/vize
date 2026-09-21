//! Machine-applicable fixes, shown as the edited source rather than described.
//!
//! A run of consecutive [`PartKind::Suggestion`](crate::diagnostic::PartKind)
//! parts is one fix: every edit applied together to the lines they touch.
//! Two presentations, as rustc chooses them:
//!
//! - **insertions only**, none spanning lines — the edited lines under a
//!   plain rule with `+++` beneath exactly the inserted text;
//! - **anything else** — the touched lines before (`-`, removed text red) and
//!   after (`+`, inserted text green), closed by a bare rule.
//!
//! Edits are applied in span order to one copy of the region, so the result
//! is exactly what applying the fix would write.

use alloc::vec::Vec;

use super::frame::Frame;
use super::paint::Style;
use super::row::Row;
use super::source::SourceFile;
use super::text;
use vize_s0::String;

/// One replacement: `[start, end)` becomes `replacement`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Edit<'d> {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) replacement: &'d str,
}

/// A laid-out fix.
pub(crate) struct Fix<'d> {
    title: &'d str,
    first_line: usize,
    old: String,
    new: String,
    removed: Vec<(usize, usize)>,
    inserted: Vec<(usize, usize)>,
    insertions_only: bool,
}

impl<'d> Fix<'d> {
    /// Lay out `edits` (sorted by start, non-overlapping) under `title`.
    pub(crate) fn new(file: &SourceFile<'_>, title: &'d str, edits: &[Edit<'d>]) -> Self {
        let text = file.text();
        let low = edits.iter().map(|edit| edit.start).min().unwrap_or(0);
        let high = edits.iter().map(|edit| edit.end).max().unwrap_or(0);
        let first_line = file.line_of(low);
        let region_start = file.line_start(first_line);
        let region_end = file.line_end(file.line_of(high)).max(high);

        let mut new = String::new("");
        let mut removed = Vec::new();
        let mut inserted = Vec::new();
        let mut cursor = region_start;
        for edit in edits {
            new.push_str(&text[cursor..edit.start]);
            if edit.end > edit.start {
                removed.push((edit.start - region_start, edit.end - region_start));
            }
            let at = new.len();
            new.push_str(edit.replacement);
            if !edit.replacement.is_empty() {
                inserted.push((at, new.len()));
            }
            cursor = edit.end;
        }
        new.push_str(&text[cursor..region_end]);

        let insertions_only = edits
            .iter()
            .all(|edit| edit.start == edit.end && !edit.replacement.contains('\n'));
        Self {
            title,
            first_line,
            old: String::from(&text[region_start..region_end]),
            new,
            removed,
            inserted,
            insertions_only,
        }
    }

    /// The largest one-based line number the fix prints.
    pub(crate) fn max_line_number(&self) -> usize {
        let lines = self
            .old
            .split('\n')
            .count()
            .max(self.new.split('\n').count());
        self.first_line + lines
    }

    pub(crate) fn write(&self, out: &mut String, frame: &Frame, help: &str) {
        let painter = frame.painter();
        painter.paint(out, Style::Help, help);
        out.push_str(": ");
        let indent = text::width(help) + 2;
        super::frame::write_lines(out, painter, Style::Plain, self.title, indent);
        frame.blank(out);
        if self.insertions_only {
            self.write_insertions(out, frame);
        } else {
            self.write_diff(out, frame);
        }
    }

    fn write_insertions(&self, out: &mut String, frame: &Frame) {
        let blank = Row::new();
        for (index, (line_start, line)) in lines_of(&self.new).enumerate() {
            let line_end = line_start + line.len();
            let spans: Vec<(usize, usize)> = self
                .inserted
                .iter()
                .filter(|&&(start, end)| start >= line_start && end <= line_end)
                .map(|&(start, end)| (start - line_start, end - line_start))
                .collect();
            if spans.is_empty() {
                continue;
            }
            let number = self.first_line + index + 1;
            frame.source(out, number, &blank, 0, line);
            let mut marks = Row::new();
            for (start, end) in spans {
                let from = text::width(&line[..start]);
                let to = from + text::width(&line[start..end]).max(1);
                marks.fill(from, to, b'+', Style::Added);
            }
            frame.row(out, &marks);
        }
    }

    fn write_diff(&self, out: &mut String, frame: &Frame) {
        let blank = Row::new();
        for (index, (line_start, line)) in lines_of(&self.old).enumerate() {
            let highlights = clip(&self.removed, line_start, line.len(), Style::Removed);
            let number = self.first_line + index + 1;
            frame.numbered(
                out,
                number,
                "-",
                Style::Removed,
                &blank,
                0,
                line,
                &highlights,
            );
        }
        if !self.new.is_empty() {
            for (index, (line_start, line)) in lines_of(&self.new).enumerate() {
                let highlights = clip(&self.inserted, line_start, line.len(), Style::Added);
                let number = self.first_line + index + 1;
                frame.numbered(out, number, "+", Style::Added, &blank, 0, line, &highlights);
            }
        }
        frame.blank(out);
    }
}

/// `text`'s lines with their byte starts, a `\r` before each `\n` dropped.
fn lines_of(text: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut start = 0;
    text.split('\n').map(move |line| {
        let at = start;
        start += line.len() + 1;
        (at, line.strip_suffix('\r').unwrap_or(line))
    })
}

/// The parts of `ranges` inside the line `[line_start, line_start + len)`,
/// relative to the line.
fn clip(
    ranges: &[(usize, usize)],
    line_start: usize,
    len: usize,
    style: Style,
) -> Vec<(usize, usize, Style)> {
    let line_end = line_start + len;
    ranges
        .iter()
        .filter(|&&(start, end)| start < line_end && end > line_start)
        .map(|&(start, end)| {
            (
                start.max(line_start) - line_start,
                end.min(line_end) - line_start,
                style,
            )
        })
        .collect()
}
