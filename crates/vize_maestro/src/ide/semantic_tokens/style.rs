//! Style token collection for semantic highlighting.
//!
//! Handles CSS `v-bind()` expressions in `<style>` blocks.

use super::{
    encoding::{offset_to_line_col, utf16_len},
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
            line: base_line + line,
            start: col,
            length: 6, // "v-bind"
            token_type: TokenType::Function as u32,
            modifiers: 0,
        });

        // Find the variable inside
        if let Some(end) = style[abs_start + pattern.len()..].find(')') {
            let var_start = abs_start + pattern.len();
            let raw_var = &style[var_start..var_start + end];
            let leading_ws = raw_var.len() - raw_var.trim_start().len();
            let trimmed = raw_var.trim();
            let leading_quote = usize::from(
                trimmed
                    .as_bytes()
                    .first()
                    .is_some_and(|quote| *quote == b'"' || *quote == b'\''),
            );
            let var = trimmed.trim_matches(|c| c == '"' || c == '\'');

            if !var.is_empty() {
                let var_offset = var_start + leading_ws + leading_quote;
                let (var_line, var_col) = offset_to_line_col(style, var_offset);
                tokens.push(AbsoluteToken {
                    line: base_line + var_line,
                    start: var_col,
                    length: utf16_len(var),
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
