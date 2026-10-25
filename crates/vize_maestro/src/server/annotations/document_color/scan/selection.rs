//! Allocation-free selection over the shared declaration scanner.

use super::{Collection, ColorLiteral, CssMode};

impl Collection for Vec<ColorLiteral> {
    fn checkpoint(&self) -> usize {
        self.len()
    }

    fn truncate(&mut self, checkpoint: usize) {
        Vec::truncate(self, checkpoint);
    }

    fn push(&mut self, literal: ColorLiteral) {
        Vec::push(self, literal);
    }

    fn finished(&self, _cursor: usize, _committed: bool) -> bool {
        false
    }
}

struct Selected {
    offset: usize,
    count: usize,
    literal: Option<(usize, ColorLiteral)>,
    scanned_end: usize,
}

impl Collection for Selected {
    fn checkpoint(&self) -> usize {
        self.count
    }

    fn truncate(&mut self, checkpoint: usize) {
        if self
            .literal
            .as_ref()
            .is_some_and(|(index, _)| *index >= checkpoint)
        {
            self.literal = None;
        }
        self.count = checkpoint;
    }

    fn push(&mut self, literal: ColorLiteral) {
        if literal.start <= self.offset && self.offset < literal.end {
            self.literal = Some((self.count, literal));
        }
        self.count += 1;
    }

    fn finished(&self, cursor: usize, committed: bool) -> bool {
        // A nested selector can resemble a declaration until its opening
        // brace. Do not publish a candidate until that checkpoint commits.
        committed && cursor > self.offset
    }

    fn finish(&mut self, cursor: usize) {
        self.scanned_end = cursor;
    }
}

pub(in crate::server::annotations::document_color) struct Selection {
    pub(in crate::server::annotations::document_color) literal: Option<ColorLiteral>,
    pub(in crate::server::annotations::document_color) consumed_bytes: usize,
}

pub(in crate::server::annotations::document_color) fn color_at(
    content: &str,
    region: (usize, usize),
    mode: CssMode,
    offset: usize,
) -> Selection {
    if offset < region.0 || offset >= region.1 || !content.is_char_boundary(offset) {
        return Selection {
            literal: None,
            consumed_bytes: 0,
        };
    }
    let selected = super::scan(
        content,
        region,
        mode,
        Selected {
            offset,
            count: 0,
            literal: None,
            scanned_end: region.0,
        },
    );
    Selection {
        literal: selected.literal.map(|(_, literal)| literal),
        consumed_bytes: selected.scanned_end - region.0,
    }
}

#[cfg(test)]
mod tests;
