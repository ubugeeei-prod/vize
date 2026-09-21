//! The annotated source excerpt: the lines a diagnostic's spans touch, each
//! followed by its marks and labels, laid out the way rustc does.
//!
//! - A span on one line is underlined in place: `^^^` for a primary span,
//!   `---` for a secondary one (a primary mark wins where they overlap). The
//!   rightmost label sits on the underline row when nothing extends past it;
//!   every other label hangs below on its own row, connected by a `|` from
//!   its span's first column, rightmost first, so connectors never cross.
//! - A span over several lines gets a margin column: `/` where it opens at
//!   the start of a line's content, or a `_` connector from the margin to its
//!   first column; `|` down the lines it covers; and `|_^ label` under its
//!   last character. Overlapping multi-line spans get one margin column each.
//! - Lines between shown lines are kept when exactly one line would be
//!   skipped and elided as `...` otherwise; a span over many lines shows its
//!   first two and last two.

use alloc::vec::Vec;
use core::cmp::Reverse;

use super::frame::Frame;
use super::paint::Style;
use super::row::Row;
use super::source::SourceFile;
use vize_s0::String;

/// A span to mark in the excerpt, in normalized byte offsets.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Annotation<'d> {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) primary: bool,
    pub(crate) label: &'d str,
    pub(crate) style: Style,
}

impl Annotation<'_> {
    const fn mark(&self) -> u8 {
        if self.primary { b'^' } else { b'-' }
    }
}

/// A span within one line, in display columns (`end > start`).
struct Single<'d> {
    line: usize,
    start: usize,
    end: usize,
    annotation: Annotation<'d>,
}

/// A span across lines.
struct Multi<'d> {
    first: usize,
    last: usize,
    /// Display column of the first character, on `first`.
    start: usize,
    /// Display column of the last character, on `last`.
    end: usize,
    /// Opens at the start of its first line's content: drawn as `/`.
    slash: bool,
    /// Its margin column. Spans whose lines do not overlap share one.
    depth: usize,
    annotation: Annotation<'d>,
}

/// A laid-out excerpt, ready to write once the gutter width is known.
pub(crate) struct Excerpt<'d> {
    singles: Vec<Single<'d>>,
    multis: Vec<Multi<'d>>,
    depths: usize,
    lines: Vec<usize>,
}

impl<'d> Excerpt<'d> {
    pub(crate) fn new(file: &SourceFile<'_>, annotations: &[Annotation<'d>]) -> Self {
        let mut singles = Vec::new();
        let mut multis = Vec::new();
        for &annotation in annotations {
            let first = file.line_of(annotation.start);
            let last = if annotation.end > annotation.start {
                file.line_of(annotation.end - 1)
            } else {
                first
            };
            let start = file.column(first, annotation.start);
            if first == last {
                let end = file.column(first, annotation.end).max(start + 1);
                singles.push(Single {
                    line: first,
                    start,
                    end,
                    annotation,
                });
            } else {
                let text = file.text();
                let last_char = text[..annotation.end]
                    .char_indices()
                    .next_back()
                    .map_or(annotation.start, |(at, _)| at);
                let lead = &text[file.line_start(first)..annotation.start];
                multis.push(Multi {
                    first,
                    last,
                    start,
                    end: file.column(last, last_char),
                    slash: lead.chars().all(|ch| ch == ' ' || ch == '\t'),
                    depth: 0,
                    annotation,
                });
            }
        }
        multis.sort_by_key(|multi| (multi.first, multi.start, Reverse(multi.last)));
        // Greedy interval colouring: a span takes the leftmost margin column
        // whose previous occupant ended on an earlier line.
        let mut occupied_until: Vec<usize> = Vec::new();
        for multi in &mut multis {
            let free = occupied_until.iter().position(|&last| last < multi.first);
            multi.depth = free.unwrap_or(occupied_until.len());
            match free {
                Some(depth) => occupied_until[depth] = multi.last,
                None => occupied_until.push(multi.last),
            }
        }

        let mut lines = Vec::new();
        for single in &singles {
            lines.push(single.line);
        }
        for multi in &multis {
            lines.extend([multi.first, multi.last]);
            if multi.last - multi.first >= 2 {
                lines.extend([multi.first + 1, multi.last - 1]);
            }
        }
        lines.sort_unstable();
        lines.dedup();
        let mut filled = Vec::with_capacity(lines.len());
        for (index, &line) in lines.iter().enumerate() {
            if index > 0 && line == lines[index - 1] + 2 {
                filled.push(line - 1);
            }
            filled.push(line);
        }
        Self {
            singles,
            multis,
            depths: occupied_until.len(),
            lines: filled,
        }
    }

    /// The largest one-based line number the excerpt prints.
    pub(crate) fn max_line_number(&self) -> usize {
        self.lines.last().map_or(0, |line| line + 1)
    }

    /// Columns the margin takes before source text: one per margin column,
    /// plus a separating space.
    const fn margin_width(&self) -> usize {
        if self.depths == 0 { 0 } else { self.depths + 1 }
    }

    /// The margin with a `|` for every open multi-line span.
    fn margin(&self, open: &[bool]) -> Row<'d> {
        let mut row = Row::new();
        for (multi, _) in self.multis.iter().zip(open).filter(|(_, open)| **open) {
            row.put(multi.depth, b'|', multi.annotation.style);
        }
        row
    }

    /// A `_` rule over `[from, to)` that leaves open spans' `|` standing, so
    /// a connector crossing another span's margin column reads as crossing.
    fn rule(row: &mut Row<'d>, from: usize, to: usize, style: Style) {
        for col in from..to {
            if row.mark_at(col) != Some(b'|') {
                row.put(col, b'_', style);
            }
        }
    }

    pub(crate) fn write(&self, out: &mut String, file: &SourceFile<'_>, frame: &Frame) {
        let offset = self.margin_width();
        let mut open = alloc::vec![false; self.multis.len()];
        let mut by_depth: Vec<usize> = (0..self.multis.len()).collect();
        by_depth.sort_by_key(|&index| self.multis[index].depth);
        for (index, &line) in self.lines.iter().enumerate() {
            if index > 0 && line > self.lines[index - 1] + 1 {
                frame.elision(out, &self.margin(&open));
            }

            let mut margin = self.margin(&open);
            for (at, multi) in self.multis.iter().enumerate() {
                if multi.first == line && multi.slash {
                    margin.put(multi.depth, b'/', multi.annotation.style);
                    open[at] = true;
                }
            }
            frame.source(out, line + 1, &margin, offset, file.line_text(line));

            for &at in &by_depth {
                let multi = &self.multis[at];
                if multi.first == line && !multi.slash {
                    let mut row = self.margin(&open);
                    let style = multi.annotation.style;
                    Self::rule(&mut row, multi.depth + 1, offset + multi.start, style);
                    row.put(offset + multi.start, multi.annotation.mark(), style);
                    frame.row(out, &row);
                    open[at] = true;
                }
            }

            let singles: Vec<&Single<'d>> = self
                .singles
                .iter()
                .filter(|single| single.line == line)
                .collect();
            if !singles.is_empty() {
                for row in single_rows(&singles, &self.margin(&open), offset) {
                    frame.row(out, &row);
                }
            }

            for &at in by_depth.iter().rev() {
                let multi = &self.multis[at];
                if multi.last != line {
                    continue;
                }
                let mut row = self.margin(&open);
                let style = multi.annotation.style;
                Self::rule(&mut row, multi.depth + 1, offset + multi.end, style);
                row.put(offset + multi.end, multi.annotation.mark(), style);
                open[at] = false;
                let label_col = offset + multi.end + 2;
                let mut label = multi.annotation.label.split('\n');
                row.label(label_col, label.next().unwrap_or(""), style);
                frame.row(out, &row);
                for rest in label {
                    let mut row = self.margin(&open);
                    row.label(label_col, rest, style);
                    frame.row(out, &row);
                }
            }
        }
    }
}

