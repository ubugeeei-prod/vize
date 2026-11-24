//! Tag positions from the descriptor and the original UTF-8 prefix.
use super::SfcScriptBlock;
use tower_lsp::lsp_types::{Position, Range};
use vize_l0::line_index::utf16_len;

pub(super) fn tag_ranges(block: &SfcScriptBlock<'_>, prefix: &str) -> Option<(Range, Range)> {
    let loc = &block.loc;
    let opening = prefix.get(loc.tag_start..loc.start)?;
    let closing = prefix.get(loc.end..loc.tag_end)?;
    if !opening.get(..7)?.eq_ignore_ascii_case("<script") {
        return None;
    }
    let before = prefix.get(..loc.tag_start)?;
    let (start, end) = if loc.end_line == 1 && !closing.as_bytes().contains(&b'\n') {
        // The first line starts at byte zero. Measure its two disjoint spans once,
        // including astral characters that occupy two UTF-16 units.
        let start = utf16_len(before) as u32;
        let end = start + utf16_len(prefix.get(loc.tag_start..loc.tag_end)?) as u32;
        (Position::new(0, start), Position::new(0, end))
    } else {
        // The descriptor records one-based content lines. Count only tag text
        // between each recorded boundary and the requested position.
        let start_line = loc
            .start_line
            .checked_sub(1 + memchr::memchr_iter(b'\n', opening.as_bytes()).count())?;
        let end_line =
            loc.end_line.checked_sub(1)? + memchr::memchr_iter(b'\n', closing.as_bytes()).count();
        (
            position_on_line(before, start_line)?,
            position_on_line(prefix.get(..loc.tag_end)?, end_line)?,
        )
    };
    let name_start = Position::new(start.line, start.character + 1);
    let name_end = Position::new(start.line, start.character + 7);
    Some((Range::new(start, end), Range::new(name_start, name_end)))
}

fn position_on_line(prefix: &str, line: usize) -> Option<Position> {
    let start = memchr::memrchr(b'\n', prefix.as_bytes()).map_or(0, |at| at + 1);
    Some(Position::new(
        line as u32,
        utf16_len(prefix.get(start..)?) as u32,
    ))
}
