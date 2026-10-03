//! Strict original UTF-8 span to LSP UTF-16 conversion, including CRLF.
use super::NavigationRefusal;
use tower_lsp::lsp_types::{Position, Range};
use vize_l0::Span;

pub(super) fn offset(
    source: &str,
    lines: &[usize],
    position: Position,
) -> Result<usize, NavigationRefusal> {
    let start = *lines
        .get(position.line as usize)
        .ok_or(NavigationRefusal::Position)?;
    let end = lines
        .get(
            (position.line as usize)
                .checked_add(1)
                .ok_or(NavigationRefusal::Position)?,
        )
        .copied()
        .unwrap_or(source.len());
    let line = source
        .get(start..end)
        .ok_or(NavigationRefusal::Position)?
        .trim_end_matches(['\r', '\n']);
    let mut units = 0;
    for (byte, ch) in line.char_indices() {
        if units == position.character {
            return Ok(start + byte);
        }
        units += ch.len_utf16() as u32;
        if units > position.character {
            return Err(NavigationRefusal::Position);
        }
    }
    if units == position.character {
        Ok(start + line.len())
    } else {
        Err(NavigationRefusal::Position)
    }
}

pub(super) fn range(source: &str, lines: &[usize], span: Span) -> Result<Range, NavigationRefusal> {
    if span.start > span.end || source.get(span.start as usize..span.end as usize).is_none() {
        return Err(NavigationRefusal::Projection);
    }
    let position = |byte: u32| -> Result<Position, NavigationRefusal> {
        let line = lines
            .partition_point(|start| *start <= byte as usize)
            .saturating_sub(1);
        let start = *lines.get(line).ok_or(NavigationRefusal::Projection)?;
        let column = source
            .get(start..byte as usize)
            .ok_or(NavigationRefusal::Projection)?
            .encode_utf16()
            .count();
        Ok(Position::new(
            u32::try_from(line).map_err(|_| NavigationRefusal::Projection)?,
            u32::try_from(column).map_err(|_| NavigationRefusal::Projection)?,
        ))
    };
    Ok(Range::new(position(span.start)?, position(span.end)?))
}
