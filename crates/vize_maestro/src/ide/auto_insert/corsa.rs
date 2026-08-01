//! Corsa-backed Vue ref classification for automatic `.value` insertion.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use std::sync::Arc;

use vize_canon::{CorsaBridge, LspHoverContents, LspMarkedString};

use super::IdeContext;

pub(super) async fn dot_value(
    ctx: &IdeContext<'_>,
    selection: usize,
    bridge: Option<Arc<CorsaBridge>>,
) -> Option<String> {
    let bridge = bridge?;
    if !bridge.is_initialized() {
        return None;
    }
    let document =
        super::super::corsa_support::open_canonical_virtual_document(ctx, &bridge).await?;
    let (line, character) =
        super::super::corsa_support::canonical_source_offset_to_position(&document, selection)?;
    let hover = bridge
        .hover(&document.request_uri, line, character)
        .await
        .ok()??;
    hover_is_ref(hover.contents).then(|| "${1:.value}".to_string())
}

pub(super) fn hover_is_ref(contents: LspHoverContents) -> bool {
    match contents {
        LspHoverContents::Markup(markup) => {
            let quick_info = if markup.kind == "markdown" {
                markdown_quick_info(&markup.value)
            } else {
                first_nonempty_line(&markup.value)
            };
            quick_info.is_some_and(quick_info_has_ref_type)
        }
        LspHoverContents::String(value) => {
            first_nonempty_line(&value).is_some_and(quick_info_has_ref_type)
        }
        LspHoverContents::Array(items) => {
            let mut legacy_string = None;
            let mut saw_language_string = false;
            for item in items {
                match item {
                    LspMarkedString::LanguageString { value, .. } => {
                        saw_language_string = true;
                        if quick_info_has_ref_type(&value) {
                            return true;
                        }
                    }
                    LspMarkedString::String(value) if legacy_string.is_none() => {
                        legacy_string = Some(value);
                    }
                    _ => {}
                }
            }
            if saw_language_string {
                return false;
            }
            legacy_string
                .as_deref()
                .and_then(first_nonempty_line)
                .is_some_and(quick_info_has_ref_type)
        }
    }
}

fn markdown_quick_info(markdown: &str) -> Option<&str> {
    let fence = markdown.find("```")?;
    let after_fence = markdown.get(fence + 3..)?;
    let code_start = after_fence.find('\n')? + 1;
    let code = after_fence.get(code_start..)?;
    let code_end = code.find("```")?;
    code.get(..code_end)
}

fn first_nonempty_line(value: &str) -> Option<&str> {
    value.lines().find(|line| !line.trim().is_empty())
}

fn quick_info_has_ref_type(value: &str) -> bool {
    [
        "Ref<",
        "ShallowRef<",
        "ComputedRef<",
        "WritableComputedRef<",
    ]
    .iter()
    .any(|needle| {
        value.match_indices(needle).any(|(offset, _)| {
            offset == 0
                || !value.as_bytes()[offset - 1].is_ascii_alphanumeric()
                    && value.as_bytes()[offset - 1] != b'_'
        })
    })
}
