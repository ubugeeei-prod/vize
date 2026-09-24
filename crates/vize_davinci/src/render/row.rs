//! One annotation row under a source line: single-column ASCII marks at
//! display columns, optionally ended by a label.
//!
//! Marks (`^`, `-`, `|`, `_`, `/`, `+`) are one column wide by construction,
//! so a row is a plain cell vector indexed by display column. Labels are
//! producer text of any width and script; the layout only ever places a label
//! as the **last** thing on its row (hanging labels have their pending `|`
//! connectors strictly to their left), so a label is a tail rather than
//! cells, and its width never has to be split into columns.

use alloc::vec::Vec;

use super::paint::{Painter, Style};
use super::text;
use vize_s0::String;

#[derive(Debug, Clone, Default)]
pub(crate) struct Row<'t> {
    cells: Vec<(u8, Style)>,
    tail: Option<(usize, &'t str, Style)>,
}

impl<'t> Row<'t> {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Set the cell at `col` to `mark`, padding with spaces.
    pub(crate) fn put(&mut self, col: usize, mark: u8, style: Style) {
        if self.cells.len() <= col {
            self.cells.resize(col + 1, (b' ', Style::Plain));
        }
        if let Some(cell) = self.cells.get_mut(col) {
            *cell = (mark, style);
        }
    }

    /// Set every cell in `[from, to)` to `mark`.
    pub(crate) fn fill(&mut self, from: usize, to: usize, mark: u8, style: Style) {
        for col in from..to {
            self.put(col, mark, style);
        }
    }

    /// The mark at `col`, if any non-space mark is there.
    pub(crate) fn mark_at(&self, col: usize) -> Option<u8> {
        self.cells
            .get(col)
            .map(|&(mark, _)| mark)
            .filter(|&mark| mark != b' ')
    }

    /// End the row with `label` starting at `col`. Cells at or past `col` are
    /// dropped: nothing may sit under a label.
    pub(crate) fn label(&mut self, col: usize, label: &'t str, style: Style) {
        self.cells.truncate(col);
        self.tail = Some((col, label, style));
    }

    /// Whether the row prints anything.
    pub(crate) fn is_blank(&self) -> bool {
        self.visible_tail().is_none() && self.cells.iter().all(|&(mark, _)| mark == b' ')
    }

    /// The label, if it prints anything once trailing whitespace is trimmed.
    fn visible_tail(&self) -> Option<(usize, &'t str, Style)> {
        self.tail
            .map(|(col, label, style)| (col, label.trim_end(), style))
            .filter(|&(_, label, _)| !label.is_empty())
    }

    /// Append the row's content, trailing spaces trimmed; returns the columns
    /// written.
    pub(crate) fn write(&self, out: &mut String, painter: Painter) -> usize {
        let tail = self.visible_tail();
        let cells = match tail {
            Some(_) => self.cells.as_slice(),
            None => {
                let used = self
                    .cells
                    .iter()
                    .rposition(|&(mark, _)| mark != b' ')
                    .map_or(0, |last| last + 1);
                self.cells.get(..used).unwrap_or_default()
            }
        };
        let mut run = String::new("");
        let mut run_style = Style::Plain;
        for &(mark, style) in cells {
            let style = if mark == b' ' { Style::Plain } else { style };
            if style != run_style && !run.is_empty() {
                painter.paint(out, run_style, run.as_str());
                run.clear();
            }
            run_style = style;
            run.push(char::from(mark));
        }
        painter.paint(out, run_style, run.as_str());
        if let Some((col, label, style)) = tail {
            for _ in cells.len()..col {
                out.push(' ');
            }
            let opened = painter.open(out, style);
            text::push_display(out, label);
            painter.close(out, opened);
            return col.max(cells.len()) + text::width(label);
        }
        cells.len()
    }
}
