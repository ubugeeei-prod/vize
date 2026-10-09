//! Style completion provider.
//!
//! Handles completions within `<style>` blocks including Vue CSS features.

use tower_lsp::lsp_types::CompletionItem;

use crate::ide::IdeContext;

/// Get completions for style context.
pub(crate) fn complete_style(ctx: &IdeContext, index: usize) -> Vec<CompletionItem> {
    crate::ide::style::complete(ctx, index)
}

/// Vue CSS feature completions.
#[cfg(any(test, feature = "native"))]
pub(crate) fn vue_css_completions() -> Vec<CompletionItem> {
    crate::ide::style::vue_completions(false)
}
