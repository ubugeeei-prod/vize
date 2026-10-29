//! Typed queries inside a standalone Art `<script setup>`.
//!
//! The canonical Vue opener refuses `.art.vue` containers. Completion already
//! answers from the Art setup projection, so hover and definition reuse it
//! instead of falling back to spelling.

use tower_lsp::lsp_types::{Position, Range};
use vize_canon::{CorsaBridge, LspRange};
use vize_l0::String;

use crate::ide::{IdeContext, offset_to_position};
use crate::virtual_code::{BlockType, VirtualDocument};

/// An authored Art setup position, opened in the native session.
pub(crate) struct ArtScriptQuery<'a> {
    pub(crate) document: &'a VirtualDocument,
    pub(crate) request_uri: String,
    pub(crate) line: u32,
    pub(crate) character: u32,
}

impl ArtScriptQuery<'_> {
    /// Map a range in this projection back to the authored Art file.
    pub(crate) fn authored_range(&self, ctx: &IdeContext<'_>, range: &LspRange) -> Option<Range> {
        super::map_virtual_range(
            ctx,
            self.document,
            &Range {
                start: Position::new(range.start.line, range.start.character),
                end: Position::new(range.end.line, range.end.character),
            },
        )
    }
}

pub(crate) async fn open_art_script_query<'a>(
    ctx: &'a IdeContext<'_>,
    bridge: &CorsaBridge,
) -> Option<ArtScriptQuery<'a>> {
    if !ctx.uri.path().ends_with(".art.vue")
        || !matches!(ctx.block_type, Some(BlockType::ScriptSetup))
        || !bridge.is_initialized()
    {
        return None;
    }
    let document = ctx.virtual_docs.as_ref()?.script_setup.as_ref()?;
    let offset = document.source_map.to_generated(ctx.offset)?;
    let (line, character) = offset_to_position(&document.content, offset);
    let request_uri = bridge
        .open_or_update_virtual_document(
            &super::script_request_path(ctx.uri, true),
            &document.content,
        )
        .await
        .ok()?;
    Some(ArtScriptQuery {
        document,
        request_uri,
        line,
        character,
    })
}
