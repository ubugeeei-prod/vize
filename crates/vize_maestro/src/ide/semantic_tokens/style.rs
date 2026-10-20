//! Style token collection for semantic highlighting.
//!
//! Handles CSS `v-bind()` expressions in `<style>` blocks.

use std::ops::Range;

use super::{
    encoding::{offset_to_line_col, utf16_len},
    types::{AbsoluteToken, TokenType},
};

const V_BIND: &str = "v-bind(";

/// One CSS `v-bind(` call in a style block.
pub(crate) struct VBind {
    /// Byte offset of `v-bind`.
    pub(crate) keyword: usize,
    /// The bound expression, without quotes or trailing space; `None` when
    /// the call is never closed, which also ends the scan.
    pub(crate) argument: Option<Range<usize>>,
}

/// Every `v-bind(` call in style content. `v-bind(` inside comments and
/// strings is CSS text, and a quoted argument ends at its closing quote, so
/// `v-bind('fn(x)')` binds `fn(x)`.
pub(crate) fn v_binds(style: &str) -> Vec<VBind> {
    let bytes = style.as_bytes();
    let mut found = Vec::new();
    let mut pos = 0;
    while let Some(&byte) = bytes.get(pos) {
        let tail = bytes.get(pos..).unwrap_or_default();
        if tail.starts_with(b"/*") {
            pos = memchr::memmem::find(tail.get(2..).unwrap_or_default(), b"*/")
                .map_or(bytes.len(), |end| pos + 2 + end + 2);
            continue;
        }
        if matches!(byte, b'"' | b'\'') {
            pos = quoted_end(bytes, pos).map_or(bytes.len(), |end| end + 1);
            continue;
        }
        let at_word_start = pos
            .checked_sub(1)
            .and_then(|before| bytes.get(before))
            .is_none_or(|before| {
                !(before.is_ascii_alphanumeric() || matches!(before, b'-' | b'_'))
            });
        if !(at_word_start && tail.starts_with(V_BIND.as_bytes())) {
            pos += 1;
            continue;
        }
        let keyword = pos;
        let mut arg = pos + V_BIND.len();
        while bytes.get(arg).is_some_and(u8::is_ascii_whitespace) {
            arg += 1;
        }
        let closed = if bytes.get(arg).is_some_and(|b| matches!(b, b'"' | b'\'')) {
            quoted_end(bytes, arg).map(|close| ((arg + 1)..close, close + 1))
        } else {
            memchr::memchr(b')', bytes.get(arg..).unwrap_or_default()).map(|close| {
                let raw = style.get(arg..arg + close).unwrap_or_default();
                (arg..arg + raw.trim_end().len(), arg + close + 1)
            })
        };
        let Some((argument, end)) = closed else {
            found.push(VBind {
                keyword,
                argument: None,
            });
            break;
        };
        found.push(VBind {
            keyword,
            argument: Some(argument),
        });
        pos = end;
    }
    found
}

/// Collect tokens from style content.
pub(crate) fn collect_style_tokens(style: &str, base_line: u32, tokens: &mut Vec<AbsoluteToken>) {
    for v_bind in v_binds(style) {
        push(
            style,
            base_line,
            v_bind.keyword,
            6,
            TokenType::Function,
            tokens,
        );
        let Some(argument) = v_bind.argument else {
            continue;
        };
        if let Some(text) = style.get(argument.clone()).filter(|text| !text.is_empty()) {
            push(
                style,
                base_line,
                argument.start,
                utf16_len(text),
                TokenType::Variable,
                tokens,
            );
        }
    }
}

fn push(
    style: &str,
    base_line: u32,
    offset: usize,
    length: u32,
    token_type: TokenType,
    tokens: &mut Vec<AbsoluteToken>,
) {
    let (line, start) = offset_to_line_col(style, offset);
    tokens.push(AbsoluteToken {
        line: base_line + line,
        start,
        length,
        token_type: token_type as u32,
        modifiers: 0,
    });
}

fn quoted_end(bytes: &[u8], open: usize) -> Option<usize> {
    let quote = *bytes.get(open)?;
    let mut index = open + 1;
    while let Some(&byte) = bytes.get(index) {
        match byte {
            b'\\' => index += 2,
            _ if byte == quote => return Some(index),
            _ => index += 1,
        }
    }
    None
}
