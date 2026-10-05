//! Component attribute names require copied or producer-owned query coordinates.
//! Diagnostic whole-element rows cannot identify an attribute symbol.

use super::CanonicalVirtualDocument;
use crate::ide::{IdeContext, diagnostics::VirtualTsResult, offset_to_position};

mod prop_key;

/// `None` leaves ordinary expressions, events, models and native DOM routes
/// with their existing query policy. `Some(None)` rejects an unproven
/// component attribute; `Some(Some(position))` supplies its owned native key.
pub(crate) fn component_attribute_position(
    ctx: &IdeContext<'_>,
    document: &CanonicalVirtualDocument,
) -> Option<Option<(u32, u32)>> {
    let (_, component) = crate::ide::definition::helpers::get_non_model_attribute_at_offset(ctx)?;
    if !crate::ide::is_component_tag(&component) {
        return None;
    }
    let result = &document.virtual_result;
    Some(
        copied_position(&ctx.content, result, ctx.offset)
            .or_else(|| prop_key::source_position(&ctx.content, result, ctx.offset)),
    )
}

fn copied_position(source: &str, result: &VirtualTsResult, offset: usize) -> Option<(u32, u32)> {
    if !source.is_char_boundary(offset) {
        return None;
    }
    let map = &result.import_source_map;
    let spans = result.source_mappings.iter().flat_map(|mapping| {
        std::iter::once((&mapping.gen_range, &mapping.src_range)).chain(
            mapping
                .sub_spans
                .iter()
                .map(|span| (&span.gen_range, &span.src_range)),
        )
    });
    let mut candidate = None;
    for (generated, authored) in spans {
        if !authored.contains(&offset) || generated.len() != authored.len() {
            continue;
        }
        let start = u32::try_from(generated.start).ok()?;
        let end = u32::try_from(generated.end).ok()?;
        let native_start = map.get_virtual_offset(start);
        let native_end = map.get_virtual_offset(end);
        let Some(authored_text) = source.get(authored.clone()) else {
            continue;
        };
        let Some(generated_text) = result.code.get(native_start as usize..native_end as usize)
        else {
            continue;
        };
        if map.get_original_offset(native_start) != start
            || map.get_original_offset(native_end) != end
            || authored_text != generated_text
        {
            continue;
        }
        let point = u32::try_from(generated.start + offset - authored.start).ok()?;
        let native = map.get_virtual_offset(point);
        if map.get_original_offset(native) != point
            || !result.code.is_char_boundary(native as usize)
        {
            continue;
        }
        let position = offset_to_position(&result.code, native as usize);
        if candidate.is_some_and(|previous| previous != position) {
            return None;
        }
        candidate = Some(position);
    }
    candidate
}

#[cfg(test)]
mod tests;
