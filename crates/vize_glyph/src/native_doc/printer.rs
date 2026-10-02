//! Iterative document layout. This module never inspects parser nodes.

use vize_l0::String;

use super::document::{Doc, Kind, Line};

/// Newline bytes for generated layout. Source text keeps its original bytes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LineEnding {
    #[default]
    Lf,
    CrLf,
}

/// Layout policy. Width measures Unicode scalars, not UTF-8 byte lengths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrintOptions {
    pub width: usize,
    pub indent_width: usize,
    pub line_ending: LineEnding,
}

impl Default for PrintOptions {
    fn default() -> Self {
        Self {
            width: 80,
            indent_width: 2,
            line_ending: LineEnding::Lf,
        }
    }
}

#[derive(Clone, Copy)]
enum Mode {
    Flat,
    Broken,
}

#[derive(Clone, Copy)]
struct Frame<'d, 'a> {
    doc: &'d Doc<'a>,
    indent: usize,
    mode: Mode,
}

/// Print a document without reparsing source, consulting semantic levels or
/// recursively consuming the native call stack. Unbreakable text may exceed
/// the target width; its bytes are never truncated or rewritten.
pub fn print(doc: &Doc<'_>, options: &PrintOptions) -> String {
    let mut output = String::default();
    let mut column = 0usize;
    let mut pending = std::vec::Vec::new();
    let mut lookahead = std::vec::Vec::new();
    pending.push(Frame {
        doc,
        indent: 0,
        mode: Mode::Broken,
    });
    while let Some(Frame { doc, indent, mode }) = pending.pop() {
        match &doc.kind {
            Kind::Text(text) => {
                output.push_str(text);
                for ch in text.chars() {
                    column = match ch {
                        '\r' | '\n' => 0,
                        '\t' => column.saturating_add(8 - column % 8),
                        _ => column.saturating_add(1),
                    };
                }
            }
            Kind::Line(line) => match mode {
                Mode::Flat => {
                    if *line == Line::Space {
                        output.push(' ');
                        column = column.saturating_add(1);
                    }
                }
                Mode::Broken => newline(&mut output, &mut column, indent, options),
            },
            Kind::HardLine => newline(&mut output, &mut column, indent, options),
            Kind::Concat(parts) => {
                for part in parts.iter().rev() {
                    pending.push(Frame {
                        doc: part,
                        indent,
                        mode,
                    });
                }
            }
            Kind::Group(inner) => {
                let mode = if matches!(mode, Mode::Flat)
                    || fits(doc, &pending, &mut lookahead, column, options.width)
                {
                    Mode::Flat
                } else {
                    Mode::Broken
                };
                pending.push(Frame {
                    doc: inner,
                    indent,
                    mode,
                });
            }
            Kind::Indent(levels, inner) => pending.push(Frame {
                doc: inner,
                indent: indent.saturating_add(levels.saturating_mul(options.indent_width)),
                mode,
            }),
        }
    }
    output
}

/// Include the pending continuation through its first line break. A group
/// cannot steal the width needed by a closing token outside its own boundary.
/// One reusable stack handles lookahead; complete flat subtrees use the cache.
fn fits<'d, 'a>(
    doc: &'d Doc<'a>,
    pending: &[Frame<'d, 'a>],
    lookahead: &mut std::vec::Vec<Frame<'d, 'a>>,
    mut column: usize,
    width: usize,
) -> bool {
    let Some(own_width) = doc.flat_width else {
        return false;
    };
    if own_width > width.saturating_sub(column) {
        return false;
    }
    column = column.saturating_add(own_width);
    lookahead.clear();
    lookahead.extend_from_slice(pending);
    while let Some(Frame { doc, indent, mode }) = lookahead.pop() {
        if matches!(mode, Mode::Flat)
            && let Some(flat_width) = doc.flat_width
        {
            if flat_width > width.saturating_sub(column) {
                return false;
            }
            column = column.saturating_add(flat_width);
            continue;
        }
        match &doc.kind {
            Kind::Text(text) => {
                for ch in text.chars() {
                    if matches!(ch, '\r' | '\n') {
                        return true;
                    }
                    let amount = if ch == '\t' { 8 - column % 8 } else { 1 };
                    if amount > width.saturating_sub(column) {
                        return false;
                    }
                    column = column.saturating_add(amount);
                }
            }
            Kind::Line(line) => {
                if matches!(mode, Mode::Broken) {
                    return true;
                }
                if *line == Line::Space {
                    if column >= width {
                        return false;
                    }
                    column = column.saturating_add(1);
                }
            }
            Kind::HardLine => return true,
            Kind::Concat(parts) => {
                for part in parts.iter().rev() {
                    lookahead.push(Frame {
                        doc: part,
                        indent,
                        mode,
                    });
                }
            }
            Kind::Group(inner) | Kind::Indent(_, inner) => {
                lookahead.push(Frame {
                    doc: inner,
                    indent,
                    mode,
                });
            }
        }
    }
    true
}

fn newline(output: &mut String, column: &mut usize, indent: usize, options: &PrintOptions) {
    output.push_str(match options.line_ending {
        LineEnding::Lf => "\n",
        LineEnding::CrLf => "\r\n",
    });
    for _ in 0..indent {
        output.push(' ');
    }
    *column = indent;
}
