//! Art-file token collection for semantic highlighting.
//!
//! Handles `.art.vue` files and inline `<art>` custom blocks of regular
//! `.vue` files: `<art>`/`<variant>` tags, art-specific attributes, embedded
//! `<script>` blocks, and the Vue templates inside each `<variant>` body.

use super::{
    SemanticTokensService,
    encoding::{LineIndex, utf16_len},
    template,
    types::{AbsoluteToken, TokenModifier, TokenType},
};

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

impl SemanticTokensService {
    pub(super) fn collect_art_tokens(content: &str) -> Vec<AbsoluteToken> {
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
    pub(super) fn collect_art_block_tokens(
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
    pub(super) fn collect_variant_block_tokens(
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
    pub(super) fn collect_art_attribute_tokens(
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
    pub(super) fn collect_art_variant_template_tokens(
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
    pub(super) fn collect_art_script_tokens(
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
    pub(super) fn collect_inline_art_tokens(
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
