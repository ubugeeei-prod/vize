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

use tower_lsp::lsp_types::{
    Range, SemanticTokens, SemanticTokensRangeResult, SemanticTokensResult,
};

use encoding::{LineIndex, encode_tokens, utf16_len};
use types::AbsoluteToken;

/// Semantic tokens service.
pub struct SemanticTokensService;

/// Art-specific attribute names highlighted as `name="value"` in `.art.vue`
/// files. Combines the `<art>` block attributes with the `<variant>` block
/// attributes (including the valued `default="..."` form). Built once as a
/// `const` instead of allocating a `format!("{name}=")` string per attribute
/// on every request.
///
/// Invariant: no entry is a suffix of another, so at most one name matches a
/// given `=`. This keeps the single-pass scan equivalent to the previous
/// per-attribute `content.find` loops.
const ART_FILE_ATTR_NAMES: &[&str] = &[
    "title",
    "description",
    "component",
    "category",
    "tags",
    "status",
    "order",
    "name",
    "default",
    "args",
    "viewport",
    "skip-vrt",
];

/// Art-specific attribute names highlighted as `name="value"` in inline
/// `<art>` blocks of regular `.vue` files. Unlike [`ART_FILE_ATTR_NAMES`],
/// the inline path never treats `default="..."` as a valued attribute (it is
/// only ever highlighted as a boolean modifier), matching prior behavior.
const INLINE_ART_ATTR_NAMES: &[&str] = &[
    "title",
    "description",
    "component",
    "category",
    "tags",
    "status",
    "order",
    "name",
    "args",
    "viewport",
    "skip-vrt",
];

