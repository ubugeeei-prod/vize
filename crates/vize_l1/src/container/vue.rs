//! Lossless byte-span Vue SFC block splitter for opt-in stage capture.
//! Full product SFC descriptor migration remains #6837.

use vize_l0::{Allocator, Span, Vec};

use super::{Block, BlockAttr, Container, ContainerError, ContainerErrorCode, ContainerFormat};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Vue;

struct Open<'a> {
    name: &'a str,
    end: usize,
    attrs: Vec<'a, BlockAttr<'a>>,
    self_closing: bool,
}

impl ContainerFormat for Vue {
    fn split<'a>(&self, allocator: &'a Allocator, source: &'a str) -> Container<'a> {
        let mut result = Container {
            source,
            blocks: Vec::new_in(&allocator),
            errors: Vec::new_in(&allocator),
        };
        if source.len() > u32::MAX as usize {
            result.errors.push(ContainerError {
                code: ContainerErrorCode::SourceTooLarge,
                offset: 0,
            });
            return result;
        }
        let bytes = source.as_bytes();
        let (mut at, mut template, mut script, mut setup) = (0, false, false, false);
        while at < bytes.len() {
            if bytes[at] != b'<' {
                at += 1;
                continue;
            }
            if bytes[at..].starts_with(b"<!--") {
                at = find_bytes(bytes, at + 4, b"-->").map_or(bytes.len(), |end| end + 3);
                continue;
            }
            let open = match read_open(allocator, source, at) {
                Ok(Some(open)) => open,
                Ok(None) => {
                    at += 1;
                    continue;
                }
                Err(()) => {
                    result.errors.push(ContainerError {
                        code: ContainerErrorCode::UnterminatedOpenTag,
                        offset: at as u32,
                    });
                    break;
                }
            };
            let duplicate = if open.name.eq_ignore_ascii_case("template") {
                let old = template;
                template = true;
                old
            } else if open.name.eq_ignore_ascii_case("script") {
                if open.attrs.iter().any(|attr| attr.name == "setup") {
                    let old = setup;
                    setup = true;
                    old
                } else {
                    let old = script;
                    script = true;
                    old
                }
            } else {
                false
            };
            if duplicate {
                result.errors.push(ContainerError {
                    code: ContainerErrorCode::DuplicateBlock,
                    offset: at as u32,
                });
            }
            let (content_end, close_end, uncertain) = if open.self_closing {
                (open.end, Some(open.end), false)
            } else if open.name.eq_ignore_ascii_case("template") {
                find_template_close(allocator, source, open.end)
            } else {
                find_close(bytes, open.end, open.name)
                    .map_or((bytes.len(), None, false), |(start, end)| {
                        (start, Some(end), false)
                    })
            };
            if uncertain {
                result.errors.push(ContainerError {
                    code: ContainerErrorCode::UncertainInterpolation,
                    offset: at as u32,
                });
            }
            if close_end.is_none() {
                result.errors.push(ContainerError {
                    code: ContainerErrorCode::MissingCloseTag,
                    offset: at as u32,
                });
            }
            result.blocks.push(Block {
                name: open.name,
                open_tag: Span::new(at as u32, open.end as u32),
                attrs: open.attrs,
                content: Span::new(open.end as u32, content_end as u32),
                close_tag: close_end.map(|end| Span::new(content_end as u32, end as u32)),
            });
            at = close_end.unwrap_or(bytes.len());
        }
        result
    }
}

fn read_open<'a>(
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
    let name = &source[at + 1..pos];
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
        let name = &source[start..pos];
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
                let value = &source[value_start..pos];
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
                Some(&source[value_start..pos])
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

