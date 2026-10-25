//! Source-owned intrinsic targets share the existing DOM hover renderer.

use super::MaestroServer;
#[cfg(not(feature = "experimental-source-navigation"))]
use tower_lsp::lsp_types::Url;

impl MaestroServer {
    #[cfg(not(feature = "experimental-source-navigation"))]
    pub(super) fn notify_retained_navigation(&self, uri: &Url) {
        if !crate::utils::is_jsx_path(uri.path()) {
            return;
        }
        if let Some(project) = &self.navigation {
            project.notify_host_change(uri);
        }
    }

    #[cfg(feature = "native")]
    pub(super) async fn jsx_intrinsic_hover(
        &self,
        ctx: &crate::ide::IdeContext<'_>,
        position: tower_lsp::lsp_types::Position,
    ) -> Option<tower_lsp::lsp_types::Hover> {
        let project = self.navigation.as_deref()?;
        let target = project.intrinsic_tag(ctx.uri, position).await.ok()??;
        let bridge = self.state.get_corsa_bridge().await;
        if let Some(hover) = crate::ide::HoverService::hover_dom_tag_at_with_corsa(
            ctx.uri,
            &target.name,
            target.range,
            bridge.as_ref(),
        )
        .await
        {
            return Some(hover);
        }
        crate::ide::HoverService::hover_dom_tag_at(&target.name, target.range)
    }
}
