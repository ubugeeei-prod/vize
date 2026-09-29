use std::{collections::BTreeMap, fs, path::PathBuf};

use vize_l0::{String as CompactString, cstr};

#[derive(Default)]
pub(super) struct SourceContextCache {
    lines: BTreeMap<PathBuf, Option<Vec<CompactString>>>,
}

impl SourceContextCache {
    pub(super) fn render(
        &mut self,
        diagnostic: &vize_canon::BatchDiagnostic,
    ) -> Option<CompactString> {
        let lines = self
            .lines
            .entry(diagnostic.file.clone())
            .or_insert_with(|| {
                fs::read_to_string(&diagnostic.file)
                    .ok()
                    .map(|source| source.lines().map(CompactString::from).collect())
            })
            .as_ref()?;
        let line = lines.get(diagnostic.line as usize)?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }
        let mut context = truncate(trimmed);
        let column = utf16_column_to_byte_offset(line, diagnostic.column as usize);
        if let Some(binding) = binding_context(line, column) {
            context.push_str("; binding: ");
            context.push_str(binding.as_str());
        }
        Some(context)
    }
}

fn truncate(line: &str) -> CompactString {
    const MAX_CHARS: usize = 160;
    let mut output = CompactString::default();
    for (index, character) in line.chars().enumerate() {
        if index == MAX_CHARS {
            output.push_str("...");
            break;
        }
        output.push(character);
    }
    output
}

fn binding_context(line: &str, column: usize) -> Option<CompactString> {
    if start_tag_end(line).is_some_and(|end| column >= end) {
        return None;
    }
    source_token_at(line, column)
        .and_then(|(start, token)| attribute_context(line, start, token))
        .or_else(|| binding_context_from_line(line, column))
}

fn start_tag_end(line: &str) -> Option<usize> {
    if !line.trim_start().starts_with('<') {
        return None;
    }
    let mut quote = None;
    for (index, byte) in line.bytes().enumerate() {
        if let Some(active_quote) = quote {
            if byte == active_quote {
                quote = None;
            }
        } else if matches!(byte, b'\'' | b'"') {
            quote = Some(byte);
        } else if byte == b'>' {
            return Some(index + 1);
        }
    }
    None
}

fn utf16_column_to_byte_offset(line: &str, column: usize) -> usize {
    if column == 0 {
        return 0;
    }
    let mut byte_offset = 0;
    let mut utf16_column = 0;
    for character in line.chars() {
        byte_offset += character.len_utf8();
        utf16_column += character.len_utf16();
        if utf16_column >= column {
            return byte_offset;
        }
    }
    byte_offset
}

fn binding_context_from_line(line: &str, column: usize) -> Option<CompactString> {
    if !line.trim_start().starts_with('<') {
        return None;
    }

    let mut cursor = 0;
    let mut best = None;
    let mut best_distance = usize::MAX;
    let mut quote = None;
    let bytes = line.as_bytes();
    while let Some(&byte) = bytes.get(cursor) {
        if !line.is_char_boundary(cursor) {
            cursor += 1;
            continue;
        }

        if let Some(active_quote) = quote {
            if byte == active_quote {
                quote = None;
            }
            cursor += 1;
            continue;
        }
        if matches!(byte, b'\'' | b'"') {
            quote = Some(byte);
            cursor += 1;
            continue;
        }

        if !is_attribute_boundary(line, cursor) || is_inside_quoted_attribute_value(line, cursor) {
            cursor += 1;
            continue;
        }

        let mut end = cursor;
        while bytes
            .get(end)
            .is_some_and(|&byte| is_template_binding_byte(byte))
        {
            end += 1;
        }
        if let Some(context) =
            attribute_context(line, cursor, line.get(cursor..end).unwrap_or_default())
        {
            if (cursor..attribute_end(line, end)).contains(&column) {
                return Some(context);
            }
            // A diagnostic on the component tag can only be associated with a
            // nearby directive. A static prop is relevant when the position
            // actually falls inside that attribute, not merely on the same tag.
            if contextual_binding_starts_at(line, cursor) {
                let distance = cursor.abs_diff(column);
                if distance < best_distance {
                    best = Some(context);
                    best_distance = distance;
                }
            }
        }
        cursor = end.max(cursor + 1);
    }
    best
}

fn attribute_context(line: &str, cursor: usize, token: &str) -> Option<CompactString> {
    if !is_attribute_boundary(line, cursor) || is_inside_quoted_attribute_value(line, cursor) {
        return None;
    }
    binding_context_from_token(token).or_else(|| {
        let after_name = line.get(cursor + token.len()..)?;
        (line.trim_start().starts_with('<')
            && token
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_alphabetic)
            && after_name.trim_start().starts_with('='))
        .then(|| CompactString::from(token))
    })
}

