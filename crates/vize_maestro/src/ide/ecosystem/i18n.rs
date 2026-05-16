//! Same-file vue-i18n editor helpers.

use serde_json::Value;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, Documentation, InlayHint, InlayHintKind, InlayHintLabel,
    Position, Range,
};
use vize_atelier_sfc::SfcDescriptor;
use vize_carton::{FxHashMap, String};

use super::context::string_literal_at_cursor;
use crate::ide::offset_to_position;

const I18N_CALLS: &[&str] = &["$t", "$te", "$tm", "t", "te", "tm"];
const PREVIEW_LIMIT: usize = 48;

#[derive(Debug, Default)]
pub(crate) struct I18nCatalog {
    entries: FxHashMap<String, String>,
    keys: Vec<String>,
}

impl I18nCatalog {
    fn insert(&mut self, key: String, message: &str) {
        if key.is_empty() || self.entries.contains_key(&key) {
            return;
        }

        self.entries.insert(key.clone(), String::from(message));
        self.keys.push(key);
    }

    fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    fn message(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(String::as_str)
    }
}

pub(crate) fn completions(
    content: &str,
    offset: usize,
    descriptor: &SfcDescriptor<'_>,
) -> Vec<CompletionItem> {
    if !is_i18n_call_string(content, offset) {
        return Vec::new();
    }

    let catalog = collect_catalog(descriptor);
    if catalog.is_empty() {
        return Vec::new();
    }

    catalog
        .keys
        .iter()
        .map(|key| CompletionItem {
            label: key.to_string(),
            kind: Some(CompletionItemKind::PROPERTY),
            detail: Some(std::string::String::from("vue-i18n message")),
            documentation: catalog
                .message(key)
                .map(|message| Documentation::String(message.to_string())),
            sort_text: Some(String::from("0").to_string()),
            ..Default::default()
        })
        .collect()
}

pub(crate) fn collect_inlay_hints(
    content: &str,
    descriptor: &SfcDescriptor<'_>,
    range: Range,
    hints: &mut Vec<InlayHint>,
) {
    let catalog = collect_catalog(descriptor);
    if catalog.is_empty() {
        return;
    }

    collect_call_hints(content, "$t", &catalog, range, hints);
    collect_call_hints(content, "t", &catalog, range, hints);
}

pub(crate) fn collect_catalog(descriptor: &SfcDescriptor<'_>) -> I18nCatalog {
    let mut catalog = I18nCatalog::default();

    for block in &descriptor.custom_blocks {
        if block.block_type != "i18n" || !is_json_block(descriptor.source.as_ref(), block) {
            continue;
        }

        let Ok(value) = serde_json::from_str::<Value>(&block.content) else {
            continue;
        };

        if has_locale_roots(&value) {
            if let Value::Object(locales) = &value {
                for messages in locales.values() {
                    collect_json_messages(String::default(), messages, &mut catalog);
                }
            }
        } else {
            collect_json_messages(String::default(), &value, &mut catalog);
        }
    }

    catalog
}

fn is_i18n_call_string(content: &str, offset: usize) -> bool {
    let Some(ctx) = string_literal_at_cursor(content, offset) else {
        return false;
    };

    let mut before = ctx.before_open.trim_end();
    let Some(rest) = before.strip_suffix('(') else {
        return false;
    };
    before = rest.trim_end();

    I18N_CALLS
        .iter()
        .any(|name| ends_with_call_name(before, name))
}

fn ends_with_call_name(text: &str, name: &str) -> bool {
    let Some(prefix) = text.strip_suffix(name) else {
        return false;
    };
    prefix
        .as_bytes()
        .last()
        .map(|byte| !super::context::is_ident_byte(*byte))
        .unwrap_or(true)
}

fn is_json_block(source: &str, block: &vize_atelier_sfc::SfcCustomBlock<'_>) -> bool {
    if block
        .attrs
        .get("lang")
        .is_some_and(|lang| lang.as_ref() != "json")
    {
        return false;
    }

    let open_tag = &source[block.loc.tag_start..block.loc.start.min(source.len())];
    open_tag.contains("<i18n")
}

fn has_locale_roots(value: &Value) -> bool {
    let Value::Object(object) = value else {
        return false;
    };
    !object.is_empty()
        && object
            .iter()
            .all(|(key, value)| is_locale_key(key) && value.is_object())
}

fn is_locale_key(key: &str) -> bool {
    let bytes = key.as_bytes();
    if bytes.len() < 2 || bytes.len() > 8 {
        return false;
    }
    bytes
        .iter()
        .all(|byte| byte.is_ascii_alphabetic() || *byte == b'-' || *byte == b'_')
}

