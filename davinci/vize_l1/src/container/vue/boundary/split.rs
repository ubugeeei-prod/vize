//! Complete compatibility SFC block walk over the L1-owned boundary scanners.
//!
//! Descriptor policy stays with each compatibility consumer. This module owns
//! the source walk, block boundaries, attributes, and byte line/column facts so
//! Croquis and Atelier no longer carry a second SFC splitter.

use alloc::borrow::Cow;
use memchr::{memchr, memmem::Finder};
use vize_l0::String;

use super::{BlockAttrs, newlines, parse_block_fast};

/// Original-source location of one compatibility block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompatBlockLocation {
    pub start: usize,
    pub end: usize,
    pub tag_start: usize,
    pub tag_end: usize,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

/// One complete original block emitted by [`split_blocks`].
#[derive(Debug)]
pub struct CompatBlock<'a> {
    pub tag_name: &'a [u8],
    pub attrs: BlockAttrs<'a>,
    pub content: Cow<'a, str>,
    pub location: CompatBlockLocation,
}

/// A malformed block found by the shared compatibility walk.
#[derive(Debug)]
pub struct CompatSplitError {
    pub code: &'static str,
    pub message: String,
    pub location: CompatBlockLocation,
}

#[inline]
fn advance_line_column(bytes: &[u8], line: &mut usize, column: &mut usize) {
    let (count, last_newline) = newlines(bytes);
    *line += count;
    if let Some(offset) = last_newline {
        *column = bytes.len() - offset;
    } else {
        *column += bytes.len();
    }
}

/// Visit every top-level SFC block in original order.
///
/// Returning `false` from `visit` stops after that block. This lets descriptor
/// adapters preserve their first duplicate-block error without rescanning or
/// allocating an intermediate block vector.
pub fn split_blocks<'a>(
    source: &'a str,
    mut visit: impl FnMut(CompatBlock<'a>) -> bool,
) -> Result<(), CompatSplitError> {
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut pos = 0;
    let mut line = 1;
    let mut column = 1;
    let comment_end_finder = Finder::new(b"-->");

    while pos < len {
        while pos < len
            && let Some(&byte) = bytes.get(pos)
        {
            if matches!(byte, b' ' | b'\t' | b'\r') {
                pos += 1;
                column += 1;
            } else if byte == b'\n' {
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

        if bytes.get(pos) != Some(&b'<') {
            let Some(next_lt) = memchr(b'<', bytes.get(pos..).unwrap_or_default()) else {
                break;
            };
            advance_line_column(
                bytes.get(pos..pos + next_lt).unwrap_or_default(),
                &mut line,
                &mut column,
            );
            pos += next_lt;
        }
        if pos >= len {
            break;
        }

        if bytes.get(pos..).unwrap_or_default().starts_with(b"<!--") {
            let comment_body = bytes.get(pos + 4..).unwrap_or_default();
            let end = comment_end_finder
                .find(comment_body)
                .map_or(len, |offset| pos + 4 + offset + 3);
            advance_line_column(
                bytes.get(pos..end).unwrap_or_default(),
                &mut line,
                &mut column,
            );
            pos = end;
            continue;
        }

        match parse_block_fast(bytes, source, pos, line, column) {
            Ok(Some((
                tag_name,
                attrs,
                content,
                content_start,
                content_end,
                end_pos,
                content_end_line,
                content_end_column,
            ))) => {
                let mut content_start_line = line;
                let mut content_start_column = column;
                advance_line_column(
                    bytes.get(pos..content_start).unwrap_or_default(),
                    &mut content_start_line,
                    &mut content_start_column,
                );
                let keep_going = visit(CompatBlock {
                    tag_name,
                    attrs,
                    content,
                    location: CompatBlockLocation {
                        start: content_start,
                        end: content_end,
                        tag_start: pos,
                        tag_end: end_pos,
                        start_line: content_start_line,
                        start_column: content_start_column,
                        end_line: content_end_line,
                        end_column: content_end_column,
                    },
                });
                if !keep_going {
                    return Ok(());
                }
                pos = end_pos;
                line = content_end_line;
                column = content_end_column;
                advance_line_column(
                    bytes.get(content_end..end_pos).unwrap_or_default(),
                    &mut line,
                    &mut column,
                );
            }
            Ok(None) => {
                pos += 1;
                column += 1;
            }
            Err((code, message)) => {
                let mut end_line = line;
                let mut end_column = column;
                advance_line_column(
                    bytes.get(pos..len).unwrap_or_default(),
                    &mut end_line,
                    &mut end_column,
                );
                return Err(CompatSplitError {
                    code,
                    message,
                    location: CompatBlockLocation {
                        start: pos,
                        end: len,
                        tag_start: pos,
                        tag_end: len,
                        start_line: line,
                        start_column: column,
                        end_line,
                        end_column,
                    },
                });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_walk_retains_blocks_attributes_locations_and_recovery() {
        let source = "<!--x-->\n<template lang=html>雪</template>\n<script setup>let x=1</script>";
        let mut seen = 0;
        split_blocks(source, |block| {
            match seen {
                0 => {
                    assert_eq!(block.tag_name, b"template");
                    assert_eq!(block.content, "雪");
                    assert_eq!(block.attrs.get("lang").map(Cow::as_ref), Some("html"));
                    assert_eq!(block.location.start_line, 2);
                    assert_eq!(block.location.end_column, 24);
                }
                1 => {
                    assert_eq!(block.tag_name, b"script");
                    assert!(block.attrs.contains_key("setup"));
                    assert_eq!(block.location.start_line, 3);
                }
                _ => panic!("unexpected extra block"),
            }
            seen += 1;
            true
        })
        .unwrap();
        assert_eq!(seen, 2);

        let error = split_blocks("\n<style", |_| true).unwrap_err();
        assert_eq!(error.code, "MALFORMED_BLOCK");
        assert_eq!(error.location.start_line, 2);
        assert_eq!(error.location.start_column, 1);
    }

    #[test]
    fn line_terminators_are_attribute_whitespace() {
        // Vue's tokenizer and the native open-tag scanner skip every space
        // character around `=` and end unquoted values at any of them.
        let source = "<script setup lang\n=\r\n\"ts\">let x = 1</script>\n<template lang=html\r data-x\t=\ty><p/></template>";
        let mut attrs = alloc::vec::Vec::new();
        split_blocks(source, |block| {
            attrs.push(block.attrs);
            true
        })
        .unwrap();
        let value = |index: usize, name: &str| {
            attrs
                .get(index)
                .and_then(|attrs: &super::BlockAttrs<'_>| attrs.get(name))
                .map(Cow::as_ref)
        };
        assert_eq!(attrs.first().map(|attrs| attrs.len()), Some(2));
        assert_eq!(value(0, "setup"), Some(""));
        assert_eq!(value(0, "lang"), Some("ts"));
        assert_eq!(attrs.get(1).map(|attrs| attrs.len()), Some(2));
        assert_eq!(value(1, "lang"), Some("html"));
        assert_eq!(value(1, "data-x"), Some("y"));
    }

    #[test]
    fn visitor_can_stop_without_scanning_a_later_malformed_block() {
        let mut seen = 0;
        split_blocks("<template>x</template><style", |_| {
            seen += 1;
            false
        })
        .unwrap();
        assert_eq!(seen, 1);
    }
}
