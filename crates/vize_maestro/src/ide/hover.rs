//! Hover information provider.
//!
//! Contextual hover information for template expressions/bindings and Vue directives.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "tower_lsp::lsp_types payloads take std `String`, built with `to_string()` and `format!`"
)]

mod backend;
mod builder;
#[cfg(feature = "native")]
mod component_import;
mod component_prop;
mod component_tag;
#[cfg(feature = "native")]
mod corsa;
#[cfg(all(test, feature = "native"))]
mod corsa_tests;
mod declaration_keyword;
#[cfg(feature = "native")]
mod html;
mod petite_vue;
mod script;
mod script_type_infer;
mod style;
mod template;
#[cfg(feature = "native")]
mod v_model;

pub use builder::HoverBuilder;
#[cfg(feature = "native")]
use std::sync::Arc;
use tower_lsp::lsp_types::Hover;

use super::IdeContext;
use crate::virtual_code::{ArtCursorPosition, BlockType};
#[cfg(feature = "native")]
use vize_canon::CorsaBridge;
/// Hover service for providing contextual information.
pub struct HoverService;

impl HoverService {
    /// Get hover information for the given context.
    pub fn hover(ctx: &IdeContext) -> Option<Hover> {
        match ctx.block_type? {
            _ if crate::ide::template_expression::is_in_template_comment(ctx) => None,
            BlockType::Template => Self::hover_template(ctx),
            BlockType::Script => Self::hover_script(ctx, false),
            BlockType::ScriptSetup => Self::hover_script(ctx, true),
            BlockType::Style(index) => Self::hover_style(ctx, index),
            BlockType::Art(ArtCursorPosition::VariantTemplate(_)) => Self::hover_template(ctx),
            BlockType::Art(_) => None,
        }
    }

    /// Get hover information with Corsa support (async version).
    ///
    /// This method first tries to get type information from Corsa,
    /// then falls back to the synchronous analysis.
    #[cfg(feature = "native")]
    pub async fn hover_with_corsa(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<Hover> {
        match ctx.block_type? {
            _ if crate::ide::template_expression::is_in_template_comment(ctx) => None,
            BlockType::Template => Self::hover_template_with_corsa(ctx, corsa_bridge).await,
            BlockType::Script => Self::hover_script_with_corsa(ctx, false, corsa_bridge).await,
            BlockType::ScriptSetup => Self::hover_script_with_corsa(ctx, true, corsa_bridge).await,
            BlockType::Style(index) => Self::hover_style(ctx, index),
            BlockType::Art(ArtCursorPosition::VariantTemplate(ref info)) => {
                Self::hover_art_variant_with_corsa(ctx, info, corsa_bridge).await
            }
            BlockType::Art(_) => None,
        }
    }

    // Shared utilities.

    /// Get the word at a given offset.
    pub(super) fn get_word_at_offset(content: &str, offset: usize) -> String {
        crate::ide::token_at_offset(content, offset, Self::is_word_char).unwrap_or_default()
    }

    /// Check if a byte is a valid word character.
    #[inline]
    fn is_word_char(c: u8) -> bool {
        c.is_ascii_alphanumeric() || c == b'_' || c == b'-' || c == b'$' || c == b':'
    }
}

#[cfg(test)]
mod tests;
