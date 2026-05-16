//! Style token collection for semantic highlighting.
//!
//! Handles CSS `v-bind()` expressions in `<style>` blocks.

use super::{
    encoding::offset_to_line_col,
    types::{AbsoluteToken, TokenType},
};

/// Collect tokens from style content.
pub(crate) fn collect_style_tokens(style: &str, base_line: u32, tokens: &mut Vec<AbsoluteToken>) {
    // Find v-bind() in CSS
    let pattern = "v-bind(";
    let mut pos = 0;
    while let Some(start) = style[pos..].find(pattern) {
        let abs_start = pos + start;
        let (line, col) = offset_to_line_col(style, abs_start);

        // Highlight v-bind
        tokens.push(AbsoluteToken {
            line: base_line + line - 1,
            start: col,
            length: 6, // "v-bind"
            token_type: TokenType::Function as u32,
            modifiers: 0,
        });

        // Find the variable inside
        if let Some(end) = style[abs_start + pattern.len()..].find(')') {
            let var_start = abs_start + pattern.len();
            let var = style[var_start..var_start + end].trim();
            let var = var.trim_matches(|c| c == '"' || c == '\'');

            if !var.is_empty() {
                let (var_line, var_col) = offset_to_line_col(style, var_start);
                tokens.push(AbsoluteToken {
                    line: base_line + var_line - 1,
                    start: var_col,
                    length: var.len() as u32,
                    token_type: TokenType::Variable as u32,
                    modifiers: 0,
                });
            }

            pos = var_start + end + 1;
        } else {
            break;
        }
    }
}
