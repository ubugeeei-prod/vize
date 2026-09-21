//! The gutter every row of a diagnostic hangs from.
//!
//! One width serves the whole diagnostic — excerpt, footers and fixes — so
//! every `|` of the rule lines up: it is the digit count of the largest line
//! number any part prints.

use super::paint::{Painter, Style};
use super::row::Row;
use super::text;
use vize_s0::String;

pub(crate) struct Frame {
    width: usize,
    painter: Painter,
}

impl Frame {
    pub(crate) fn new(max_line_number: usize, painter: Painter) -> Self {
        let mut width = 1;
        let mut rest = max_line_number / 10;
        while rest > 0 {
            width += 1;
            rest /= 10;
        }
        Self { width, painter }
    }

    pub(crate) const fn painter(&self) -> Painter {
        self.painter
    }

    fn pad(out: &mut String, columns: usize) {
        for _ in 0..columns {
            out.push(' ');
        }
    }

    fn number(&self, out: &mut String, number: usize) {
        let mut digits = String::new("");
        let _ = core::fmt::Write::write_fmt(&mut digits, format_args!("{number}"));
        Self::pad(out, self.width.saturating_sub(digits.len()));
        self.painter.paint(out, Style::Gutter, digits.as_str());
    }

    /// `  --> path:line:column`
    pub(crate) fn location(&self, out: &mut String, path: &str, line: u32, column: u32) {
        Self::pad(out, self.width);
        self.painter.paint(out, Style::Gutter, "-->");
        out.push(' ');
        text::push_display(out, path);
        let _ = core::fmt::Write::write_fmt(out, format_args!(":{line}:{column}\n"));
    }

    /// The bare rule: `   |`.
    pub(crate) fn blank(&self, out: &mut String) {
        Self::pad(out, self.width + 1);
        self.painter.paint(out, Style::Gutter, "|");
        out.push('\n');
    }

    /// An annotation row under the rule.
    pub(crate) fn row(&self, out: &mut String, row: &Row<'_>) {
        Self::pad(out, self.width + 1);
        self.painter.paint(out, Style::Gutter, "|");
        if !row.is_blank() {
            out.push(' ');
            row.write(out, self.painter);
        }
        out.push('\n');
    }

    /// A numbered source row: margin marks padded to `offset`, then the text.
    pub(crate) fn source(
        &self,
        out: &mut String,
        number: usize,
        margin: &Row<'_>,
        offset: usize,
        line: &str,
    ) {
        self.numbered(out, number, "|", Style::Gutter, margin, offset, line, &[]);
    }

    /// A numbered row with a custom rule mark (`+`/`-` in a fix), the text
    /// styled in `highlights` byte ranges.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn numbered(
        &self,
        out: &mut String,
        number: usize,
        rule: &str,
        rule_style: Style,
        margin: &Row<'_>,
        offset: usize,
        line: &str,
        highlights: &[(usize, usize, Style)],
    ) {
        self.number(out, number);
        out.push(' ');
        self.painter.paint(out, rule_style, rule);
        let line = line.trim_end();
        if margin.is_blank() && line.is_empty() {
            out.push('\n');
            return;
        }
        out.push(' ');
        let used = margin.write(out, self.painter);
        if !line.is_empty() {
            Self::pad(out, offset.saturating_sub(used));
            write_highlighted(out, self.painter, line, highlights);
        }
        out.push('\n');
    }

    /// The `...` of skipped lines, continuing any open margin.
    pub(crate) fn elision(&self, out: &mut String, margin: &Row<'_>) {
        out.push_str("...");
        if !margin.is_blank() {
            Self::pad(out, self.width);
            margin.write(out, self.painter);
        }
        out.push('\n');
    }

    /// `   = word: message`, continuation lines aligned under the message.
    pub(crate) fn footer(&self, out: &mut String, word: &str, message: &str) {
        Self::pad(out, self.width + 1);
        self.painter.paint(out, Style::Gutter, "=");
        out.push(' ');
        self.painter.paint(out, Style::Headline, word);
        out.push_str(": ");
        let indent = self.width + 1 + 2 + text::width(word) + 2;
        write_lines(out, self.painter, Style::Plain, message, indent);
    }
}

/// Append `message` in `style`, each line after the first indented by
/// `indent`, each line's trailing whitespace trimmed, ending in a newline.
pub(crate) fn write_lines(
    out: &mut String,
    painter: Painter,
    style: Style,
    message: &str,
    indent: usize,
) {
    for (index, line) in message.trim_end().split('\n').enumerate() {
        let line = line.trim_end();
        if index > 0 {
            out.push('\n');
            if !line.is_empty() {
                Frame::pad(out, indent);
            }
        }
        if !line.is_empty() {
            let opened = painter.open(out, style);
            text::push_display(out, line);
            painter.close(out, opened);
        }
    }
    out.push('\n');
}

/// Append `line`'s display with the byte ranges in `highlights` styled.
fn write_highlighted(
    out: &mut String,
    painter: Painter,
    line: &str,
    highlights: &[(usize, usize, Style)],
) {
    let mut at = 0;
    for &(start, end, style) in highlights {
        let (start, end) = (start.min(line.len()), end.min(line.len()));
        if start < at || start >= end {
            continue;
        }
        text::push_display(out, &line[at..start]);
        let opened = painter.open(out, style);
        text::push_display(out, &line[start..end]);
        painter.close(out, opened);
        at = end;
    }
    text::push_display(out, &line[at..]);
}
