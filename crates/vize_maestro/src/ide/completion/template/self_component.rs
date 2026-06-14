//! Inline `<art>` `Self` component metadata lookup.

use std::sync::Arc;

use super::component_meta::{ComponentMetadata, cached_component_metadata};
use crate::ide::IdeContext;

pub(super) fn metadata(ctx: &IdeContext, component_name: &str) -> Option<Arc<ComponentMetadata>> {
    if component_name != "Self"
        || ctx.uri.path().ends_with(".art.vue")
        || !matches!(
            ctx.block_type,
            Some(crate::virtual_code::BlockType::Art(
                crate::virtual_code::ArtCursorPosition::VariantTemplate(_)
            ))
        )
    {
        return None;
    }

    cached_component_metadata(ctx, &ctx.uri.to_file_path().ok()?)
}
