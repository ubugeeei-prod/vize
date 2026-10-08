//! Style hover routing.

use super::{HoverBuilder, HoverService};
use crate::ide::IdeContext;
use tower_lsp::lsp_types::Hover;

impl HoverService {
    /// Get hover for style context.
    pub(super) fn hover_style(ctx: &IdeContext, _index: usize) -> Option<Hover> {
        let word = Self::get_word_at_offset(&ctx.content, ctx.offset);

        if word.is_empty() {
            return None;
        }

        // Check for Vue-specific CSS features
        if let Some(hover) = Self::hover_vue_css(&word) {
            return Some(hover);
        }

        None
    }

    /// Get hover for Vue CSS features.
    fn hover_vue_css(word: &str) -> Option<Hover> {
        let (title, description) = match word {
            "v-bind" => (
                "v-bind() in CSS",
                "Link CSS values to dynamic component state. The value will be compiled into a hashed CSS custom property.",
            ),
            ":deep" => (
                ":deep()",
                "Affects child component styles in scoped CSS. The selector inside `:deep()` will be compiled with the scoped attribute.",
            ),
            ":slotted" => (
                ":slotted()",
                "Target content passed via slots in scoped CSS. Only works inside scoped `<style>` blocks.",
            ),
            ":global" => (
                ":global()",
                "Apply styles globally, escaping the scoped CSS encapsulation.",
            ),
            _ => return None,
        };

        Some(
            HoverBuilder::new()
                .title(title)
                .meta("Vue SFC CSS feature")
                .description(description)
                .bullets(
                    "Behavior",
                    &[
                        "Applies during Vue SFC scoped CSS compilation.",
                        "Keep the selector explicit so the compiled output remains predictable.",
                    ],
                )
                .link(
                    "Vue SFC CSS Features",
                    "https://vuejs.org/api/sfc-css-features.html",
                )
                .build(),
        )
    }
}
