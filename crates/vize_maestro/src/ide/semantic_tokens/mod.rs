//! Semantic tokens provider for syntax highlighting.
//!
//! Provides semantic tokens for:
//! - Template expressions and bindings
//! - Vue directives
//! - Script bindings
//! - CSS v-bind variables
#![allow(clippy::disallowed_methods)]

mod encoding;
mod expressions;
mod style;
mod template;
mod types;

pub use types::{TokenModifier, TokenType};

use tower_lsp::lsp_types::SemanticTokensResult;

use encoding::{encode_tokens, offset_to_line_col};
use types::AbsoluteToken;

/// Semantic tokens service.
pub struct SemanticTokensService;

impl SemanticTokensService {
    /// Get semantic tokens for a document.
    pub fn get_tokens(
        content: &str,
        uri: &tower_lsp::lsp_types::Url,
    ) -> Option<SemanticTokensResult> {
        // Check if this is an Art file
        if uri.path().ends_with(".art.vue") {
            return Self::get_art_tokens(content);
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
                template.loc.start_line as u32,
                &mut tokens,
            );
        }

        // Collect tokens from script setup
        if let Some(ref script_setup) = descriptor.script_setup {
            template::collect_script_tokens(
                &script_setup.content,
                script_setup.loc.start_line as u32,
                &mut tokens,
            );
        }

        // Collect tokens from script
        if let Some(ref script) = descriptor.script {
            template::collect_script_tokens(
                &script.content,
                script.loc.start_line as u32,
                &mut tokens,
            );
        }

        // Collect tokens from styles
        for s in &descriptor.styles {
            style::collect_style_tokens(&s.content, s.loc.start_line as u32, &mut tokens);
        }

        // Collect tokens from inline <art> custom blocks
        for custom in &descriptor.custom_blocks {
            if custom.block_type == "art" {
                Self::collect_inline_art_tokens(content, &mut tokens, &custom.loc);
            }
        }

        // Sort by position
        tokens.sort_by_key(|token| (token.line, token.start));

        // Convert to delta encoding
        let semantic_tokens = encode_tokens(&tokens);

