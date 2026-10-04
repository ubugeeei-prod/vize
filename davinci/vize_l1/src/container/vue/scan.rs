//! Byte-oriented SFC tag and interpolation scanning.

use vize_l0::{Allocator, Span, Vec};

use super::super::BlockAttr;

mod interpolation;
use interpolation::skip_interpolation;

pub(super) struct Open<'a> {
    pub(super) name: &'a str,
    pub(super) end: usize,
    pub(super) attrs: Vec<'a, BlockAttr<'a>>,
    pub(super) self_closing: bool,
}

pub(super) fn read_open<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    at: usize,
) -> Result<Option<Open<'a>>, ()> {
    let bytes = source.as_bytes();
    if bytes.get(at) != Some(&b'<') || !bytes.get(at + 1).is_some_and(u8::is_ascii_alphabetic) {
        return Ok(None);
    }
    let mut pos = at + 1;
    while bytes.get(pos).is_some_and(|byte| is_name(*byte)) {
        pos += 1;
    }
    let name = source.get(at + 1..pos).ok_or(())?;
    let mut attrs = Vec::new_in(&allocator);
    loop {
        while bytes
            .get(pos)
            .is_some_and(|byte| byte.is_ascii_whitespace())
        {
            pos += 1;
        }
        match bytes.get(pos) {
            Some(b'>') => {
                return Ok(Some(Open {
                    name,
                    end: pos + 1,
                    attrs,
                    self_closing: false,
                }));
            }
            Some(b'/') if bytes.get(pos + 1) == Some(&b'>') => {
                return Ok(Some(Open {
                    name,
                    end: pos + 2,
                    attrs,
                    self_closing: true,
                }));
            }
            None => return Err(()),
            _ => {}
        }
        let start = pos;
        while bytes
            .get(pos)
            .is_some_and(|byte| !byte.is_ascii_whitespace() && !matches!(byte, b'=' | b'>' | b'/'))
        {
            pos += 1;
        }
        if pos == start {
            return Err(());
        }
        let name = source.get(start..pos).ok_or(())?;
        while bytes
            .get(pos)
            .is_some_and(|byte| byte.is_ascii_whitespace())
        {
            pos += 1;
        }
        let value = if bytes.get(pos) == Some(&b'=') {
            pos += 1;
            while bytes
                .get(pos)
                .is_some_and(|byte| byte.is_ascii_whitespace())
            {
                pos += 1;
            }
            if let Some(quote @ (b'\'' | b'"')) = bytes.get(pos).copied() {
                pos += 1;
                let value_start = pos;
                while bytes.get(pos).is_some_and(|byte| *byte != quote) {
                    pos += 1;
                }
                if bytes.get(pos).is_none() {
                    return Err(());
                }
                let value = source.get(value_start..pos).ok_or(())?;
                pos += 1;
                Some(value)
            } else {
                let value_start = pos;
                while bytes
                    .get(pos)
                    .is_some_and(|byte| !byte.is_ascii_whitespace() && !matches!(byte, b'>' | b'/'))
                {
                    pos += 1;
                }
                Some(source.get(value_start..pos).ok_or(())?)
            }
        } else {
            None
        };
        attrs.push(BlockAttr {
            name,
            value,
            span: Span::new(start as u32, pos as u32),
        });
    }
}

pub(super) fn find_template_close<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    from: usize,
) -> (usize, Option<usize>, bool) {
    let bytes = source.as_bytes();
    let (mut pos, mut depth) = (from, 1usize);
    let mut uncertain = false;
    while pos < bytes.len() {
        if bytes
            .get(pos..)
            .is_some_and(|rest| rest.starts_with(b"<!--"))
        {
            pos = find_bytes(bytes, pos + 4, b"-->").map_or(bytes.len(), |end| end + 3);
            continue;
        }
        if bytes.get(pos..).is_some_and(|rest| rest.starts_with(b"{{")) {
            if let Some(end) = skip_interpolation(bytes, pos + 2) {
                pos = end;
            } else {
                uncertain = true;
                // An unclosed interpolation owns the remaining input. Do not
                // re-scan its suffix or guess a root closing tag inside it.
                pos = bytes.len();
            }
            continue;
        }
        if bytes.get(pos) != Some(&b'<') {
            pos += 1;
            continue;
        }
        if let Some(end) = close_at(bytes, pos, "template") {
            depth -= 1;
            if depth == 0 {
                return (pos, Some(end), uncertain);
            }
            pos = end;
            continue;
        }
        if let Ok(Some(open)) = read_open(allocator, source, pos) {
            if open.name.eq_ignore_ascii_case("template") && !open.self_closing {
                depth += 1;
            }
            if ["script", "style", "textarea", "title"]
                .iter()
                .any(|name| open.name.eq_ignore_ascii_case(name))
                && !open.self_closing
                && let Some((_, end)) = find_close(bytes, open.end, open.name)
            {
                pos = end;
                continue;
            }
            pos = open.end;
            continue;
        }
        pos += 1;
    }
    (bytes.len(), None, uncertain)
}

pub(super) fn find_close(bytes: &[u8], from: usize, name: &str) -> Option<(usize, usize)> {
    let mut pos = from;
    while pos < bytes.len() {
        if bytes.get(pos) == Some(&b'<')
            && let Some(end) = close_at(bytes, pos, name)
        {
            return Some((pos, end));
        }
        pos += 1;
    }
    None
}

fn close_at(bytes: &[u8], at: usize, name: &str) -> Option<usize> {
    if bytes.get(at..at + 2) != Some(b"</") {
        return None;
    }
    let end_name = at + 2 + name.len();
    if !bytes
        .get(at + 2..end_name)
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(name.as_bytes()))
    {
        return None;
    }
    let mut pos = end_name;
    while bytes
        .get(pos)
        .is_some_and(|byte| byte.is_ascii_whitespace())
    {
        pos += 1;
    }
    (bytes.get(pos) == Some(&b'>')).then_some(pos + 1)
}

pub(super) fn find_bytes(bytes: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    bytes
        .get(from..)?
        .windows(needle.len())
        .position(|part| part == needle)
        .map(|offset| from + offset)
}

fn is_name(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')
}
