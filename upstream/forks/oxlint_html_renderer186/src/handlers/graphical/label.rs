//! The annotations drawn under the source text.
//!
//! For spans that begin and end on one line,
//! [`render_single_line_highlights`](GraphicalReportHandler::render_single_line_highlights)
//! draws the `───┬───` underlines and then, via
//! [`write_label_text`](GraphicalReportHandler::write_label_text), the label
//! text hanging off each one. For multi-line spans the closing label is drawn
//! by [`render_multi_line_end`](GraphicalReportHandler::render_multi_line_end).

use std::{cmp::max, fmt};

use smallvec::SmallVec;

use super::{
    handler::GraphicalReportHandler,
    line::Line,
    span::{FancySpan, LabelRenderMode},
};
use crate::handlers::theme::{DiagnosticColorize, Style, ThemeCharacters};

pub(super) use super::underline::write_repeated_char;
use super::underline::{Underline, write_padding};

struct LabelText<'a> {
    chars: &'a ThemeCharacters,
    label: &'a str,
    style: Style,
    render_mode: LabelRenderMode,
}

struct LabelContext<'a, 'source, 'label> {
    line: &'a Line<'source>,
    line_number_width: usize,
    max_gutter: usize,
    all_highlights: &'a [FancySpan<'label>],
    vertical_bars: &'a [(&'a FancySpan<'label>, usize)],
}

impl LabelText<'_> {
    fn write_unstyled(&self, f: &mut impl fmt::Write) -> fmt::Result {
        let chars = self.chars;
        let label = self.label;
        match self.render_mode {
            LabelRenderMode::SingleLine => {
                write!(f, "{}{}{} {label}", chars.lbot, chars.hbar, chars.hbar)
            }
            LabelRenderMode::BlockFirst => {
                write!(f, "{}{}{} {label}", chars.lbot, chars.hbar, chars.rcross)
            }
            LabelRenderMode::BlockRest => write!(f, "  {} {label}", chars.vbar),
        }
    }
}

impl fmt::Display for LabelText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let chars = self.chars;
        let label = self.label.style(self.style);
        match self.render_mode {
            LabelRenderMode::SingleLine => {
                write!(f, "{}{}{} {label}", chars.lbot, chars.hbar, chars.hbar)
            }
            LabelRenderMode::BlockFirst => {
                write!(f, "{}{}{} {label}", chars.lbot, chars.hbar, chars.rcross)
            }
            LabelRenderMode::BlockRest => write!(f, "  {} {label}", chars.vbar),
        }
    }
}

impl GraphicalReportHandler {
    pub(super) fn render_single_line_highlights(
        &self,
        f: &mut impl fmt::Write,
        line: &Line<'_>,
        linum_width: usize,
        max_gutter: usize,
        single_liners: &[&FancySpan],
        all_highlights: &[FancySpan],
    ) -> fmt::Result {
        let mut highest = 0;

        let chars = &self.theme.characters;
        let mut vbar_offsets = SmallVec::<[_; 2]>::with_capacity(single_liners.len());
        for &hl in single_liners {
            let byte_start = hl.offset();
            let byte_end = hl.offset() + hl.len();
            let start = Self::visual_offset(line, byte_start, true).max(highest);
            let end = if hl.len() == 0 {
                start + 1
            } else {
                Self::visual_offset(line, byte_end, false).max(start + 1)
            };

            let vbar_offset = usize::midpoint(start, end);
            let num_left = vbar_offset - start;
            let num_right = end - vbar_offset - 1;
            // Throws `Formatting argument out of range` when width is above u16::MAX.
            let width = start.saturating_sub(highest).min(u16::MAX as usize);
            let marker = if hl.len() == 0 {
                chars.uarrow
            } else if hl.has_label() {
                chars.underbar
            } else {
                chars.underline
            };
            let underline = Underline {
                padding: width,
                left: num_left,
                marker,
                right: num_right,
                line: chars.underline,
            };
            if hl.style.is_plain() {
                underline.write(f)?;
            } else {
                write!(f, "{}", underline.style(hl.style))?;
            }
            highest = max(highest, end);
            vbar_offsets.push((hl, vbar_offset));
        }
        f.write_char('\n')?;

        let context = LabelContext {
            line,
            line_number_width: linum_width,
            max_gutter,
            all_highlights,
            vertical_bars: &vbar_offsets,
        };
        for &hl in single_liners.iter().rev() {
            if let Some(label) = hl.label() {
                let mut lines = label.split('\n').peekable();
                let first = lines.next().expect("split always yields at least one item");
                let first_mode = if lines.peek().is_some() {
                    LabelRenderMode::BlockFirst
                } else {
                    LabelRenderMode::SingleLine
                };
                self.write_label_text(f, &context, hl, first, first_mode)?;
                for label_line in lines {
                    self.write_label_text(f, &context, hl, label_line, LabelRenderMode::BlockRest)?;
                }
            }
        }
        Ok(())
    }

