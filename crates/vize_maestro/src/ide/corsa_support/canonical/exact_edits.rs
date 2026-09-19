//! Writable ranges require a complete, length-preserving authored projection.

use tower_lsp::lsp_types::{Position, Range};

use super::CanonicalVirtualDocument;
use crate::ide::{
    IdeContext, diagnostics::VirtualTsResult, offset_to_position, position_to_offset,
};

pub(crate) fn map_canonical_exact_edit_range(
    ctx: &IdeContext<'_>,
    document: &CanonicalVirtualDocument,
    range: Range,
) -> Option<Range> {
    map_exact_range(&ctx.content, &document.virtual_result, range)
}

fn map_exact_range(source: &str, document: &VirtualTsResult, range: Range) -> Option<Range> {
    let start = position_to_offset(&document.code, range.start.line, range.start.character)?;
    let end = position_to_offset(&document.code, range.end.line, range.end.character)?;
    if start > end {
        return None;
    }
    let map = &document.import_source_map;
    let pre_start = map.get_original_offset(u32::try_from(start).ok()?) as usize;
    let pre_end = map.get_original_offset(u32::try_from(end).ok()?) as usize;
    if map.get_virtual_offset(pre_start as u32) as usize != start
        || map.get_virtual_offset(pre_end as u32) as usize != end
    {
        return None;
    }
    let spans = document.source_mappings.iter().flat_map(|mapping| {
        std::iter::once((&mapping.gen_range, &mapping.src_range)).chain(
            mapping
                .sub_spans
                .iter()
                .map(|span| (&span.gen_range, &span.src_range)),
        )
    });
    let mut authored = None;
    for (generated, original) in spans {
        if generated.len() != original.len()
            || generated.is_empty()
            || pre_start < generated.start
            || pre_end > generated.end
            || pre_start > pre_end
        {
            continue;
        }
        let source_start = original.start + pre_start - generated.start;
        let source_end = original.start + pre_end - generated.start;
        if source.get(source_start..source_end)? != document.code.get(start..end)? {
            continue;
        }
        let candidate = (source_start, source_end);
        if authored.is_some_and(|previous| previous != candidate) {
            return None;
        }
        authored = Some(candidate);
    }
    let (start, end) = authored?;
    let (line, character) = offset_to_position(source, start);
    let start = Position::new(line, character);
    let (line, character) = offset_to_position(source, end);
    Some(Range::new(start, Position::new(line, character)))
}

#[cfg(test)]
mod tests;
