//! Arena-resident formatting vocabulary, independent of syntax and layout.

use vize_l0::{Allocator, Box, Vec};

/// What a breakable line becomes when its enclosing group fits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Line {
    /// A single space in flat mode, a newline in broken mode.
    Space,
    /// Nothing in flat mode, a newline in broken mode.
    Empty,
}

/// A document borrows source text and keeps its composition in an arena.
/// Nodes neither own strings nor perform formatting or syntax inspection.
#[derive(Debug)]
pub struct Doc<'a> {
    pub(super) kind: Kind<'a>,
    pub(super) flat_width: Option<usize>,
}

#[derive(Debug)]
pub(super) enum Kind<'a> {
    Text(&'a str),
    Line(Line),
    HardLine,
    Concat(Vec<'a, Doc<'a>>),
    Group(Box<'a, Doc<'a>>),
    Indent(usize, Box<'a, Doc<'a>>),
}

impl<'a> Doc<'a> {
    /// Source text is printed verbatim, including authored line endings.
    /// Width counts Unicode scalar values; a tab/newline prevents flattening.
    pub fn text(text: &'a str) -> Self {
        Self {
            kind: Kind::Text(text),
            flat_width: (!text.contains(['\r', '\n', '\t'])).then(|| text.chars().count()),
        }
    }

    pub fn line(line: Line) -> Self {
        Self {
            kind: Kind::Line(line),
            flat_width: Some(usize::from(line == Line::Space)),
        }
    }

    /// An unconditional newline, even inside a flattened group.
    pub fn hard_line() -> Self {
        Self {
            kind: Kind::HardLine,
            flat_width: None,
        }
    }

    /// Join documents without copying their text. The cached flat width avoids
    /// rescanning child text for every enclosing group's layout decision.
    pub fn concat(parts: Vec<'a, Self>) -> Self {
        let flat_width = parts
            .iter()
            .try_fold(0usize, |width, part| width.checked_add(part.flat_width?));
        Self {
            kind: Kind::Concat(parts),
            flat_width,
        }
    }

    /// Flatten this group when its complete contents fit the remaining width.
    pub fn group(self, allocator: &'a Allocator) -> Self {
        Self {
            flat_width: self.flat_width,
            kind: Kind::Group(Box::new_in(self, &allocator)),
        }
    }

    /// Increase indentation for line breaks inside this document only.
    pub fn indent(self, levels: usize, allocator: &'a Allocator) -> Self {
        Self {
            flat_width: self.flat_width,
            kind: Kind::Indent(levels, Box::new_in(self, &allocator)),
        }
    }
}
