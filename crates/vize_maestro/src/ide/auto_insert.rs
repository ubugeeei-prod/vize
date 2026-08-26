//! Volar-compatible automatic insertion snippets.
//!
//! The wire method is private Volar protocol, but the snippets deliberately
//! match `@vue/language-server` 3.3.8 so one VS Code client can drive either
//! server. Markup insertions are syntax-only. `.value` is different: it is
//! returned only when Corsa's TypeScript quick info identifies the symbol as a
//! Vue ref; authored source is never scanned to guess its type.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

#[cfg(feature = "native")]
mod corsa;

use super::IdeContext;
use crate::virtual_code::BlockType;

pub struct AutoInsertService;

impl AutoInsertService {
    pub async fn snippet(
        ctx: &IdeContext<'_>,
        selection_offset: usize,
        range_offset: usize,
        change_text: &str,
    ) -> Option<String> {
        match change_text {
            "{}" => bracket_spacing(&ctx.content, selection_offset, range_offset),
            "=" => auto_quote(ctx, selection_offset),
            ">" => auto_close_tag(ctx, selection_offset),
            "/" => auto_complete_close_tag(ctx, selection_offset),
            _ if dot_value_candidate(ctx, selection_offset, range_offset, change_text) => {
                #[cfg(feature = "native")]
                {
                    let bridge = ctx.state.get_corsa_bridge().await;
                    corsa::dot_value(ctx, selection_offset, bridge).await
                }
                #[cfg(not(feature = "native"))]
                {
                    None
                }
            }
            _ => None,
        }
    }
}

fn bracket_spacing(content: &str, selection: usize, change_start: usize) -> Option<String> {
    (selection == change_start.checked_add(1)?
        && change_start > 0
        && content.get(change_start - 1..change_start.checked_add(3)?)? == "{{}}")
        .then(|| " $0 ".to_string())
}

fn auto_quote(ctx: &IdeContext<'_>, selection: usize) -> Option<String> {
    let region = markup_region(ctx, selection)?;
    if selection == 0
        || ctx.content.as_bytes().get(selection - 1) != Some(&b'=')
        || matches!(ctx.content.as_bytes().get(selection), Some(b'\'' | b'"'))
        || !inside_open_start_tag(&ctx.content, region, selection - 1)
    {
        return None;
    }
    Some("\"$1\"".to_string())
}

fn auto_close_tag(ctx: &IdeContext<'_>, selection: usize) -> Option<String> {
    let region = markup_region(ctx, selection)?;
    let end = selection.checked_sub(1)?;
    if ctx.content.as_bytes().get(end) != Some(&b'>') {
        return None;
    }
    let (name, tag_start) = start_tag_before(&ctx.content, region, end)?;
    if ctx.content[tag_start..end].trim_end().ends_with('/')
        || vize_s0::is_void_tag(name)
        || already_has_close_tag(&ctx.content, selection, name)
    {
        return None;
    }
    Some(format!("$0</{name}>"))
}

fn auto_complete_close_tag(ctx: &IdeContext<'_>, selection: usize) -> Option<String> {
    let region = markup_region(ctx, selection)?;
    let slash = selection.checked_sub(1)?;
    if slash == 0 || ctx.content.as_bytes().get(slash - 1..selection) != Some(&b"</"[..]) {
        return None;
    }
    nearest_unclosed_tag(&ctx.content, region, slash - 1).map(|name| format!("{name}>"))
}

fn markup_region(ctx: &IdeContext<'_>, offset: usize) -> Option<(usize, usize)> {
    super::sfc_region::resolve(&ctx.content, ctx.uri.path(), offset).markup
}

fn inside_open_start_tag(content: &str, region: (usize, usize), offset: usize) -> bool {
    let bytes = content.as_bytes();
    let mut cursor = region.0;
    let mut tag_start = None;
    let mut quote = None;
    while cursor <= offset && cursor < region.1 {
        let byte = bytes[cursor];
        match quote {
            Some(open) if byte == open => quote = None,
            Some(_) => {}
            None if byte == b'\'' || byte == b'"' => quote = Some(byte),
            None if byte == b'<' => tag_start = Some(cursor),
            None if byte == b'>' => tag_start = None,
            None => {}
        }
        cursor += 1;
    }
    tag_start.is_some_and(|start| !content[start..].starts_with("</")) && quote.is_none()
}

fn start_tag_before(content: &str, region: (usize, usize), end: usize) -> Option<(&str, usize)> {
    let bytes = content.as_bytes();
    let mut cursor = region.0;
    let mut start = None;
    let mut quote = None;
    while cursor < end && cursor < region.1 {
        let byte = bytes[cursor];
        match quote {
            Some(open) if byte == open => quote = None,
            Some(_) => {}
            None if byte == b'\'' || byte == b'"' => quote = Some(byte),
            None if byte == b'<' => start = Some(cursor),
            None if byte == b'>' => start = None,
            None => {}
        }
        cursor += 1;
    }
    let start = start?;
    let tail = content.get(start + 1..end)?;
    if tail.starts_with(['/', '!', '?']) {
        return None;
    }
    let name_end = tail
        .bytes()
        .position(|byte| !is_tag_name_byte(byte))
        .unwrap_or(tail.len());
    (name_end > 0).then(|| (&tail[..name_end], start))
}

