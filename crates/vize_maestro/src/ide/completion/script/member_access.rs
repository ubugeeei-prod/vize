//! Member-access (`receiver.|`) completion: resolving `.value` on refs/computed
//! and lifting `reactive({ ... })` initializer keys from raw source.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, Documentation, MarkupContent, MarkupKind,
};
use vize_croquis::reactivity::ReactiveKind;

use super::context::{member_access_receiver, script_content_for_context};
use super::reactive_infer::{infer_reactive_value_type, reactive_kind_for_name};
use crate::ide::IdeContext;

pub(super) fn complete_member_access(
    ctx: &IdeContext,
    is_setup: bool,
) -> Option<Vec<CompletionItem>> {
    let receiver = member_access_receiver(&ctx.content, ctx.offset)?;
    let script_content = script_content_for_context(ctx, is_setup)?;
    let kind = reactive_kind_for_name(&script_content, receiver)?;

    if !kind.needs_value_access() {
        // reactive() / shallowReactive() bindings expose their object's own
        // keys directly. Without Corsa we can't resolve the type, but for
        // the common `reactive({ a: 1, b: '' })` form we can lift the
        // literal's keys from the source. Falls through to no completion
        // when the receiver is not a recognized reactive object.
        if matches!(kind, ReactiveKind::Reactive | ReactiveKind::ShallowReactive) {
            return Some(reactive_object_key_completions(&script_content, receiver));
        }
        return None;
    }

    let value_type = infer_reactive_value_type(&script_content, receiver, kind)
        .unwrap_or_else(|| "unknown".to_string());
    let readonly = kind == ReactiveKind::Computed;

    Some(vec![value_completion_item(&value_type, readonly)])
}

/// Extract the keys of `reactive({ ... })` / `shallowReactive({ ... })`
/// initializer literals so completion can offer them at `receiver.|` even
/// without a backing Corsa session. Returns an empty Vec when the initializer
/// is too dynamic (e.g. `reactive(someFn())`) — callers downstream handle
/// the empty case as "no completion".
fn reactive_object_key_completions(script_content: &str, name: &str) -> Vec<CompletionItem> {
    let initializer = match reactive_object_initializer(script_content, name) {
        Some(s) => s,
        None => return Vec::new(),
    };
    let keys = extract_object_literal_keys(initializer);
    keys.into_iter()
        .map(|key| {
            #[allow(clippy::disallowed_macros)]
            CompletionItem {
                label: key.clone(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("reactive property".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("Key inferred from `reactive(...)` initializer for `{name}`."),
                })),
                sort_text: Some(format!("0{key}")),
                ..Default::default()
            }
        })
        .collect()
}

fn reactive_object_initializer<'a>(script_content: &'a str, name: &str) -> Option<&'a str> {
    for callee in ["reactive", "shallowReactive"] {
        for keyword in ["const", "let"] {
            let needle = vize_carton::cstr!("{keyword} {name} = {callee}(");
            let Some(pos) = script_content.find(needle.as_str()) else {
                continue;
            };
            let after = &script_content[pos + needle.len()..];
            let after = after.trim_start();
            if after.starts_with('{') {
                return Some(after);
            }
        }
    }
    None
}

/// Extract top-level keys from a JS/TS object literal starting at `{`.
/// Handles nested braces, string keys, and trailing commas; returns names in
/// source order without duplicates.
fn extract_object_literal_keys(initializer: &str) -> Vec<String> {
    let bytes = initializer.as_bytes();
    if bytes.first() != Some(&b'{') {
        return Vec::new();
    }
    let mut depth = 0i32;
    let mut keys = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    let mut i = 0;
    let mut at_key_start = false;
    while i < bytes.len() {
        let b = bytes[i];
        match b {
            b'{' | b'[' | b'(' => {
                depth += 1;
                if depth == 1 {
                    at_key_start = true;
                }
            }
            b'}' | b']' | b')' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            b',' if depth == 1 => {
                at_key_start = true;
            }
            b' ' | b'\t' | b'\n' | b'\r' => {}
            b'"' | b'\'' | b'`' if depth == 1 && at_key_start => {
                let quote = b;
                let key_start = i + 1;
                let mut j = key_start;
                while j < bytes.len() && bytes[j] != quote {
                    j += 1;
                }
                if let Ok(key) = std::str::from_utf8(&bytes[key_start..j])
                    && seen.insert(key.to_string())
                {
                    keys.push(key.to_string());
                }
                i = j;
                at_key_start = false;
            }
            b if depth == 1
                && at_key_start
                && (b.is_ascii_alphabetic() || b == b'_' || b == b'$') =>
            {
                let key_start = i;
                let mut j = i;
                while j < bytes.len()
                    && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_' || bytes[j] == b'$')
                {
                    j += 1;
                }
                if let Ok(key) = std::str::from_utf8(&bytes[key_start..j])
                    && seen.insert(key.to_string())
                {
                    keys.push(key.to_string());
                }
                i = j.saturating_sub(1);
                at_key_start = false;
            }
            _ => {}
        }
        i += 1;
    }
    keys
}

#[allow(clippy::disallowed_macros)]
fn value_completion_item(value_type: &str, readonly: bool) -> CompletionItem {
    CompletionItem {
        label: "value".to_string(),
        kind: Some(CompletionItemKind::PROPERTY),
        detail: Some(if readonly {
            format!("readonly value: {value_type}")
        } else {
            format!("value: {value_type}")
        }),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: if readonly {
                format!(
                    "Readonly computed value.\n\n```typescript\nreadonly value: {value_type}\n```"
                )
            } else {
                format!("Inner ref value.\n\n```typescript\nvalue: {value_type}\n```")
            },
        })),
        sort_text: Some("0value".to_string()),
        ..Default::default()
    }
}
