//! Corsa-independent hover fallback for imported Vue SFC components.

use tower_lsp::lsp_types::{Hover, HoverContents};

use super::{authored_token_range, component_contract_markdown};
use crate::ide::{IdeContext, markup};

pub(in crate::ide::hover) fn vue_component_import_hover(
    ctx: &IdeContext<'_>,
    local_name: &str,
) -> Option<Hover> {
    Some(Hover {
        contents: HoverContents::Markup(markup::markdown_content(component_contract_markdown(
            ctx, local_name,
        )?)),
        range: authored_token_range(ctx),
    })
}
