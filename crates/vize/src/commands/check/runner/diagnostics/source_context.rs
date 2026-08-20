use std::{collections::BTreeMap, fs, path::PathBuf};

use vize_carton::{String as CompactString, cstr};

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
        if let Some(binding) = binding_context(line, diagnostic.column as usize) {
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
    source_token_at(line, column)
        .and_then(binding_context_from_token)
        .or_else(|| binding_context_from_line(line, column))
}

fn binding_context_from_line(line: &str, column: usize) -> Option<CompactString> {
    if !line.trim_start().starts_with('<') {
        return None;
    }

    let mut cursor = 0;
    let mut best = None;
    let mut best_distance = usize::MAX;
    while cursor < line.len() {
        if !line.is_char_boundary(cursor) {
            cursor += 1;
            continue;
        }

        if !contextual_binding_starts_at(line, cursor) {
            cursor += 1;
            continue;
        }

        let mut end = cursor;
        let bytes = line.as_bytes();
        while end < bytes.len() && is_template_binding_byte(bytes[end]) {
            end += 1;
        }
        if let Some(context) = binding_context_from_token(&line[cursor..end]) {
            let distance = cursor.abs_diff(column);
            if distance < best_distance {
                best = Some(context);
                best_distance = distance;
            }
        }
        cursor = end.max(cursor + 1);
    }
    best
}

fn contextual_binding_starts_at(line: &str, cursor: usize) -> bool {
    let rest = &line[cursor..];
    rest.starts_with(':')
        || rest.starts_with('@')
        || rest.starts_with('#')
        || rest.starts_with("v-bind:")
        || rest.starts_with("v-model")
        || rest.starts_with("v-on:")
        || rest.starts_with("v-slot")
}

fn source_token_at(line: &str, column: usize) -> Option<&str> {
    let bytes = line.as_bytes();
    let mut cursor = column.min(bytes.len());
    while cursor > 0 && !line.is_char_boundary(cursor) {
        cursor -= 1;
    }
    let mut start = cursor;
    while start > 0 && is_template_binding_byte(bytes[start - 1]) {
        start -= 1;
    }
    let mut end = cursor;
    while end < bytes.len() && is_template_binding_byte(bytes[end]) {
        end += 1;
    }
    (start < end).then(|| &line[start..end])
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

#[cfg(test)]
mod tests {
    use super::{binding_context, binding_context_from_token};

    fn context_at(line: &str, column: usize) -> Option<std::string::String> {
        binding_context(line, column).map(|context| context.to_string())
    }

    fn token_context(token: &str) -> Option<std::string::String> {
        binding_context_from_token(token).map(|context| context.to_string())
    }

    #[test]
    fn binding_context_falls_back_to_authored_bound_prop_on_template_lines() {
        assert_eq!(
            context_at(r#"<Child kind="num" :s="'bad'" />"#, 2).as_deref(),
            Some("'s'")
        );
        assert_eq!(
            context_at(
                r#"<Child :model-value="1" kind="num" :n="1" v-slot="{ count }">{{ count.toUpperCase() }}</Child>"#,
                73,
            )
            .as_deref(),
            Some("#default")
        );
    }

    #[test]
    fn binding_context_maps_event_and_slot_directive_tokens() {
        assert_eq!(token_context("@save.once").as_deref(), Some("@save"));
        assert_eq!(token_context("v-on:submit").as_deref(), Some("@submit"));
        assert_eq!(token_context("#item").as_deref(), Some("#item"));
        assert_eq!(token_context("v-slot:default").as_deref(), Some("#default"));
        assert_eq!(token_context("v-slot").as_deref(), Some("#default"));
    }
}