    fn write_label_text(
        &self,
        f: &mut impl fmt::Write,
        context: &LabelContext<'_, '_, '_>,
        hl: &FancySpan,
        label: &str,
        render_mode: LabelRenderMode,
    ) -> fmt::Result {
        self.write_no_linum(f, context.line_number_width)?;
        self.render_highlight_gutter(
            f,
            context.max_gutter,
            context.line,
            context.all_highlights,
            LabelRenderMode::SingleLine,
        )?;
        let mut curr_offset = 1usize;
        for (offset_hl, vbar_offset) in context.vertical_bars {
            let padding = (*vbar_offset + 1).saturating_sub(curr_offset);
            write_padding(f, padding)?;
            curr_offset += padding;
            if *offset_hl == hl {
                let line = LabelText {
                    chars: &self.theme.characters,
                    label,
                    style: hl.style,
                    render_mode,
                };
                if hl.style.is_plain() {
                    line.write_unstyled(f)?;
                } else {
                    write!(f, "{}", line.style(hl.style))?;
                }
                f.write_char('\n')?;
                break;
            }
            write!(f, "{}", self.theme.characters.vbar.style(offset_hl.style))?;
            curr_offset += 1;
        }
        Ok(())
    }

    pub(super) fn render_multi_line_end(
        &self,
        f: &mut impl fmt::Write,
        labels: &[FancySpan],
        max_gutter: usize,
        linum_width: usize,
        line: &Line<'_>,
        label: &FancySpan,
    ) -> fmt::Result {
        self.write_no_linum(f, linum_width)?;

        if let Some(label_text) = label.label() {
            let mut lines = label_text.split('\n').peekable();
            let first = lines.next().expect("split always yields at least one item");
            let first_mode = if lines.peek().is_some() {
                LabelRenderMode::BlockFirst
            } else {
                LabelRenderMode::SingleLine
            };
            self.render_highlight_gutter(f, max_gutter, line, labels, first_mode)?;
            self.render_multi_line_end_single(f, first, label.style, first_mode)?;

            for label_line in lines {
                self.write_no_linum(f, linum_width)?;
                self.render_highlight_gutter(
                    f,
                    max_gutter,
                    line,
                    labels,
                    LabelRenderMode::BlockRest,
                )?;
                self.render_multi_line_end_single(
                    f,
                    label_line,
                    label.style,
                    LabelRenderMode::BlockRest,
                )?;
            }
        } else {
            self.render_highlight_gutter(f, max_gutter, line, labels, LabelRenderMode::SingleLine)?;
            writeln!(f, "{}", self.theme.characters.hbar.style(label.style))?;
        }

        Ok(())
    }

    fn render_multi_line_end_single(
        &self,
        f: &mut impl fmt::Write,
        label: &str,
        style: Style,
        render_mode: LabelRenderMode,
    ) -> fmt::Result {
        match render_mode {
            LabelRenderMode::SingleLine => {
                writeln!(
                    f,
                    "{} {}",
                    self.theme.characters.hbar.style(style),
                    label.style(style)
                )?;
            }
            LabelRenderMode::BlockFirst => {
                writeln!(
                    f,
                    "{} {}",
                    self.theme.characters.rcross.style(style),
                    label.style(style)
                )?;
            }
            LabelRenderMode::BlockRest => {
                writeln!(
                    f,
                    "{} {}",
                    self.theme.characters.vbar.style(style),
                    label.style(style)
                )?;
            }
        }

        Ok(())
    }
}