/// The underline row and label rows for the single-line spans of one line.
fn single_rows<'d>(singles: &[&Single<'d>], margin: &Row<'d>, offset: usize) -> Vec<Row<'d>> {
    let mut underline = margin.clone();
    for pass_primary in [false, true] {
        for single in singles {
            let annotation = &single.annotation;
            if annotation.primary == pass_primary {
                let (from, to) = (offset + single.start, offset + single.end);
                underline.fill(from, to, annotation.mark(), annotation.style);
            }
        }
    }

    let furthest = singles.iter().map(|single| single.end).max().unwrap_or(0);
    let labeled: Vec<&Single<'d>> = singles
        .iter()
        .copied()
        .filter(|single| !single.annotation.label.trim().is_empty())
        .collect();
    let inline = labeled
        .iter()
        .copied()
        .max_by_key(|single| (single.start, single.end))
        .filter(|single| single.end == furthest);
    let mut hanging: Vec<&Single<'d>> = labeled
        .iter()
        .copied()
        .filter(|single| !inline.is_some_and(|inline| core::ptr::eq(*single, inline)))
        .collect();
    hanging.sort_by_key(|single| Reverse((single.start, single.end)));

    let mut rows = Vec::new();
    let mut connected = false;
    let mut inline_rest = None;
    if let Some(single) = inline {
        let col = offset + single.end + 1;
        let mut label = single.annotation.label.split('\n');
        underline.label(col, label.next().unwrap_or(""), single.annotation.style);
        inline_rest = Some((col, label, single.annotation.style));
    }
    rows.push(underline);
    if let Some((col, rest, style)) = inline_rest {
        for text in rest {
            let mut row = margin.clone();
            for pending in &hanging {
                row.put(offset + pending.start, b'|', pending.annotation.style);
            }
            row.label(col, text, style);
            rows.push(row);
            connected = true;
        }
    }
    if !hanging.is_empty() && !connected {
        let mut row = margin.clone();
        for pending in &hanging {
            row.put(offset + pending.start, b'|', pending.annotation.style);
        }
        rows.push(row);
    }
    for (index, single) in hanging.iter().enumerate() {
        for text in single.annotation.label.split('\n') {
            let mut row = margin.clone();
            for pending in &hanging[index + 1..] {
                row.put(offset + pending.start, b'|', pending.annotation.style);
            }
            row.label(offset + single.start, text, single.annotation.style);
            rows.push(row);
        }
    }
    rows
}