fn find_template_close<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    from: usize,
) -> (usize, Option<usize>, bool) {
    let bytes = source.as_bytes();
    let (mut pos, mut depth) = (from, 1usize);
    let mut uncertain = false;
    while pos < bytes.len() {
        if bytes[pos..].starts_with(b"<!--") {
            pos = find_bytes(bytes, pos + 4, b"-->").map_or(bytes.len(), |end| end + 3);
            continue;
        }
        if bytes[pos..].starts_with(b"{{") {
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
        if bytes[pos] != b'<' {
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

fn skip_interpolation(bytes: &[u8], from: usize) -> Option<usize> {
    let (mut pos, mut quote) = (from, None);
    while pos + 1 < bytes.len() {
        match (quote, bytes[pos]) {
            (Some(_), b'\\') => pos += 2,
            (Some(active), byte) if active == byte => {
                quote = None;
                pos += 1;
            }
            (None, b'\'' | b'"' | 0x60) => {
                quote = Some(bytes[pos]);
                pos += 1;
            }
            (None, b'}') if bytes[pos + 1] == b'}' => return Some(pos + 2),
            _ => pos += 1,
        }
    }
    None
}

fn find_close(bytes: &[u8], from: usize, name: &str) -> Option<(usize, usize)> {
    let mut pos = from;
    while pos < bytes.len() {
        if bytes[pos] == b'<'
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

fn find_bytes(bytes: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    bytes
        .get(from..)?
        .windows(needle.len())
        .position(|part| part == needle)
        .map(|offset| from + offset)
}

fn is_name(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_blocks_with_authored_byte_spans() {
        let source = "<!-- head -->\n<template lang='pug'>\n  p 日本語\n</template>\n<script setup lang=ts>const x = 1</script>\n<style scoped>.x { color: red }</style>";
        let allocator = Allocator::default();
        let result = Vue.split(&allocator, source);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.blocks.len(), 3);
        assert_eq!(result.blocks[0].attr("lang").unwrap().value, Some("pug"));
        assert_eq!(result.blocks[0].content.slice(source), "\n  p 日本語\n");
        assert_eq!(result.blocks[1].attr("setup").unwrap().value, None);
        assert_eq!(result.blocks[1].content.slice(source), "const x = 1");
        assert_eq!(
            result.blocks[2].close_tag.unwrap().slice(source),
            "</style>"
        );
    }

    #[test]
    fn nested_template_and_interpolation_do_not_end_root() {
        let source = "<template><template #default><p :x='\"</template>\"'>{{ '</template>' }}</p></template><div>ok</div></template>";
        let allocator = Allocator::default();
        let result = Vue.split(&allocator, source);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.blocks.len(), 1);
        assert!(
            result.blocks[0]
                .content
                .slice(source)
                .ends_with("<div>ok</div>")
        );
        assert_eq!(
            result.blocks[0].close_tag.unwrap().slice(source),
            "</template>"
        );
    }

    #[test]
    fn duplicate_and_missing_closer_report_offsets() {
        let source = "<template>x</template><template>y";
        let allocator = Allocator::default();
        let result = Vue.split(&allocator, source);
        assert_eq!(result.blocks.len(), 2);
        assert_eq!(result.errors.len(), 2);
        assert_eq!(result.errors[0].code, ContainerErrorCode::DuplicateBlock);
        assert_eq!(result.errors[1].code, ContainerErrorCode::MissingCloseTag);
        assert_eq!(result.blocks[1].content.slice(source), "y");
    }

    #[test]
    fn unclosed_interpolation_is_not_a_trusted_capture_boundary() {
        let source = "<template>{{ '</template>' </template>";
        let allocator = Allocator::default();
        let result = Vue.split(&allocator, source);
        assert!(
            result
                .errors
                .iter()
                .any(|error| error.code == ContainerErrorCode::UncertainInterpolation)
        );
    }

    #[test]
    fn long_closed_interpolation_has_no_artificial_limit() {
        let mut source = alloc::string::String::from("<template>{{ ");
        source.push_str(&"x".repeat(8192));
        source.push_str(" }}</template>");
        let allocator = Allocator::default();
        let result = Vue.split(&allocator, &source);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.blocks.len(), 1);
        assert_eq!(
            result.blocks[0].close_tag.unwrap().slice(&source),
            "</template>"
        );
    }
}