fn already_has_close_tag(content: &str, selection: usize, name: &str) -> bool {
    let Some(after) = content.get(selection..) else {
        return false;
    };
    let after = after.trim_start();
    after.starts_with("</")
        && after.get(2..).is_some_and(|tail| {
            tail.starts_with(name) && tail.as_bytes().get(name.len()) == Some(&b'>')
        })
}

fn nearest_unclosed_tag(content: &str, region: (usize, usize), before: usize) -> Option<&str> {
    let bytes = content.as_bytes();
    let mut stack = Vec::new();
    let mut cursor = region.0;
    let limit = before.min(region.1);
    while cursor < limit {
        if bytes[cursor] != b'<' {
            cursor += 1;
            continue;
        }
        if content[cursor..limit].starts_with("<!--") {
            cursor = content[cursor..limit]
                .find("-->")
                .map_or(limit, |relative| cursor + relative + 3);
            continue;
        }
        let closing = bytes.get(cursor + 1) == Some(&b'/');
        let name_start = cursor + if closing { 2 } else { 1 };
        let mut name_end = name_start;
        while name_end < limit && is_tag_name_byte(bytes[name_end]) {
            name_end += 1;
        }
        let name = content.get(name_start..name_end)?;
        let tag_end = find_tag_end(content, name_end, limit)?;
        if closing {
            if let Some(index) = stack.iter().rposition(|open| *open == name) {
                stack.truncate(index);
            }
        } else if !name.is_empty()
            && !vize_s0::is_void_tag(name)
            && !content[cursor..tag_end]
                .trim_end_matches('>')
                .trim_end()
                .ends_with('/')
        {
            stack.push(name);
        }
        cursor = tag_end;
    }
    stack.pop()
}

fn find_tag_end(content: &str, start: usize, limit: usize) -> Option<usize> {
    let bytes = content.as_bytes();
    let mut quote = None;
    for (relative, byte) in bytes[start..limit].iter().copied().enumerate() {
        match quote {
            Some(open) if byte == open => quote = None,
            Some(_) => {}
            None if byte == b'\'' || byte == b'"' => quote = Some(byte),
            None if byte == b'>' => return Some(start + relative + 1),
            None => {}
        }
    }
    None
}

fn dot_value_candidate(
    ctx: &IdeContext<'_>,
    selection: usize,
    range_offset: usize,
    change_text: &str,
) -> bool {
    if !matches!(
        ctx.block_type,
        Some(BlockType::Script | BlockType::ScriptSetup)
    ) || change_text.contains('\n')
        || change_text.is_empty()
        || !change_text
            .as_bytes()
            .last()
            .is_some_and(u8::is_ascii_alphanumeric)
        || selection <= range_offset
        || ctx
            .content
            .as_bytes()
            .get(selection)
            .is_some_and(|byte| is_identifier_byte(*byte))
    {
        return false;
    }
    let Some((start, end)) =
        super::token_span_at_offset(&ctx.content, selection, is_identifier_byte)
    else {
        return false;
    };
    if end != selection
        || ctx.content.as_bytes().get(start.wrapping_sub(1)) == Some(&b'.')
        || ctx
            .content
            .get(selection..)
            .is_some_and(|tail| tail.starts_with(".value"))
    {
        return false;
    }
    let line_start = ctx.content[..start].rfind('\n').map_or(0, |at| at + 1);
    let before = ctx.content[line_start..start].trim_start();
    !is_declaration_or_pass_through(before, ctx.content.get(end..).unwrap_or_default())
}

fn is_declaration_or_pass_through(before: &str, after: &str) -> bool {
    const PREFIXES: &[&str] = &[
        "const ",
        "let ",
        "var ",
        "function ",
        "class ",
        "interface ",
        "type ",
        "import ",
    ];
    if PREFIXES.iter().any(|prefix| before.starts_with(prefix))
        || after.trim_start().starts_with(':')
    {
        return true;
    }
    let compact = before.trim_end();
    ["watch(", "unref(", "triggerRef(", "isRef("]
        .iter()
        .any(|prefix| compact.ends_with(prefix))
        || compact.rsplit_once('(').is_some_and(|(head, _)| {
            head.split_whitespace()
                .last()
                .is_some_and(|name| name.starts_with("use"))
        })
}

#[inline]
fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$')
}

#[inline]
fn is_tag_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':')
}

#[cfg(test)]
mod tests;