fn collect_json_messages(prefix: String, value: &Value, catalog: &mut I18nCatalog) {
    match value {
        Value::String(message) => catalog.insert(prefix, message),
        Value::Object(object) => {
            for (key, value) in object {
                let mut next = String::with_capacity(prefix.len() + key.len() + 1);
                if !prefix.is_empty() {
                    next.push_str(&prefix);
                    next.push('.');
                }
                next.push_str(key);
                collect_json_messages(next, value, catalog);
            }
        }
        _ => {}
    }
}

fn collect_call_hints(
    content: &str,
    call_name: &str,
    catalog: &I18nCatalog,
    range: Range,
    hints: &mut Vec<InlayHint>,
) {
    let mut pos = 0usize;
    while let Some(found) = content[pos..].find(call_name) {
        let call_start = pos + found;
        if call_start > 0
            && content
                .as_bytes()
                .get(call_start - 1)
                .is_some_and(|byte| super::context::is_ident_byte(*byte))
        {
            pos = call_start + call_name.len();
            continue;
        }

        let Some((key, key_start, key_end)) =
            literal_first_arg(content, call_start + call_name.len())
        else {
            pos = call_start + call_name.len();
            continue;
        };

        if let Some(message) = catalog.message(key) {
            let (line, character) = offset_to_position(content, key_end);
            let position = Position { line, character };
            if super::position_in_range(position, range) {
                hints.push(InlayHint {
                    position,
                    label: InlayHintLabel::String(preview_label(message)),
                    kind: Some(InlayHintKind::TYPE),
                    text_edits: None,
                    tooltip: Some(tower_lsp::lsp_types::InlayHintTooltip::String(
                        key.to_string(),
                    )),
                    padding_left: Some(true),
                    padding_right: None,
                    data: None,
                });
            }
        }

        pos = key_start + 1;
    }
}

fn literal_first_arg(content: &str, mut pos: usize) -> Option<(&str, usize, usize)> {
    let bytes = content.as_bytes();
    skip_ascii_ws(bytes, &mut pos);
    if bytes.get(pos).copied()? != b'(' {
        return None;
    }
    pos += 1;
    skip_ascii_ws(bytes, &mut pos);

    let quote = bytes.get(pos).copied()?;
    if quote != b'\'' && quote != b'"' {
        return None;
    }
    let start = pos + 1;
    pos = start;
    while pos < bytes.len() {
        if bytes[pos] == quote && !is_escaped(bytes, pos) {
            return Some((&content[start..pos], start, pos));
        }
        if bytes[pos] == b'\n' || bytes[pos] == b'\r' {
            return None;
        }
        pos += 1;
    }
    None
}

fn skip_ascii_ws(bytes: &[u8], pos: &mut usize) {
    while bytes.get(*pos).is_some_and(u8::is_ascii_whitespace) {
        *pos += 1;
    }
}

fn is_escaped(bytes: &[u8], quote: usize) -> bool {
    let mut slash_count = 0usize;
    let mut pos = quote;
    while pos > 0 && bytes[pos - 1] == b'\\' {
        slash_count += 1;
        pos -= 1;
    }
    slash_count % 2 == 1
}

fn preview_label(message: &str) -> std::string::String {
    let mut label = std::string::String::from("= ");
    for (index, ch) in message.chars().enumerate() {
        if index >= PREVIEW_LIMIT {
            label.push_str("...");
            break;
        }
        if ch == '\n' || ch == '\r' {
            label.push(' ');
        } else {
            label.push(ch);
        }
    }
    label
}

#[cfg(test)]
mod tests {
    use super::{collect_catalog, is_i18n_call_string};

    #[test]
    fn detects_i18n_call_strings() {
        let source = r#"{{ $t("greeting") }} {{ t('farewell') }}"#;

        assert!(is_i18n_call_string(
            source,
            source.find("greeting").unwrap()
        ));
        assert!(is_i18n_call_string(
            source,
            source.find("farewell").unwrap()
        ));
    }

    #[test]
    fn collects_locale_stripped_json_catalog_keys() {
        let source = r#"<template>{{ $t("auth.login") }}</template>
<i18n lang="json">
{
  "en": { "auth": { "login": "Log in" } },
  "ja": { "auth": { "login": "ログイン" } }
}
</i18n>
"#;
        let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();
        let catalog = collect_catalog(&descriptor);

        assert_eq!(catalog.message("auth.login"), Some("Log in"));
        assert!(catalog.message("en.auth.login").is_none());
    }
}
