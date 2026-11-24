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
    // The descriptor records one-based lines at the content boundaries using LF.
    // Only the tag text lies between each boundary and its requested position.
    let start_line = loc
        .start_line
        .checked_sub(1 + memchr::memchr_iter(b'\n', opening.as_bytes()).count())?;
    let end_line =
        loc.end_line.checked_sub(1)? + memchr::memchr_iter(b'\n', closing.as_bytes()).count();
    let start = position_on_line(prefix.get(..loc.tag_start)?, start_line)?;
    let end = position_on_line(prefix.get(..loc.tag_end)?, end_line)?;
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
