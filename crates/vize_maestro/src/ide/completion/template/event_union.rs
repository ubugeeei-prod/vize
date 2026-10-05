//! Declared component listeners take precedence over equivalent generic events.

use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind};

pub(super) fn append_component_surface(
    items: &mut Vec<CompletionItem>,
    component: Vec<CompletionItem>,
) {
    items.retain(|fallback| {
        !component.iter().any(|declared| {
            declared.kind == Some(CompletionItemKind::EVENT)
                && fallback.kind == declared.kind
                && fallback.label == declared.label
                && fallback.insert_text == declared.insert_text
                && fallback.insert_text_format == declared.insert_text_format
                && fallback.text_edit == declared.text_edit
                && fallback.additional_text_edits == declared.additional_text_edits
                && fallback.command == declared.command
        })
    });
    // Keep the declaration's complete type, documentation, ordering and data.
    items.extend(component);
}

#[cfg(test)]
#[path = "event_union_tests.rs"]
mod tests;
