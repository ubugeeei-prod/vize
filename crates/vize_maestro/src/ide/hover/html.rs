use std::sync::Arc;

use tower_lsp::lsp_types::Hover;
use vize_canon::CorsaBridge;

use super::HoverService;
use crate::ide::IdeContext;

impl HoverService {
    pub(super) async fn hover_html_tag_with_corsa(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<&Arc<CorsaBridge>>,
    ) -> Option<Hover> {
        let tag_name =
            crate::ide::definition::helpers::get_tag_at_offset(&ctx.content, ctx.offset)?;
        if crate::ide::is_component_tag(&tag_name) {
            return None;
        }

        let bridge = corsa_bridge?;
        if !bridge.is_initialized() {
            return None;
        }

        let doc = crate::ide::corsa_support::html_tag_virtual_document(&tag_name)?;
        let request_path = crate::ide::corsa_support::html_tag_request_path(ctx.uri);
        let request_uri = bridge
            .open_or_update_virtual_document(&request_path, &doc.content)
            .await
            .ok()?;
        let (line, character) = crate::ide::offset_to_position(&doc.content, doc.hover_offset);
        let hover = bridge.hover(&request_uri, line, character).await.ok()??;

        Some(Self::convert_lsp_hover(hover))
    }
}
