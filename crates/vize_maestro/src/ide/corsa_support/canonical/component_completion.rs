//! Stable prop completion anchors, independent of partially authored names.

use super::CanonicalVirtualDocument;
use vize_canon::virtual_ts::VizeSemanticLinkKind;

pub(crate) fn canonical_component_prop_position(
    document: &CanonicalVirtualDocument,
    tag_source_offset: usize,
) -> Option<(u32, u32)> {
    let result = &document.virtual_result;
    let link = result.semantic_links.iter().find(|link| {
        link.kind == VizeSemanticLinkKind::VueComponentPropCompletion
            && result.source_mappings.iter().any(|mapping| {
                tag_source_offset >= mapping.src_range.start
                    && tag_source_offset < mapping.src_range.end
                    && link.source_range.start
                        == result
                            .import_source_map
                            .get_virtual_offset(mapping.gen_range.start as u32)
                            as usize
            })
    })?;
    Some(crate::ide::offset_to_position(
        &result.code,
        link.target_range.start,
    ))
}
