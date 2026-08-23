//! Template-ref attribute support.
//!
//! Static `ref="name"` attributes are not Vue template expressions, so the
//! canonical template virtual document has no expression position to query.
//! They still have an authored TypeScript declaration when the component uses
//! `useTemplateRef("name")`; this module maps the authored attribute value to
//! that declaration.

use tower_lsp::lsp_types::{Location, Position, Range};

use super::{IdeContext, definition::script};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TemplateRefTarget {
    pub(crate) ref_name: String,
    pub(crate) value_start: usize,
    pub(crate) value_end: usize,
    pub(crate) binding_name: String,
    pub(crate) binding_start: usize,
    pub(crate) binding_end: usize,
}

impl TemplateRefTarget {
    pub(crate) fn value_range(&self, content: &str) -> Range {
        range_from_offsets(content, self.value_start, self.value_end)
    }

    pub(crate) fn binding_location(&self, ctx: &IdeContext<'_>) -> Location {
        Location {
            uri: ctx.uri.clone(),
            range: range_from_offsets(&ctx.content, self.binding_start, self.binding_end),
        }
    }
}

pub(crate) fn target_at_offset(ctx: &IdeContext<'_>) -> Option<TemplateRefTarget> {
    let value = static_ref_value_at_offset(&ctx.content, ctx.offset)?;
    let binding = use_template_ref_binding(ctx, &value.ref_name)?;

    Some(TemplateRefTarget {
        ref_name: value.ref_name,
        value_start: value.start,
        value_end: value.end,
        binding_end: binding.start + binding.name.len(),
        binding_name: binding.name,
        binding_start: binding.start,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StaticRefValue {
    ref_name: String,
    start: usize,
    end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TemplateRefBinding {
    name: String,
    start: usize,
}

fn static_ref_value_at_offset(content: &str, offset: usize) -> Option<StaticRefValue> {
    let offset = normalize_offset(content, offset);
    let tag_start = content[..offset].rfind('<')?;
    if content[tag_start..offset].contains('>') {
        return None;
    }
    let tag_end = content[offset..]
        .find('>')
        .map(|relative| offset + relative)
        .unwrap_or(content.len());

    let bytes = content.as_bytes();
    let mut cursor = tag_start + 1;
    while cursor + "ref".len() <= tag_end {
        let Some(relative) = content[cursor..tag_end].find("ref") else {
            break;
        };
        let name_start = cursor + relative;
        let name_end = name_start + "ref".len();
        cursor = name_end;

        if !is_attribute_name_boundary(bytes, name_start, name_end) {
            continue;
        }

        let mut pos = skip_ascii_whitespace(bytes, name_end);
        if bytes.get(pos) != Some(&b'=') {
            continue;
        }
        pos += 1;
        pos = skip_ascii_whitespace(bytes, pos);
        let quote = *bytes.get(pos)?;
        if quote != b'\'' && quote != b'"' {
            continue;
        }
        let value_start = pos + 1;
        let value_end = find_quote(bytes, value_start, tag_end, quote)?;
        if !(value_start <= offset && offset <= value_end) {
            continue;
        }
        let ref_name = &content[value_start..value_end];
        if !is_identifier_name(ref_name) {
            return None;
        }
        return Some(StaticRefValue {
            ref_name: ref_name.to_string(),
            start: value_start,
            end: value_end,
        });
    }

    None
}

fn use_template_ref_binding(ctx: &IdeContext<'_>, ref_name: &str) -> Option<TemplateRefBinding> {
    let descriptor = vize_atelier_sfc::parse_sfc(
        &ctx.content,
        vize_atelier_sfc::SfcParseOptions {
            filename: ctx.uri.path().to_string().into(),
            ..Default::default()
        },
    )
    .ok()?;
    let script_setup = descriptor.script_setup.as_ref()?;
    let content = script_setup.content.as_ref();

    let mut search_start = 0;
    while let Some(relative) = content[search_start..].find("useTemplateRef") {
        let callee_start = search_start + relative;
        let callee_end = callee_start + "useTemplateRef".len();
        search_start = callee_end;
        if !is_identifier_boundary(content.as_bytes(), callee_start, callee_end) {
            continue;
        }
        if first_static_argument(content, callee_end).as_deref() != Some(ref_name) {
            continue;
        }
        let binding_name = variable_binding_before_call(content, callee_start)?;
        let binding = script::find_binding_location_raw(content, &binding_name)?;
        return Some(TemplateRefBinding {
            start: script_setup.loc.start + binding.offset,
            name: binding.name,
        });
    }

    None
}

fn first_static_argument(content: &str, callee_end: usize) -> Option<String> {
    let bytes = content.as_bytes();
    let mut pos = skip_ascii_whitespace(bytes, callee_end);
    if bytes.get(pos) == Some(&b'<') {
        pos = find_matching_byte(content, pos, b'<', b'>')? + 1;
        pos = skip_ascii_whitespace(bytes, pos);
    }
    if bytes.get(pos) != Some(&b'(') {
        return None;
    }
    pos += 1;
    pos = skip_ascii_whitespace(bytes, pos);
    let quote = *bytes.get(pos)?;
    if quote != b'\'' && quote != b'"' {
        return None;
    }
    let value_start = pos + 1;
    let value_end = find_quote(bytes, value_start, content.len(), quote)?;
    Some(unescape_simple_string(
        &content[value_start..value_end],
        quote,
    ))
}

fn variable_binding_before_call(content: &str, callee_start: usize) -> Option<String> {
    let statement_start = content[..callee_start]
        .rfind(['\n', ';'])
        .map_or(0, |offset| offset + 1);
    let prefix = &content[statement_start..callee_start];
    let eq = prefix.rfind('=')?;
    let before_eq = prefix[..eq].trim_end();

    for keyword in ["const", "let", "var"] {
        let Some(keyword_start) = before_eq.rfind(keyword) else {
            continue;
        };
        let keyword_end = keyword_start + keyword.len();
        if keyword_start > 0 && is_identifier_byte(before_eq.as_bytes()[keyword_start - 1]) {
            continue;
        }
        let after_keyword = before_eq[keyword_end..].trim_start();
        let name = after_keyword
            .split(|ch: char| !is_identifier_char(ch))
            .next()
            .unwrap_or_default();
        if is_identifier_name(name) {
            return Some(name.to_string());
        }
    }

    None
}

fn normalize_offset(content: &str, mut offset: usize) -> usize {
    offset = offset.min(content.len());
    while offset > 0 && !content.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

fn is_attribute_name_boundary(bytes: &[u8], start: usize, end: usize) -> bool {
    let before = start.checked_sub(1).and_then(|index| bytes.get(index));
    let after = bytes.get(end);
    !before.is_some_and(|byte| is_attribute_name_byte(*byte))
        && !after.is_some_and(|byte| is_attribute_name_byte(*byte))
}

fn is_attribute_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b':' | b'.' | b'@')
}

fn is_identifier_boundary(bytes: &[u8], start: usize, end: usize) -> bool {
    let before = start.checked_sub(1).and_then(|index| bytes.get(index));
    let after = bytes.get(end);
    !before.is_some_and(|byte| is_identifier_byte(*byte))
        && !after.is_some_and(|byte| is_identifier_byte(*byte))
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$')
}

fn is_identifier_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '$'
}

fn is_identifier_name(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_alphabetic() || first == '_' || first == '$') {
        return false;
    }
    chars.all(is_identifier_char)
}

fn skip_ascii_whitespace(bytes: &[u8], mut pos: usize) -> usize {
    while bytes.get(pos).is_some_and(u8::is_ascii_whitespace) {
        pos += 1;
    }
    pos
}

fn find_quote(bytes: &[u8], mut pos: usize, end: usize, quote: u8) -> Option<usize> {
    while pos < end {
        match bytes[pos] {
            b'\\' => pos += 2,
            byte if byte == quote => return Some(pos),
            _ => pos += 1,
        }
    }
    None
}

fn find_matching_byte(content: &str, start: usize, open: u8, close: u8) -> Option<usize> {
    let bytes = content.as_bytes();
    if bytes.get(start) != Some(&open) {
        return None;
    }
    let mut depth = 0usize;
    let mut pos = start;
    while pos < bytes.len() {
        match bytes[pos] {
            b'\'' | b'"' => {
                pos = find_quote(bytes, pos + 1, bytes.len(), bytes[pos])? + 1;
                continue;
            }
            byte if byte == open => depth += 1,
            byte if byte == close => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(pos);
                }
            }
            _ => {}
        }
        pos += 1;
    }
    None
}

fn unescape_simple_string(value: &str, quote: u8) -> String {
    let quote = quote as char;
    let mut output = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\'
            && let Some(next) = chars.next()
        {
            if next == quote || next == '\\' {
                output.push(next);
                continue;
            }
            output.push(ch);
            output.push(next);
            continue;
        }
        output.push(ch);
    }
    output
}

fn range_from_offsets(content: &str, start: usize, end: usize) -> Range {
    let (start_line, start_character) = super::offset_to_position(content, start);
    let (end_line, end_character) = super::offset_to_position(content, end);
    Range {
        start: Position {
            line: start_line,
            character: start_character,
        },
        end: Position {
            line: end_line,
            character: end_character,
        },
    }
}

#[cfg(test)]
mod tests;
