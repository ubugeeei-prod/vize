//! Completion items for declared component events.

use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, InsertTextFormat};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::ide::completion::template) struct ComponentEmit {
    pub(in crate::ide::completion::template) name: String,
    pub(in crate::ide::completion::template) payload: Option<String>,
}

pub(super) fn completion_items(emits: &[ComponentEmit]) -> Vec<CompletionItem> {
    emits
        .iter()
        .map(|event| {
            let label = format!("@{}", event.name);
            CompletionItem {
                label: label.clone(),
                kind: Some(CompletionItemKind::EVENT),
                detail: Some(event.payload.as_ref().map_or_else(
                    || "component event".to_string(),
                    |payload| format!("event: {payload}"),
                )),
                insert_text: Some(format!("{label}=\"$1\"")),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                sort_text: Some(format!("00-event-{}", event.name)),
                ..Default::default()
            }
        })
        .collect()
}
