//! Style hover routing.

use super::HoverService;
use crate::ide::IdeContext;
use tower_lsp::lsp_types::Hover;

impl HoverService {
    /// Get hover for style context.
    pub(super) fn hover_style(ctx: &IdeContext, index: usize) -> Option<Hover> {
        crate::ide::style::hover(ctx, index)
    }
}
