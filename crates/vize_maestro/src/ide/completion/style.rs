//! Style completion provider.
//!
//! Handles completions within `<style>` blocks including Vue CSS features.

use tower_lsp::lsp_types::CompletionItem;

use super::items;
use crate::ide::IdeContext;

/// Get completions for style context.
pub(crate) fn complete_style(_ctx: &IdeContext, _index: usize) -> Vec<CompletionItem> {
    vue_css_completions()
}

/// Vue CSS feature completions.
pub(crate) fn vue_css_completions() -> Vec<CompletionItem> {
    vec![
        items::css_item("v-bind", "v-bind()", "Dynamic CSS value", "v-bind($1)"),
        items::css_item(
            ":deep",
            ":deep()",
            "Deep selector in scoped CSS",
            ":deep($1)",
        ),
        items::css_item(
            ":slotted",
            ":slotted()",
            "Slotted content selector",
            ":slotted($1)",
        ),
        items::css_item(":global", ":global()", "Global selector", ":global($1)"),
    ]
}
