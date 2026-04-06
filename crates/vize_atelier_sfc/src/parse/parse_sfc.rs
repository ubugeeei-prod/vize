use crate::types::{
    BlockLocation, SfcCustomBlock, SfcDescriptor, SfcError, SfcParseOptions, SfcScriptBlock,
    SfcStyleBlock, SfcTemplateBlock,
};
use memchr::{memchr, memmem::Finder};
use std::borrow::Cow;

use super::block::{parse_block_fast, tag_name_eq};

// Tag name bytes for fast comparison
const TAG_TEMPLATE: &[u8] = b"template";
const TAG_SCRIPT: &[u8] = b"script";
const TAG_STYLE: &[u8] = b"style";

/// Parse a Vue SFC into a descriptor with zero-copy strings
pub fn parse_sfc<'a>(
    source: &'a str,
    options: SfcParseOptions,
) -> Result<SfcDescriptor<'a>, SfcError> {
    let mut descriptor = SfcDescriptor {
        filename: Cow::Owned(options.filename.into()),
        source: Cow::Borrowed(source),
        ..Default::default()
    };

    let bytes = source.as_bytes();
    let len = bytes.len();

    let mut pos = 0;
    let mut line = 1;
    let mut column = 1;
    let comment_end_finder = Finder::new(b"-->");

    while pos < len {
        // Skip whitespace using byte comparison
        while pos < len {
            let c = bytes[pos];
            if c == b' ' || c == b'\t' || c == b'\r' {
                pos += 1;
                column += 1;
            } else if c == b'\n' {
                pos += 1;
                line += 1;
                column = 1;
            } else {
                break;
            }
        }

        if pos >= len {
            break;
        }

        // Use memchr to find next '<' quickly
        if bytes[pos] != b'<' {
            if let Some(next_lt) = memchr(b'<', &bytes[pos..]) {
                // Update line/column for skipped content
                for &b in &bytes[pos..pos + next_lt] {
                    if b == b'\n' {
                        line += 1;
                        column = 1;
                    } else {
                        column += 1;
                    }
                }
                pos += next_lt;
            } else {
                break;
            }
        }

        if pos >= len {
            break;
        }

        // Skip HTML comments <!-- ... --> before block parsing.
        if bytes[pos..].starts_with(b"<!--") {
            let comment_body = &bytes[pos + 4..];
            let end = comment_end_finder
                .find(comment_body)
                .map(|off| pos + 4 + off + 3) // position after '-->'
                .unwrap_or(len); // unclosed comment: skip to EOF

            // Update line/column for the skipped comment
            for &b in &bytes[pos..end] {
                if b == b'\n' {
                    line += 1;
                    column = 1;
                } else {
                    column += 1;
                }
            }
            pos = end;
            continue;
        }

        // Parse block starting at '<'
        if let Some(block_result) = parse_block_fast(bytes, source, pos, line) {
            let (tag_name, attrs, content, content_start, content_end, end_pos, end_line, end_col) =
                block_result;

            let loc = BlockLocation {
                start: content_start,
                end: content_end,
                tag_start: pos,
                tag_end: end_pos,
                start_line: line,
                start_column: column,
                end_line,
                end_column: end_col,
            };

            // Match tag name using byte comparison
            if tag_name_eq(tag_name, TAG_TEMPLATE) {
                if descriptor.template.is_some() {
                    return Err(SfcError {
                        message: "SFC can only contain one <template> block".into(),
                        code: Some("DUPLICATE_TEMPLATE".into()),
                        loc: Some(loc.clone()),
                    });
                }
                descriptor.template = Some(SfcTemplateBlock {
                    content,
                    loc,
                    lang: attrs.get("lang").cloned(),
                    src: attrs.get("src").cloned(),
                    attrs,
                });
            } else if tag_name_eq(tag_name, TAG_SCRIPT) {
                let is_setup = attrs.contains_key("setup");
                let script_block = SfcScriptBlock {
                    content,
                    loc,
                    lang: attrs.get("lang").cloned(),
                    src: attrs.get("src").cloned(),
                    setup: is_setup,
                    attrs,
                    bindings: None,
                };

                if is_setup {
                    if descriptor.script_setup.is_some() {
                        return Err(SfcError {
                            message: "SFC can only contain one <script setup> block".into(),
                            code: Some("DUPLICATE_SCRIPT_SETUP".into()),
                            loc: Some(script_block.loc),
                        });
                    }
                    descriptor.script_setup = Some(script_block);
                } else {
                    if descriptor.script.is_some() {
                        return Err(SfcError {
                            message: "SFC can only contain one <script> block".into(),
                            code: Some("DUPLICATE_SCRIPT".into()),
                            loc: Some(script_block.loc),
                        });
                    }
                    descriptor.script = Some(script_block);
                }
            } else if tag_name_eq(tag_name, TAG_STYLE) {
                let scoped = attrs.contains_key("scoped");
                let module = if attrs.contains_key("module") {
                    Some(
                        attrs
                            .get("module")
                            .filter(|v| !v.is_empty())
                            .cloned()
                            .unwrap_or_else(|| Cow::Borrowed("$style")),
                    )
                } else {
                    None
                };

                descriptor.styles.push(SfcStyleBlock {
                    content,
                    loc,
                    lang: attrs.get("lang").cloned(),
                    src: attrs.get("src").cloned(),
                    scoped,
                    module,
                    attrs,
                });
            } else {
                // Custom block - use borrowed tag name
                let tag_str = unsafe { std::str::from_utf8_unchecked(tag_name) };
                descriptor.custom_blocks.push(SfcCustomBlock {
                    block_type: Cow::Borrowed(tag_str),
                    content,
                    loc,
                    attrs,
                });
            }

            pos = end_pos;
            line = end_line;
            column = end_col;
        } else {
            pos += 1;
            column += 1;
        }
    }

    Ok(descriptor)
}