/// Scan `slice` once for `name="value"` art attributes, emitting a `Property`
/// token for each known attribute name preceded by whitespace and a `String`
/// token for its quoted value. `range_start` is the byte offset of `slice`
/// within the document `line_index` was built for (0 when `slice` is the whole
/// document).
///
/// This collapses the previous N full `slice.find("{name}=")` scans (one per
/// attribute) into a single pass over the `=` bytes: on each `=` it looks
/// backward for a known attribute name from `attr_names`. Because no attribute
/// name is a suffix of another, at most one name matches a given `=`, so the
/// emitted token set is identical to the per-attribute loops.
fn collect_named_attribute_tokens(
    slice: &str,
    range_start: usize,
    attr_names: &[&str],
    line_index: &LineIndex<'_>,
    tokens: &mut Vec<AbsoluteToken>,
) {
    let bytes = slice.as_bytes();
    for (eq, &byte) in bytes.iter().enumerate() {
        if byte != b'=' {
            continue;
        }

        for attr in attr_names {
            let len = attr.len();
            // Attribute name must end exactly at `=` and have a whitespace
            // character before it (so `eq` must be at least `len + 1`).
            if eq < len + 1 {
                continue;
            }
            let name_start = eq - len;
            // Compare bytes (attribute names are ASCII) so a `name_start` that
            // happens to fall inside a multi-byte UTF-8 character never panics
            // the way string slicing would; non-matching bytes simply skip.
            if &bytes[name_start..eq] != attr.as_bytes() {
                continue;
            }
            let before = bytes[name_start - 1];
            if before != b' ' && before != b'\n' && before != b'\t' {
                continue;
            }

            // Highlight attribute name.
            let (line, col) = line_index.line_col(range_start + name_start);
            tokens.push(AbsoluteToken {
                line,
                start: col,
                length: utf16_len(attr),
                token_type: TokenType::Property as u32,
                modifiers: 0,
            });

            // Highlight quoted string value, if present.
            let value_start = eq + 1; // after `=`
            if value_start < slice.len() {
                let quote_char = bytes[value_start];
                if (quote_char == b'"' || quote_char == b'\'')
                    && let Some(end) = slice[value_start + 1..].find(quote_char as char)
                {
                    let (val_line, val_col) = line_index.line_col(range_start + value_start);
                    tokens.push(AbsoluteToken {
                        line: val_line,
                        start: val_col,
                        length: utf16_len(&slice[value_start..value_start + end + 2]),
                        token_type: TokenType::String as u32,
                        modifiers: 0,
                    });
                }
            }

            // At most one attribute name matches a given `=`.
            break;
        }
    }
}

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

    fn collect_art_tokens(content: &str) -> Vec<AbsoluteToken> {
        let mut tokens: Vec<AbsoluteToken> = Vec::new();

        // Build the line index once and share it across every collector below.
        let line_index = LineIndex::new(content);

        // Collect Art-specific tokens
        Self::collect_art_block_tokens(content, &mut tokens, &line_index);
        Self::collect_variant_block_tokens(content, &mut tokens, &line_index);
        Self::collect_art_attribute_tokens(content, &mut tokens, &line_index);
        Self::collect_art_variant_template_tokens(content, &mut tokens, &line_index);
        Self::collect_art_script_tokens(content, &mut tokens, &line_index);

        // Sort by position
        tokens.sort_by_key(|token| (token.line, token.start));

        tokens
    }

    /// Collect <art> and </art> tag tokens.
    fn collect_art_block_tokens(
        content: &str,
        tokens: &mut Vec<AbsoluteToken>,
        line_index: &LineIndex<'_>,
    ) {
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
                    let (line, col) = line_index.line_col(abs_start);
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
            let (line, col) = line_index.line_col(abs_start);
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
    fn collect_variant_block_tokens(
        content: &str,
        tokens: &mut Vec<AbsoluteToken>,
        line_index: &LineIndex<'_>,
    ) {
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
                    let (line, col) = line_index.line_col(abs_start);
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
            let (line, col) = line_index.line_col(abs_start);
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
    fn collect_art_attribute_tokens(
        content: &str,
        tokens: &mut Vec<AbsoluteToken>,
        line_index: &LineIndex<'_>,
    ) {
        // Find attributes and their values in a single pass (see
        // `collect_named_attribute_tokens`). `content` is the whole document,
        // so the slice offset is 0.
        collect_named_attribute_tokens(content, 0, ART_FILE_ATTR_NAMES, line_index, tokens);

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
                    let (line, col) = line_index.line_col(abs_start);
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

    /// Collect Vue template semantic tokens from each `<variant>` body in an `.art.vue` file.
    fn collect_art_variant_template_tokens(
        content: &str,
        tokens: &mut Vec<AbsoluteToken>,
        line_index: &LineIndex<'_>,
    ) {
        let allocator = vize_carton::Bump::new();
        let Ok(art_desc) =
            vize_musea::parse_art(&allocator, content, vize_musea::ArtParseOptions::default())
        else {
            return;
        };

        for variant in art_desc.variants.iter() {
            Self::collect_template_slice_tokens(content, variant.template, tokens, line_index);
        }
    }

    fn collect_template_slice_tokens(
        full_content: &str,
        template_slice: &str,
        tokens: &mut Vec<AbsoluteToken>,
        line_index: &LineIndex<'_>,
    ) {
        if template_slice.trim().is_empty() {
            return;
        }

        let source_ptr = full_content.as_ptr() as usize;
        let template_ptr = template_slice.as_ptr() as usize;
        let Some(start_offset) = template_ptr.checked_sub(source_ptr) else {
            return;
        };
        if start_offset > full_content.len() {
            return;
        }

        let (base_line, base_col) = line_index.line_col(start_offset);
        let mut local_tokens = Vec::new();
        template::collect_template_tokens(template_slice, 0, &mut local_tokens);

        for mut token in local_tokens {
            if token.line == 0 {
                token.start = token.start.saturating_add(base_col);
            }
            token.line = token.line.saturating_add(base_line);
            tokens.push(token);
        }
    }

    /// Collect tokens from script in Art files.
    fn collect_art_script_tokens(
        content: &str,
        tokens: &mut Vec<AbsoluteToken>,
        line_index: &LineIndex<'_>,
    ) {
        // Find script setup block
        if let Some(script_start) = content.find("<script")
            && let Some(script_end) = content[script_start..].find("</script>")
        {
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
                    let (line, col) = line_index.line_col(abs_start);
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
                    let (line, col) = line_index.line_col(abs_start);
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
                            let (line, col) = line_index.line_col(abs_start);
                            tokens.push(AbsoluteToken {
                                line,
                                start: col,
                                length: utf16_len(&remaining[start..start + end + 2]),
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

    /// Collect semantic tokens for inline <art> blocks in regular .vue files.
    ///
    /// Scans the specified range of the content for <art>, </art>, <variant>,
    /// </variant> tags, and art-specific attributes.
    fn collect_inline_art_tokens(
        content: &str,
        tokens: &mut Vec<AbsoluteToken>,
        loc: &vize_atelier_sfc::BlockLocation,
        line_index: &LineIndex<'_>,
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
                        let (line, col) = line_index.line_col(abs_pos);
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
                let (line, col) = line_index.line_col(abs_pos);
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
                        let (line, col) = line_index.line_col(abs_pos);
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
                let (line, col) = line_index.line_col(abs_pos);
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

        // Collect art-specific attribute tokens in the slice in a single pass
        // (see `collect_named_attribute_tokens`). Inline blocks never treat
        // `default="..."` as a valued attribute, so use `INLINE_ART_ATTR_NAMES`.
        collect_named_attribute_tokens(
            slice,
            range_start,
            INLINE_ART_ATTR_NAMES,
            line_index,
            tokens,
        );

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
                        let (line, col) = line_index.line_col(abs_pos);
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

        let allocator = vize_carton::Bump::new();
        let Ok(art_desc) =
            vize_musea::parse_art(&allocator, slice, vize_musea::ArtParseOptions::default())
        else {
            return;
        };

        for variant in art_desc.variants.iter() {
            if variant.template.trim().is_empty() {
                continue;
            }

            let slice_ptr = slice.as_ptr() as usize;
            let template_ptr = variant.template.as_ptr() as usize;
            let Some(relative_start) = template_ptr.checked_sub(slice_ptr) else {
                continue;
            };
            if relative_start > slice.len() {
                continue;
            }

            let absolute_start = range_start + relative_start;
            let (base_line, base_col) = line_index.line_col(absolute_start);
            let mut local_tokens = Vec::new();
            template::collect_template_tokens(variant.template, 0, &mut local_tokens);

            for mut token in local_tokens {
                if token.line == 0 {
                    token.start = token.start.saturating_add(base_col);
                }
                token.line = token.line.saturating_add(base_line);
                tokens.push(token);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LineIndex, SemanticTokensService, TokenModifier, TokenType, encoding::offset_to_line_col,
        expressions, template,
    };
    use tower_lsp::lsp_types::{
        Position, Range, SemanticToken, SemanticTokensRangeResult, SemanticTokensResult,
    };

    #[derive(Debug)]
    struct DecodedToken {
        line: u32,
        start: u32,
        length: u32,
        token_type: u32,
    }

    fn decode_tokens(tokens: &[SemanticToken]) -> Vec<DecodedToken> {
        let mut decoded = Vec::with_capacity(tokens.len());
        let mut line = 0;
        let mut start = 0;

        for token in tokens {
            line += token.delta_line;
            if token.delta_line == 0 {
                start += token.delta_start;
            } else {
                start = token.delta_start;
            }

            decoded.push(DecodedToken {
                line,
                start,
                length: token.length,
                token_type: token.token_type,
            });
        }

        decoded
    }

    fn has_token_text(
        template_str: &str,
        tokens: &[super::types::AbsoluteToken],
        token_type: TokenType,
        text: &str,
    ) -> bool {
        let Some(start) = template_str.find(text) else {
            return false;
        };

        tokens.iter().any(|token| {
            token.line == 0
                && token.start == start as u32
                && token.length == text.len() as u32
                && token.token_type == token_type as u32
        })
    }

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
        assert_eq!(offset_to_line_col(source, 0), (0, 0));
        assert_eq!(offset_to_line_col(source, 4), (1, 0));
        assert_eq!(offset_to_line_col(source, 8), (2, 0));
    }

    #[test]
    fn test_offset_to_line_col_counts_utf16_code_units() {
        let source = "const icon = \"😀\"; missing";
        let offset = source.find("missing").unwrap();

        assert_eq!(offset_to_line_col(source, offset), (0, 19));
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

        let uri = tower_lsp::lsp_types::Url::parse("file:///test.art.vue").unwrap();
        let result = SemanticTokensService::get_tokens(content, &uri);
        assert!(result.is_some());

        if let Some(SemanticTokensResult::Tokens(tokens)) = result {
            assert!(!tokens.data.is_empty());
        }
    }

    #[test]
    fn test_art_block_tokens() {
        let content = "<art title=\"Test\">\n</art>";
        let mut tokens = Vec::new();
        let line_index = LineIndex::new(content);
        SemanticTokensService::collect_art_block_tokens(content, &mut tokens, &line_index);

        // Should find <art and </art>
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].length, 4); // "<art"
        assert_eq!(tokens[1].length, 6); // "</art>"
    }

    #[test]
    fn test_variant_block_tokens() {
        let content = "<variant name=\"Primary\">\n</variant>";
        let mut tokens = Vec::new();
        let line_index = LineIndex::new(content);
        SemanticTokensService::collect_variant_block_tokens(content, &mut tokens, &line_index);

        // Should find <variant and </variant>
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].length, 8); // "<variant"
        assert_eq!(tokens[1].length, 10); // "</variant>"
    }

    #[test]
    fn test_art_attribute_tokens() {
        let content = r#"<art title="Button" component="./Button.vue">"#;
        let mut tokens = Vec::new();
        let line_index = LineIndex::new(content);
        SemanticTokensService::collect_art_attribute_tokens(content, &mut tokens, &line_index);

        // Should find title, "Button", component, "./Button.vue"
        assert!(tokens.len() >= 4);
    }

    #[test]
    fn test_art_variant_template_tokens() {
        let content = r#"<art title="Button" component="./Button.vue">
  <variant name="Primary" default>
    <Button :label="label" @click="handleClick">{{ label }}</Button>
  </variant>
</art>"#;
        let mut tokens = Vec::new();
        let line_index = LineIndex::new(content);
        SemanticTokensService::collect_art_variant_template_tokens(
            content,
            &mut tokens,
            &line_index,
        );

        assert!(
            tokens
                .iter()
                .any(|token| token.line == 2 && token.token_type == TokenType::Property as u32),
            "{tokens:#?}"
        );
        assert!(
            tokens
                .iter()
                .any(|token| token.line == 2 && token.token_type == TokenType::Event as u32),
            "{tokens:#?}"
        );
        assert!(
            tokens
                .iter()
                .any(|token| token.line == 2 && token.token_type == TokenType::Variable as u32),
            "{tokens:#?}"
        );
    }

    #[test]
    fn test_art_script_tokens() {
        let content = r#"<script setup>
import Button from './Button.vue'
</script>"#;
        let mut tokens = Vec::new();
        let line_index = LineIndex::new(content);
        SemanticTokensService::collect_art_script_tokens(content, &mut tokens, &line_index);

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
    fn test_interpolation_string_token_uses_utf16_length() {
        let template_str = "  {{ \"😀\" }}";
        let mut tokens = Vec::new();
        template::collect_interpolation_tokens(template_str, 1, &mut tokens);

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::String as u32);
        assert_eq!(tokens[0].length, 4);
    }

    #[test]
    fn test_template_semantic_tokens_ignore_plain_text_lookalikes() {
        let template_str = "<div>email dev@example.com and text v-if :class @click</div>";
        let mut tokens = Vec::new();
        template::collect_template_tokens(template_str, 0, &mut tokens);

        assert!(tokens.is_empty(), "{tokens:#?}");
    }

    #[test]
    fn test_template_semantic_tokens_ignore_static_attribute_text() {
        let template_str = r#"<div title="plain v-if @click :class"></div>"#;
        let mut tokens = Vec::new();
        template::collect_template_tokens(template_str, 0, &mut tokens);

        assert!(tokens.is_empty(), "{tokens:#?}");
    }

    #[test]
    fn test_directive_expression_does_not_steal_next_attribute_value() {
        let template_str = r#"<div v-else title="message"></div>"#;
        let mut tokens = Vec::new();
        template::collect_directive_expression_tokens(template_str, 0, &mut tokens);

        assert!(tokens.is_empty(), "{tokens:#?}");
    }

    #[test]
    fn test_template_semantic_tokens_still_collect_attribute_tokens() {
        let template_str = r#"<div v-if="ready" @click="save" :class="classes"></div>"#;
        let mut tokens = Vec::new();
        template::collect_template_tokens(template_str, 0, &mut tokens);

        assert!(
            tokens
                .iter()
                .any(|token| token.token_type == TokenType::Keyword as u32),
            "{tokens:#?}"
        );
        assert!(
            tokens
                .iter()
                .any(|token| token.token_type == TokenType::Event as u32),
            "{tokens:#?}"
        );
        assert!(
            tokens
                .iter()
                .any(|token| token.token_type == TokenType::Property as u32),
            "{tokens:#?}"
        );
        assert!(
            tokens
                .iter()
                .any(|token| token.token_type == TokenType::Variable as u32),
            "{tokens:#?}"
        );
    }

    #[test]
    fn test_template_semantic_tokens_collect_dynamic_shorthand_args() {
        let template_str = r#"<button @[eventName].stop="run" :[propName].camel="value"></button>"#;
        let mut tokens = Vec::new();
        template::collect_template_tokens(template_str, 0, &mut tokens);

        assert!(
            has_token_text(template_str, &tokens, TokenType::Event, "@[eventName].stop"),
            "{tokens:#?}"
        );
        assert!(
            has_token_text(template_str, &tokens, TokenType::Property, ":[propName]"),
            "{tokens:#?}"
        );
        for name in ["eventName", "propName", "run", "value"] {
            assert!(
                has_token_text(template_str, &tokens, TokenType::Variable, name),
                "missing {name}: {tokens:#?}"
            );
        }
    }

    #[test]
    fn test_template_semantic_tokens_collect_unquoted_directive_values() {
        let template_str = r#"<div v-if=ready @click=save :class=classes></div>"#;
        let mut tokens = Vec::new();
        template::collect_template_tokens(template_str, 0, &mut tokens);

        for name in ["ready", "save", "classes"] {
            assert!(
                has_token_text(template_str, &tokens, TokenType::Variable, name),
                "missing {name}: {tokens:#?}"
            );
        }
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
    fn test_full_sfc_semantic_tokens_use_lsp_coordinates() {
        let content = r#"<template>
  <div>{{ count }}</div>
</template>

<script setup>
const icon = "😀"
const count = ref(icon)
</script>
"#;

        let uri = tower_lsp::lsp_types::Url::parse("file:///test.vue").unwrap();
        let result = SemanticTokensService::get_tokens(content, &uri);
        let Some(SemanticTokensResult::Tokens(tokens)) = result else {
            panic!("expected semantic tokens");
        };
        let decoded = decode_tokens(&tokens.data);

        assert!(
            decoded.iter().any(|token| {
                token.line == 1
                    && token.start == 10
                    && token.length == "count".len() as u32
                    && token.token_type == TokenType::Variable as u32
            }),
            "{decoded:#?}"
        );
        assert!(
            decoded.iter().any(|token| {
                token.line == 6
                    && token.start == 14
                    && token.length == "ref".len() as u32
                    && token.token_type == TokenType::Function as u32
            }),
            "{decoded:#?}"
        );
    }

    #[test]
    fn test_range_semantic_tokens_return_only_requested_lines() {
        let content = r#"<template>
  <div>{{ count }}</div>
</template>

<script setup>
const count = ref(0)
</script>
"#;

        let uri = tower_lsp::lsp_types::Url::parse("file:///test.vue").unwrap();
        let result = SemanticTokensService::get_tokens_range(
            content,
            &uri,
            Range {
                start: Position {
                    line: 5,
                    character: 0,
                },
                end: Position {
                    line: 6,
                    character: 0,
                },
            },
        );
        let Some(SemanticTokensRangeResult::Tokens(tokens)) = result else {
            panic!("expected range semantic tokens");
        };
        let decoded = decode_tokens(&tokens.data);

        assert!(!decoded.is_empty());
        assert!(decoded.iter().all(|token| token.line == 5), "{decoded:#?}");
        assert!(
            decoded
                .iter()
                .any(|token| token.start == 14 && token.token_type == TokenType::Function as u32),
            "{decoded:#?}"
        );
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
