//! Inline `<art>` definition helpers.

use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Range};

use super::IdeContext;

pub(super) fn self_component_definition(ctx: &IdeContext<'_>) -> Option<GotoDefinitionResponse> {
    if ctx.uri.path().ends_with(".art.vue")
        || !matches!(
            ctx.block_type,
            Some(crate::virtual_code::BlockType::Art(
                crate::virtual_code::ArtCursorPosition::VariantTemplate(_)
            ))
        )
    {
        return None;
    }

    Some(GotoDefinitionResponse::Scalar(Location {
        uri: ctx.uri.clone(),
        range: Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 0,
            },
        },
    }))
}
