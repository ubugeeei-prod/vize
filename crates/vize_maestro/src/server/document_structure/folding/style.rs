//! Folding for authored style blocks: one region per multi-line block of the
//! L1 style structure, in every style language.

use tower_lsp::lsp_types::FoldingRange;
use vize_atelier_sfc::SfcStyleBlock;
use vize_l1::css::lang::StyleLanguage;

use super::region;

/// One region per multi-line block, in document order and O(n + r).
pub(super) fn style_regions(
    block: &SfcStyleBlock<'_>,
    content_start_line: u32,
) -> Vec<FoldingRange> {
    collect_style_regions(block, content_start_line).0
}

fn collect_style_regions(
    block: &SfcStyleBlock<'_>,
    content_start_line: u32,
) -> (Vec<FoldingRange>, usize) {
    if block.src.is_some() || block.content.is_empty() {
        return (Vec::new(), 0);
    }
    // A language without a dialect stays opaque.
    let Some(language) = StyleLanguage::from_lang(block.lang.as_deref()) else {
        return (Vec::new(), 0);
    };
    let source = block.content.as_ref();
    let bytes = source.as_bytes();
    let sheet = language.structure(source, 0);
    // Opens come in document order and every block closes before the next
    // node after it starts, so one forward cursor counts every line.
    let mut lines = Lines {
        bytes,
        cursor: 0,
        line: content_start_line,
    };
    // Reserve a slot at the open and fill it at the close, in document order.
    let mut ranges = Vec::new();
    let mut open: Vec<Pending> = Vec::new();
    for node in sheet.nodes() {
        let Some(span) = node.block() else {
            continue;
        };
        let braced = bytes.get(span.start as usize) == Some(&b'{');
        let start = if braced { span.start } else { node.prelude().end };
        close_until(&mut open, &mut ranges, &mut lines, start as usize);
        // A block the source never closes folds nothing.
        let closed = !braced
            || bytes.get((span.end as usize).wrapping_sub(1)) == Some(&b'}')
                && (sheet.is_balanced() || span.end as usize != source.len());
        open.push(Pending {
            end: span.end as usize,
            braced,
            closed,
            slot: ranges.len(),
            open_line: lines.at(start as usize),
        });
        ranges.push(None);
    }
    close_until(&mut open, &mut ranges, &mut lines, usize::MAX);
    let work = source.len() + sheet.nodes().len();
    (ranges.into_iter().flatten().collect(), work)
}

struct Lines<'a> {
    bytes: &'a [u8],
    cursor: usize,
    line: u32,
}

impl Lines<'_> {
    /// The line of `offset`, never before the last offset asked.
    fn at(&mut self, offset: usize) -> u32 {
        let end = offset.min(self.bytes.len());
        if end > self.cursor {
            let skipped = self.bytes.get(self.cursor..end).unwrap_or_default();
            self.line += memchr::memchr_iter(b'\n', skipped).count() as u32;
            self.cursor = end;
        }
        self.line
    }
}

struct Pending {
    end: usize,
    braced: bool,
    closed: bool,
    slot: usize,
    open_line: u32,
}

/// Fill the slots of the open blocks that end at or before `offset`.
fn close_until(
    open: &mut Vec<Pending>,
    ranges: &mut [Option<FoldingRange>],
    lines: &mut Lines<'_>,
    offset: usize,
) {
    while let Some(pending) = open.pop_if(|pending| pending.end <= offset) {
        if !pending.closed {
            continue;
        }
        // Braced: the line of `}`. Indented: one past the last nested line.
        let close = if pending.braced {
            lines.at(pending.end - 1)
        } else {
            lines.at(pending.end) + 1
        };
        if let Some(slot) = ranges.get_mut(pending.slot) {
            *slot = region(pending.open_line, close, None, None);
        }
    }
}

#[cfg(test)]
pub(super) fn style_regions_with_metrics(
    block: &SfcStyleBlock<'_>,
    content_start_line: u32,
) -> (Vec<FoldingRange>, usize) {
    collect_style_regions(block, content_start_line)
}