fn attribute_end(line: &str, name_end: usize) -> usize {
    let bytes = line.as_bytes();
    let mut cursor = name_end;
    while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
        cursor += 1;
    }
    if bytes.get(cursor) != Some(&b'=') {
        return name_end;
    }
    cursor += 1;
    while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
        cursor += 1;
    }
    if let Some(&quote @ (b'\'' | b'"')) = bytes.get(cursor) {
        cursor += 1;
        while let Some(&byte) = bytes.get(cursor) {
            cursor += 1;
            if byte == quote {
                break;
            }
        }
    } else {
        while bytes
            .get(cursor)
            .is_some_and(|&byte| !byte.is_ascii_whitespace() && byte != b'>')
        {
            cursor += 1;
        }
    }
    cursor
}

fn is_attribute_boundary(line: &str, cursor: usize) -> bool {
    cursor
        .checked_sub(1)
        .and_then(|prev| line.as_bytes().get(prev))
        .is_none_or(u8::is_ascii_whitespace)
}

fn is_inside_quoted_attribute_value(line: &str, cursor: usize) -> bool {
    let mut quote = None;
    for (index, byte) in line.bytes().enumerate() {
        if index >= cursor {
            break;
        }
        if let Some(active_quote) = quote {
            if byte == active_quote {
                quote = None;
            }
        } else if matches!(byte, b'\'' | b'"') {
            quote = Some(byte);
        }
    }
    quote.is_some()
}

fn contextual_binding_starts_at(line: &str, cursor: usize) -> bool {
    let rest = line.get(cursor..).unwrap_or_default();
    rest.starts_with(':')
        || rest.starts_with('@')
        || rest.starts_with('#')
        || rest.starts_with("v-bind:")
        || rest.starts_with("v-model")
        || rest.starts_with("v-on:")
        || rest.starts_with("v-slot")
}

fn source_token_at(line: &str, column: usize) -> Option<(usize, &str)> {
    let bytes = line.as_bytes();
    let mut cursor = column.min(bytes.len());
    while cursor > 0 && !line.is_char_boundary(cursor) {
        cursor -= 1;
    }
    let mut start = cursor;
    while start
        .checked_sub(1)
        .and_then(|prev| bytes.get(prev))
        .is_some_and(|&byte| is_template_binding_byte(byte))
    {
        start -= 1;
    }
    let mut end = cursor;
    while bytes
        .get(end)
        .is_some_and(|&byte| is_template_binding_byte(byte))
    {
        end += 1;
    }
    (start < end).then(|| (start, line.get(start..end).unwrap_or_default()))
}

fn is_template_binding_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'@' | b'#' | b'-' | b'_' | b'.')
}

fn binding_context_from_token(token: &str) -> Option<CompactString> {
    if let Some(rest) = token.strip_prefix("v-model:") {
        return binding_name_context(rest);
    }
    if token.starts_with("v-model") {
        return Some(CompactString::from("modelValue"));
    }
    if let Some(rest) = token.strip_prefix(':') {
        return quoted_binding_name_context(rest);
    }
    if let Some(rest) = token.strip_prefix("v-bind:") {
        return quoted_binding_name_context(rest);
    }
    if let Some(rest) = token.strip_prefix('@') {
        return prefixed_binding_name_context("@", rest);
    }
    if let Some(rest) = token.strip_prefix("v-on:") {
        return prefixed_binding_name_context("@", rest);
    }
    if let Some(rest) = token.strip_prefix('#') {
        return prefixed_binding_name_context("#", rest);
    }
    if let Some(rest) = token.strip_prefix("v-slot:") {
        return prefixed_binding_name_context("#", rest);
    }
    if token.starts_with("v-slot") {
        return Some(CompactString::from("#default"));
    }
    None
}

fn binding_name_context(token: &str) -> Option<CompactString> {
    let name = token.split('.').next()?.trim();
    (!name.is_empty()).then(|| CompactString::from(name))
}

fn quoted_binding_name_context(token: &str) -> Option<CompactString> {
    let name = binding_name_context(token)?;
    Some(cstr!("'{name}'"))
}

fn prefixed_binding_name_context(prefix: &str, token: &str) -> Option<CompactString> {
    let name = binding_name_context(token)?;
    Some(cstr!("{prefix}{name}"))
}

#[expect(clippy::string_slice, reason = "tests assert by panicking")]
#[expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#[expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#[cfg(test)]
mod tests {
    include!("source_context_tests.rs");
}
