//! Semantic tokens provider for syntax highlighting.
//!
//! Provides semantic tokens for:
//! - Template expressions and bindings
//! - Vue directives
//! - Script bindings
//! - CSS v-bind variables
#![allow(clippy::disallowed_methods)]

mod art;
mod encoding;
mod expressions;
mod style;
mod template;
mod types;

#[cfg(test)]
mod tests;

pub use types::{TokenModifier, TokenType};

use tower_lsp::lsp_types::{
    Range, SemanticTokens, SemanticTokensRangeResult, SemanticTokensResult,
};

use encoding::{LineIndex, encode_tokens};
use types::AbsoluteToken;

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
    /// Get semantic tokens for a document.
    pub fn get_tokens(
        content: &str,
        uri: &tower_lsp::lsp_types::Url,
    ) -> Option<SemanticTokensResult> {
        let tokens = Self::collect_tokens(content, uri)?;
        Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: encode_tokens(&tokens),
        }))
    }

    /// Get semantic tokens for the visible range of a document.
    pub fn get_tokens_range(
        content: &str,
        uri: &tower_lsp::lsp_types::Url,
        range: Range,
    ) -> Option<SemanticTokensRangeResult> {
        let tokens = Self::collect_tokens(content, uri)?;
        let tokens = tokens
            .into_iter()
            .filter(|token| token_overlaps_range(token, range))
            .collect::<Vec<_>>();

        Some(SemanticTokensRangeResult::Tokens(SemanticTokens {
            result_id: None,
            data: encode_tokens(&tokens),
        }))
    }

    fn collect_tokens(
        content: &str,
        uri: &tower_lsp::lsp_types::Url,
    ) -> Option<Vec<AbsoluteToken>> {
        // Check if this is an Art file
        if uri.path().ends_with(".art.vue") {
            return Some(Self::collect_art_tokens(content));
        }

        let options = vize_atelier_sfc::SfcParseOptions {
            filename: uri.path().to_string().into(),
            ..Default::default()
        };

        let descriptor = vize_atelier_sfc::parse_sfc(content, options).ok()?;

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

        Some(tokens)
    }
}