        Some(SemanticTokensResult::Tokens(
            tower_lsp::lsp_types::SemanticTokens {
                result_id: None,
                data: semantic_tokens,
            },
        ))
    }

    /// Get semantic tokens for Art files (*.art.vue).
    fn get_art_tokens(content: &str) -> Option<SemanticTokensResult> {
        let mut tokens: Vec<AbsoluteToken> = Vec::new();

        // Collect Art-specific tokens
        Self::collect_art_block_tokens(content, &mut tokens);
        Self::collect_variant_block_tokens(content, &mut tokens);
        Self::collect_art_attribute_tokens(content, &mut tokens);
        Self::collect_art_script_tokens(content, &mut tokens);

        // Sort by position
        tokens.sort_by_key(|token| (token.line, token.start));

        // Convert to delta encoding
        let semantic_tokens = encode_tokens(&tokens);

        Some(SemanticTokensResult::Tokens(
            tower_lsp::lsp_types::SemanticTokens {
                result_id: None,
                data: semantic_tokens,
            },
        ))
    }

    /// Collect <art> and </art> tag tokens.
    fn collect_art_block_tokens(content: &str, tokens: &mut Vec<AbsoluteToken>) {
        // Find <art ...> opening tags
        let mut pos = 0;
        while let Some(start) = content[pos..].find("<art") {
            let abs_start = pos + start;
            // Check if followed by space, newline, or >
            let next_char_pos = abs_start + 4;
            if next_char_pos < content.len() {
                let next_char = content.as_bytes()[next_char_pos];
                if next_char == b' '
                    || next_char == b'\n'
                    || next_char == b'\t'
                    || next_char == b'>'
                {
                    let (line, col) = offset_to_line_col(content, abs_start);
                    tokens.push(AbsoluteToken {
                        line,
                        start: col,
                        length: 4, // "<art"
                        token_type: TokenType::Keyword as u32,
                        modifiers: TokenModifier::encode(&[TokenModifier::Declaration]),
                    });
                }
            }
            pos = abs_start + 4;
        }

        // Find </art> closing tags
        pos = 0;
        while let Some(start) = content[pos..].find("</art>") {
            let abs_start = pos + start;
            let (line, col) = offset_to_line_col(content, abs_start);
            tokens.push(AbsoluteToken {
                line,
                start: col,
                length: 6, // "</art>"
                token_type: TokenType::Keyword as u32,
                modifiers: 0,
            });
            pos = abs_start + 6;
        }
    }

    /// Collect <variant> and </variant> tag tokens.
    fn collect_variant_block_tokens(content: &str, tokens: &mut Vec<AbsoluteToken>) {
        // Find <variant ...> opening tags
        let mut pos = 0;
        while let Some(start) = content[pos..].find("<variant") {
            let abs_start = pos + start;
            let next_char_pos = abs_start + 8;
            if next_char_pos < content.len() {
                let next_char = content.as_bytes()[next_char_pos];
                if next_char == b' '
                    || next_char == b'\n'
                    || next_char == b'\t'
                    || next_char == b'>'
                {
                    let (line, col) = offset_to_line_col(content, abs_start);
                    tokens.push(AbsoluteToken {
                        line,
                        start: col,
                        length: 8, // "<variant"
                        token_type: TokenType::Class as u32,
                        modifiers: TokenModifier::encode(&[TokenModifier::Declaration]),
                    });
                }
            }
            pos = abs_start + 8;
        }

        // Find </variant> closing tags
        pos = 0;
        while let Some(start) = content[pos..].find("</variant>") {
            let abs_start = pos + start;
            let (line, col) = offset_to_line_col(content, abs_start);
            tokens.push(AbsoluteToken {
                line,
                start: col,
                length: 10, // "</variant>"
                token_type: TokenType::Class as u32,
                modifiers: 0,
            });
            pos = abs_start + 10;
        }
    }

    /// Collect Art-specific attribute tokens.
    fn collect_art_attribute_tokens(content: &str, tokens: &mut Vec<AbsoluteToken>) {
        // Art block attributes
        let art_attrs = [
            "title",
            "description",
            "component",
            "category",
            "tags",
            "status",
            "order",
        ];
        // Variant block attributes
        let variant_attrs = ["name", "default", "args", "viewport", "skip-vrt"];

        // Find attributes and their values
        for attr in art_attrs.iter().chain(variant_attrs.iter()) {
            #[allow(clippy::disallowed_macros)]
            let pattern_eq = format!("{}=", attr);
            let mut pos = 0;
            while let Some(start) = content[pos..].find(pattern_eq.as_str()) {
                let abs_start = pos + start;

                // Check if preceded by whitespace (attribute context)
                if abs_start > 0 {
                    let before = content.as_bytes()[abs_start - 1];
                    if before == b' ' || before == b'\n' || before == b'\t' {
                        let (line, col) = offset_to_line_col(content, abs_start);

                        // Highlight attribute name
                        tokens.push(AbsoluteToken {
                            line,
                            start: col,
                            length: attr.len() as u32,
                            token_type: TokenType::Property as u32,
                            modifiers: 0,
                        });

                        // Find and highlight string value
                        let value_start = abs_start + attr.len() + 1; // after =
                        if value_start < content.len() {
                            let quote_char = content.as_bytes()[value_start];
                            if quote_char == b'"' || quote_char == b'\'' {
                                if let Some(end) =
                                    content[value_start + 1..].find(quote_char as char)
                                {
                                    let (val_line, val_col) =
                                        offset_to_line_col(content, value_start);
                                    tokens.push(AbsoluteToken {
                                        line: val_line,
                                        start: val_col,
                                        length: (end + 2) as u32, // include quotes
                                        token_type: TokenType::String as u32,
                                        modifiers: 0,
                                    });
                                }
                            }
                        }
                    }
                }
                pos = abs_start + attr.len();
            }
        }

        // Highlight 'default' as boolean attribute (no value)
        let mut pos = 0;
        while let Some(start) = content[pos..].find(" default") {
            let abs_start = pos + start + 1; // skip leading space
            let after_pos = abs_start + 7;

            // Check if followed by space, > or newline (boolean attribute)
            if after_pos < content.len() {
                let after = content.as_bytes()[after_pos];
                if after == b' '
                    || after == b'>'
                    || after == b'\n'
                    || after == b'\t'
                    || after == b'/'
                {
                    let (line, col) = offset_to_line_col(content, abs_start);
                    tokens.push(AbsoluteToken {
                        line,
                        start: col,
                        length: 7, // "default"
                        token_type: TokenType::Modifier as u32,
                        modifiers: 0,
                    });
                }
            }
            pos = abs_start + 7;
        }
    }

    /// Collect tokens from script in Art files.
    fn collect_art_script_tokens(content: &str, tokens: &mut Vec<AbsoluteToken>) {
        // Find script setup block
        if let Some(script_start) = content.find("<script") {
            if let Some(script_end) = content[script_start..].find("</script>") {
                let script_content_start = content[script_start..]
                    .find('>')
                    .map(|p| script_start + p + 1)
                    .unwrap_or(script_start);
                let script_content_end = script_start + script_end;

                if script_content_start < script_content_end {
                    let script_content = &content[script_content_start..script_content_end];
                    let base_offset = script_content_start;

                    // Highlight import keyword
                    let mut pos = 0;
                    while let Some(start) = script_content[pos..].find("import ") {
                        let abs_start = base_offset + pos + start;
                        let (line, col) = offset_to_line_col(content, abs_start);
                        tokens.push(AbsoluteToken {
                            line,
                            start: col,
                            length: 6, // "import"
                            token_type: TokenType::Keyword as u32,
                            modifiers: 0,
                        });
                        pos += start + 6;
                    }

                    // Highlight from keyword
                    pos = 0;
                    while let Some(start) = script_content[pos..].find(" from ") {
                        let abs_start = base_offset + pos + start + 1; // skip leading space
                        let (line, col) = offset_to_line_col(content, abs_start);
                        tokens.push(AbsoluteToken {
                            line,
                            start: col,
                            length: 4, // "from"
                            token_type: TokenType::Keyword as u32,
                            modifiers: 0,
                        });
                        pos += start + 5;
                    }

                    // Highlight string literals (import paths)
                    pos = 0;
                    while pos < script_content.len() {
                        let remaining = &script_content[pos..];
                        let quote_pos = remaining.find(['"', '\'']);
                        if let Some(start) = quote_pos {
                            let quote_char = remaining.as_bytes()[start];
                            let after_quote = &remaining[start + 1..];
                            if let Some(end) = after_quote.find(quote_char as char) {
                                let abs_start = base_offset + pos + start;
                                let (line, col) = offset_to_line_col(content, abs_start);
                                tokens.push(AbsoluteToken {
                                    line,
                                    start: col,
                                    length: (end + 2) as u32, // include quotes
                                    token_type: TokenType::String as u32,
                                    modifiers: 0,
                                });
                                pos += start + end + 2;
                            } else {
                                pos += start + 1;
                            }
                        } else {
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Collect semantic tokens for inline <art> blocks in regular .vue files.
    ///
    /// Scans the specified range of the content for <art>, </art>, <variant>,
    /// </variant> tags, and art-specific attributes.
    fn collect_inline_art_tokens(
        content: &str,
        tokens: &mut Vec<AbsoluteToken>,
        loc: &vize_atelier_sfc::BlockLocation,
    ) {
        let range_start = loc.tag_start;
        let range_end = loc.end;

        // Ensure we don't go out of bounds
        let range_end = range_end.min(content.len());
        if range_start >= range_end {
            return;
        }

        let slice = &content[range_start..range_end];

        // Collect <art> / </art> tokens
        {
            let mut pos = 0;
            while let Some(start) = slice[pos..].find("<art") {
                let abs_pos = range_start + pos + start;
                let next_pos = pos + start + 4;
                if next_pos < slice.len() {
                    let next_char = slice.as_bytes()[next_pos];
                    if next_char == b' '
                        || next_char == b'\n'
                        || next_char == b'\t'
                        || next_char == b'>'
                    {
                        let (line, col) = offset_to_line_col(content, abs_pos);
                        tokens.push(AbsoluteToken {
                            line,
                            start: col,
                            length: 4,
                            token_type: TokenType::Keyword as u32,
                            modifiers: TokenModifier::encode(&[TokenModifier::Declaration]),
                        });
                    }
                }
                pos = next_pos;
            }

            pos = 0;
            while let Some(start) = slice[pos..].find("</art>") {
                let abs_pos = range_start + pos + start;
                let (line, col) = offset_to_line_col(content, abs_pos);
                tokens.push(AbsoluteToken {
                    line,
                    start: col,
                    length: 6,
                    token_type: TokenType::Keyword as u32,
                    modifiers: 0,
                });
                pos += start + 6;
            }
        }

        // Collect <variant> / </variant> tokens
        {
            let mut pos = 0;
            while let Some(start) = slice[pos..].find("<variant") {
                let abs_pos = range_start + pos + start;
                let next_pos = pos + start + 8;
                if next_pos < slice.len() {
                    let next_char = slice.as_bytes()[next_pos];
                    if next_char == b' '
                        || next_char == b'\n'
                        || next_char == b'\t'
                        || next_char == b'>'
                    {
                        let (line, col) = offset_to_line_col(content, abs_pos);
                        tokens.push(AbsoluteToken {
                            line,
                            start: col,
                            length: 8,
                            token_type: TokenType::Class as u32,
                            modifiers: TokenModifier::encode(&[TokenModifier::Declaration]),
                        });
                    }
                }
                pos = next_pos;
            }

            pos = 0;
            while let Some(start) = slice[pos..].find("</variant>") {
                let abs_pos = range_start + pos + start;
                let (line, col) = offset_to_line_col(content, abs_pos);
                tokens.push(AbsoluteToken {
                    line,
                    start: col,
                    length: 10,
                    token_type: TokenType::Class as u32,
                    modifiers: 0,
                });
                pos += start + 10;
            }
        }

        // Collect art-specific attribute tokens in the slice
        let art_attrs = [
            "title",
            "description",
            "component",
            "category",
            "tags",
            "status",
            "order",
        ];
        let variant_attrs = ["name", "args", "viewport", "skip-vrt"];

        for attr in art_attrs.iter().chain(variant_attrs.iter()) {
            #[allow(clippy::disallowed_macros)]
            let pattern_eq = format!("{}=", attr);
            let mut pos = 0;
            while let Some(start) = slice[pos..].find(pattern_eq.as_str()) {
                let rel_pos = pos + start;
                let abs_pos = range_start + rel_pos;

                if rel_pos > 0 {
                    let before = slice.as_bytes()[rel_pos - 1];
                    if before == b' ' || before == b'\n' || before == b'\t' {
                        let (line, col) = offset_to_line_col(content, abs_pos);
                        tokens.push(AbsoluteToken {
                            line,
                            start: col,
                            length: attr.len() as u32,
                            token_type: TokenType::Property as u32,
                            modifiers: 0,
                        });

                        // Highlight string value
                        let value_start = rel_pos + attr.len() + 1;
                        if value_start < slice.len() {
                            let quote_char = slice.as_bytes()[value_start];
                            if quote_char == b'"' || quote_char == b'\'' {
                                if let Some(end) = slice[value_start + 1..].find(quote_char as char)
                                {
                                    let abs_val = range_start + value_start;
                                    let (val_line, val_col) = offset_to_line_col(content, abs_val);
                                    tokens.push(AbsoluteToken {
                                        line: val_line,
                                        start: val_col,
                                        length: (end + 2) as u32,
                                        token_type: TokenType::String as u32,
                                        modifiers: 0,
                                    });
                                }
                            }
                        }
                    }
                }
                pos = rel_pos + attr.len();
            }
        }

        // Highlight 'default' boolean attribute
        {
            let mut pos = 0;
            while let Some(start) = slice[pos..].find(" default") {
                let rel_pos = pos + start + 1; // skip leading space
                let abs_pos = range_start + rel_pos;
                let after_pos = rel_pos + 7;

                if after_pos < slice.len() {
                    let after = slice.as_bytes()[after_pos];
                    if after == b' '
                        || after == b'>'
                        || after == b'\n'
                        || after == b'\t'
                        || after == b'/'
                    {
                        let (line, col) = offset_to_line_col(content, abs_pos);
                        tokens.push(AbsoluteToken {
                            line,
                            start: col,
                            length: 7,
                            token_type: TokenType::Modifier as u32,
                            modifiers: 0,
                        });
                    }
                }
                pos = rel_pos + 7;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        encoding::offset_to_line_col, expressions, template, SemanticTokensService, TokenModifier,
        TokenType,
    };
    use tower_lsp::lsp_types::SemanticTokensResult;

    #[test]
    fn test_extract_identifiers() {
        let expr = "count + message.length";
        let idents = expressions::extract_identifiers(expr);
        assert_eq!(idents.len(), 3);
        assert_eq!(idents[0].0, "count");
        assert_eq!(idents[1].0, "message");
        assert_eq!(idents[2].0, "length");
    }

    #[test]
    fn test_looks_like_function_call() {
        let expr = "handleClick()";
        assert!(expressions::looks_like_function_call(expr, 0));

        let expr = "count + 1";
        assert!(!expressions::looks_like_function_call(expr, 0));
    }

    #[test]
    fn test_offset_to_line_col() {
        let source = "abc\ndef\nghi";
        assert_eq!(offset_to_line_col(source, 0), (1, 0));
        assert_eq!(offset_to_line_col(source, 4), (2, 0));
        assert_eq!(offset_to_line_col(source, 8), (3, 0));
    }

    #[test]
    fn test_token_modifier_encode() {
        let modifiers = vec![TokenModifier::Declaration, TokenModifier::Readonly];
        let encoded = TokenModifier::encode(&modifiers);
        assert_eq!(encoded, 0b101); // bits 0 and 2
    }

    #[test]
    fn test_art_tokens_basic() {
        let content = r#"<art title="Button" component="./Button.vue">
  <variant name="Primary" default>
    <Button>Click</Button>
  </variant>
</art>

<script setup>
import Button from './Button.vue'
</script>"#;

        let result = SemanticTokensService::get_art_tokens(content);
        assert!(result.is_some());

        if let Some(SemanticTokensResult::Tokens(tokens)) = result {
            assert!(!tokens.data.is_empty());
        }
    }

    #[test]
    fn test_art_block_tokens() {
        let content = "<art title=\"Test\">\n</art>";
        let mut tokens = Vec::new();
        SemanticTokensService::collect_art_block_tokens(content, &mut tokens);

        // Should find <art and </art>
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].length, 4); // "<art"
        assert_eq!(tokens[1].length, 6); // "</art>"
    }

    #[test]
    fn test_variant_block_tokens() {
        let content = "<variant name=\"Primary\">\n</variant>";
        let mut tokens = Vec::new();
        SemanticTokensService::collect_variant_block_tokens(content, &mut tokens);

        // Should find <variant and </variant>
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].length, 8); // "<variant"
        assert_eq!(tokens[1].length, 10); // "</variant>"
    }

    #[test]
    fn test_art_attribute_tokens() {
        let content = r#"<art title="Button" component="./Button.vue">"#;
        let mut tokens = Vec::new();
        SemanticTokensService::collect_art_attribute_tokens(content, &mut tokens);

        // Should find title, "Button", component, "./Button.vue"
        assert!(tokens.len() >= 4);
    }

    #[test]
    fn test_art_script_tokens() {
        let content = r#"<script setup>
import Button from './Button.vue'
</script>"#;
        let mut tokens = Vec::new();
        SemanticTokensService::collect_art_script_tokens(content, &mut tokens);

        // Should find import, from, and string literal
        assert!(tokens.len() >= 3);
    }

    #[test]
    fn test_interpolation_tokens() {
        let template_str = "  {{ message }}";
        let mut tokens = Vec::new();
        template::collect_interpolation_tokens(template_str, 1, &mut tokens);

        // Should find 'message' as a variable
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::Variable as u32);
        assert_eq!(tokens[0].length, 7); // "message"
    }

    #[test]
    fn test_full_sfc_semantic_tokens() {
        let content = r#"<template>
  <div>{{ count }}</div>
</template>

<script setup>
const count = ref(0)
</script>
"#;

        let uri = tower_lsp::lsp_types::Url::parse("file:///test.vue").unwrap();
        let result = SemanticTokensService::get_tokens(content, &uri);
        assert!(result.is_some());

        if let Some(SemanticTokensResult::Tokens(tokens)) = result {
            // Should have tokens for:
            // - 'count' in template interpolation
            // - 'ref' in script
            assert!(!tokens.data.is_empty(), "Should have semantic tokens");
        }
    }

    #[test]
    fn test_directive_expression_tokenization() {
        let template_str =
            r#"<div v-if="todoGuards.isActive(todo) || todoGuards.isCompleted(todo)"></div>"#;
        let mut tokens = Vec::new();
        template::collect_directive_expression_tokens(template_str, 1, &mut tokens);

        // Debug: print all tokens
        for token in &tokens {
            eprintln!(
                "Token: line={}, start={}, length={}, type={}",
                token.line, token.start, token.length, token.token_type
            );
        }

        // Should find tokens for the expression:
        // - todoGuards (variable)
        // - isActive (function)
        // - todo (variable)
        // - || (operator)
        // - todoGuards (variable)
        // - isCompleted (function)
        // - todo (variable)
        assert!(
            tokens.len() >= 7,
            "Expected at least 7 tokens, got {}",
            tokens.len()
        );

        // Check that we have both variable and function tokens
        let has_variable = tokens
            .iter()
            .any(|t| t.token_type == TokenType::Variable as u32);
        let has_function = tokens
            .iter()
            .any(|t| t.token_type == TokenType::Function as u32);
        let has_operator = tokens
            .iter()
            .any(|t| t.token_type == TokenType::Operator as u32);

        assert!(has_variable, "Should have variable tokens");
        assert!(has_function, "Should have function tokens");
        assert!(has_operator, "Should have operator tokens");
    }

    #[test]
    fn test_tokenize_expression() {
        let expr = "todoGuards.isActive(todo) || todoGuards.isCompleted(todo)";
        let template_str = expr; // Use the expression as the "template" for position calculation
        let mut tokens = Vec::new();
        expressions::tokenize_expression(expr, template_str, 0, 1, &mut tokens);

        // Debug: print all tokens
        for token in &tokens {
            let token_name = match token.token_type {
                x if x == TokenType::Variable as u32 => "Variable",
                x if x == TokenType::Function as u32 => "Function",
                x if x == TokenType::Property as u32 => "Property",
                x if x == TokenType::Operator as u32 => "Operator",
                x if x == TokenType::Keyword as u32 => "Keyword",
                x if x == TokenType::Number as u32 => "Number",
                x if x == TokenType::String as u32 => "String",
                _ => "Unknown",
            };
            eprintln!(
                "Token: start={}, length={}, type={} ({})",
                token.start, token.length, token.token_type, token_name
            );
        }

        // Count token types
        let var_count = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Variable as u32)
            .count();
        let func_count = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Function as u32)
            .count();
        let prop_count = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Property as u32)
            .count();
        let op_count = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Operator as u32)
            .count();

        eprintln!(
            "Variables: {}, Functions: {}, Properties: {}, Operators: {}",
            var_count, func_count, prop_count, op_count
        );

        // We expect:
        // - todoGuards (variable) x2
        // - isActive (function) - after dot, so might be property
        // - isCompleted (function) - after dot, so might be property
        // - todo (variable) x2
        // - || (operator)
        assert!(tokens.len() >= 7, "Expected at least 7 tokens");
    }

    #[test]
    fn test_inline_art_tokens_in_vue() {
        let content = r#"<template>
  <div>test</div>
</template>

<script setup>
const x = 1
</script>

<art title="Button" component="./Button.vue">
  <variant name="Primary" default>
    <Button>Click</Button>
  </variant>
</art>"#;

        let uri = tower_lsp::lsp_types::Url::parse("file:///test.vue").unwrap();
        let result = SemanticTokensService::get_tokens(content, &uri);
        assert!(result.is_some());

        if let Some(SemanticTokensResult::Tokens(tokens)) = result {
            assert!(!tokens.data.is_empty(), "Should have inline art tokens");

            // Verify we have enough tokens (at least art/variant tags + attributes)
            assert!(
                tokens.data.len() >= 4,
                "Expected at least 4 tokens for inline art, got {}",
                tokens.data.len()
            );
        }
    }
}
