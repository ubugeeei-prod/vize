//! Whole producer-owned prop keys may change length when Vue camelizes them.

use std::ops::Range as OffsetRange;

use vize_canon::virtual_ts::{VizeSemanticLink, VizeSemanticLinkKind};

use crate::ide::{diagnostics::VirtualTsResult, offset_to_position};

pub(super) fn source_position(
    source: &str,
    result: &VirtualTsResult,
    offset: usize,
) -> Option<(u32, u32)> {
    if !source.is_char_boundary(offset) {
        return None;
    }
    let mut native = None;
    for link in &result.semantic_links {
        let Some(authored) = authored_range(source, result, link) else {
            continue;
        };
        if authored.contains(&offset) {
            let position = offset_to_position(&result.code, link.target_range.start);
            if native.is_some_and(|existing| existing != position) {
                return None;
            }
            native = Some(position);
        }
    }
    native
}

fn authored_range(
    source: &str,
    result: &VirtualTsResult,
    link: &VizeSemanticLink,
) -> Option<OffsetRange<usize>> {
    if link.kind != VizeSemanticLinkKind::VueComponentPropNavigation {
        return None;
    }
    let generated = result.code.get(link.target_range.clone())?;
    if generated.is_empty() || result.code.get(link.source_range.clone())?.is_empty() {
        return None;
    }
    let map = &result.import_source_map;
    let start = u32::try_from(link.target_range.start).ok()?;
    let end = u32::try_from(link.target_range.end).ok()?;
    let original_start = map.get_original_offset(start);
    let original_end = map.get_original_offset(end);
    if map.get_virtual_offset(original_start) != start
        || map.get_virtual_offset(original_end) != end
    {
        return None;
    }
    let generated_range = original_start as usize..original_end as usize;
    let mut authored = None;
    for mapping in &result.source_mappings {
        if mapping.gen_range != generated_range {
            continue;
        }
        let name = source.get(mapping.src_range.clone())?;
        if name.is_empty() || !link.matches_component_prop_navigation_name(name, generated) {
            continue;
        }
        if authored
            .as_ref()
            .is_some_and(|existing| *existing != mapping.src_range)
        {
            return None;
        }
        authored = Some(mapping.src_range.clone());
    }
    authored
}
