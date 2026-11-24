//! Line positions for one script block without storing the lines before it.
use super::{DocumentSymbol, LineIndex};
use std::borrow::Cow;
use vize_atelier_sfc::BlockLocation;

/// The original reader indexed every line before the block's closing tag,
/// while that block's syntax was alive. Lines before the opening tag only
/// shift line numbers, so they are counted here instead of stored.
pub(super) struct BlockLines<'a> {
    index: LineIndex<'a>,
    /// Document offset of the line that holds the opening tag.
    start: usize,
    /// Lines before `start`.
    line: u32,
}

impl<'a> BlockLines<'a> {
    pub(super) fn new(prefix: &'a str, loc: &BlockLocation) -> Self {
        let bytes = prefix.as_bytes();
        let start = bytes
            .get(..loc.tag_start)
            .and_then(|before| memchr::memrchr(b'\n', before))
            .map_or(0, |at| at + 1);
        match (prefix.get(start..), bytes.get(..start)) {
            (Some(lines), Some(before)) => Self {
                index: LineIndex::new(lines),
                start,
                line: u32::try_from(memchr::memchr_iter(b'\n', before).count()).unwrap_or(u32::MAX),
            },
            _ => Self {
                index: LineIndex::new(prefix),
                start: 0,
                line: 0,
            },
        }
    }

    pub(super) const fn index(&self) -> &LineIndex<'a> {
        &self.index
    }

    /// `loc` with offsets relative to the first indexed line. A block on the
    /// document's first line keeps its own location.
    pub(super) fn relative<'l>(&self, loc: &'l BlockLocation) -> Cow<'l, BlockLocation> {
        if self.start == 0 {
            return Cow::Borrowed(loc);
        }
        Cow::Owned(BlockLocation {
            start: loc.start.saturating_sub(self.start),
            end: loc.end.saturating_sub(self.start),
            tag_start: loc.tag_start.saturating_sub(self.start),
            tag_end: loc.tag_end.saturating_sub(self.start),
            ..loc.clone()
        })
    }

    /// Move a symbol tree read through [`Self::index`] onto document lines.
    pub(super) fn place(&self, symbol: &mut DocumentSymbol) {
        if self.line == 0 {
            return;
        }
        for position in [
            &mut symbol.range.start,
            &mut symbol.range.end,
            &mut symbol.selection_range.start,
            &mut symbol.selection_range.end,
        ] {
            position.line = position.line.saturating_add(self.line);
        }
        for child in symbol.children.iter_mut().flatten() {
            self.place(child);
        }
    }
}
