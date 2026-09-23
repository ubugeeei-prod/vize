//! Semantic tokens provider for syntax highlighting.
//!
//! Provides semantic tokens for:
//! - Template expressions and bindings
//! - Vue directives
//! - Script bindings
//! - CSS v-bind variables
//!
//! SFC blocks come from the resident descriptor (P5-6c): one parse per buffer
//! revision, shared with the other request paths. `.art.vue` files are
//! tokenized from the buffer and are not parsed as SFCs.

mod art;
mod encoding;
mod expressions;
mod style;
mod template;
mod template_attrs;
mod types;

#[cfg(test)]
#[expect(clippy::string_slice, reason = "tests assert by panicking")]
mod resident_tests;
#[cfg(test)]
mod tests;

pub use types::{TokenModifier, TokenType};

// Re-exported for the JSX/TSX semantic-tokens path (#1498), which tokenizes
// each re-emitted JSX expression directly in source coordinates using the same
// JS/TS expression tokenizer + delta encoder the SFC template path uses.
pub(crate) use encoding::encode_tokens as encode_semantic_tokens;
pub(crate) use expressions::tokenize_expression;
// `AbsoluteToken` is also used locally below; the `pub(crate) use` both brings
// it into this module's scope and re-exports it for the JSX path.
pub(crate) use types::AbsoluteToken;

use tower_lsp::lsp_types::{
    Range, SemanticTokens, SemanticTokensRangeResult, SemanticTokensResult, Url,
};

use crate::server::ServerState;
use encoding::{LineIndex, encode_tokens};

/// Semantic tokens service.
pub struct SemanticTokensService;

fn token_overlaps_range(token: &AbsoluteToken, range: Range) -> bool {
    if token.line < range.start.line || token.line > range.end.line {
        return false;
    }

    let token_end = token.start.saturating_add(token.length);

    if token.line == range.start.line && token_end <= range.start.character {
        return false;
    }

    if token.line == range.end.line && token.start >= range.end.character {
        return false;
    }

    true
}

impl SemanticTokensService {
    /// Semantic tokens for a document, from its resident descriptor.
    pub fn get_tokens(
        state: &ServerState,
        content: &str,
        uri: &Url,
    ) -> Option<SemanticTokensResult> {
        let tokens = Self::collect_tokens(state, content, uri)?;
        Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: encode_tokens(&tokens),
        }))
    }

    /// Semantic tokens for the visible range, from the same resident descriptor.
    pub fn get_tokens_range(
        state: &ServerState,
        content: &str,
        uri: &Url,
        range: Range,
    ) -> Option<SemanticTokensRangeResult> {
        let tokens = Self::collect_tokens(state, content, uri)?;
        let tokens = tokens
            .into_iter()
            .filter(|token| token_overlaps_range(token, range))
            .collect::<Vec<_>>();

        Some(SemanticTokensRangeResult::Tokens(SemanticTokens {
            result_id: None,
            data: encode_tokens(&tokens),
        }))
    }

    fn collect_tokens(state: &ServerState, content: &str, uri: &Url) -> Option<Vec<AbsoluteToken>> {
        // Art files are not SFC parses. Tokenize the buffer and do not touch
        // the resident tier.
        if uri.path().ends_with(".art.vue") {
            return Some(Self::collect_art_tokens(content));
        }

        let descriptor = state.sfc_descriptor(uri, content)?;
        Some(Self::tokens_from_descriptor(content, &descriptor))
    }

    fn tokens_from_descriptor(
        content: &str,
        descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
    ) -> Vec<AbsoluteToken> {
        let mut tokens: Vec<AbsoluteToken> = Vec::new();

        // Collect tokens from template
        if let Some(ref template) = descriptor.template {
            template::collect_template_tokens(
                &template.content,
                template.loc.start_line.saturating_sub(1) as u32,
                &mut tokens,
            );
        }

        // Collect tokens from script setup
        if let Some(ref script_setup) = descriptor.script_setup {
            template::collect_script_tokens(
                &script_setup.content,
                script_setup.loc.start_line.saturating_sub(1) as u32,
                &mut tokens,
            );
        }

        // Collect tokens from script
        if let Some(ref script) = descriptor.script {
            template::collect_script_tokens(
                &script.content,
                script.loc.start_line.saturating_sub(1) as u32,
                &mut tokens,
            );
        }

        // Collect tokens from styles
        for s in &descriptor.styles {
            style::collect_style_tokens(
                &s.content,
                s.loc.start_line.saturating_sub(1) as u32,
                &mut tokens,
            );
        }

        // Collect tokens from inline <art> custom blocks. Build the line index
        // once and share it across every art block instead of re-scanning the
        // document per offset.
        let has_art_block = descriptor
            .custom_blocks
            .iter()
            .any(|custom| custom.block_type == "art");
        if has_art_block {
            let line_index = LineIndex::new(content);
            for custom in &descriptor.custom_blocks {
                if custom.block_type == "art" {
                    Self::collect_inline_art_tokens(content, &mut tokens, &custom.loc, &line_index);
                }
            }
        }

        // Sort by position
        tokens.sort_by_key(|token| (token.line, token.start));

        tokens
    }
}
