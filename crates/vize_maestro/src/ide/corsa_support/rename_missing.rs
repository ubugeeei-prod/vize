use std::cmp::Ordering;

use tower_lsp::lsp_types::{
    AnnotatedTextEdit, DocumentChangeOperation, DocumentChanges, OneOf,
    OptionalVersionedTextDocumentIdentifier, Range, TextDocumentEdit, TextEdit, Url, WorkspaceEdit,
};

use crate::ide::IdeContext;

/// Keep the canonical rename answer, then append authored SFC edits for ranges
/// it missed.
///
/// The canonical path can preserve TypeScript-specific rewrite shapes such as
/// destructuring aliases. Authored rename is still valuable for Vue ranges the
/// canonical virtual document did not return, but it must not replace a range
/// the canonical answer already rewrote.
pub(crate) fn merge_missing_authored_rename(
    ctx: &IdeContext<'_>,
    canonical: Option<WorkspaceEdit>,
    authored: Option<WorkspaceEdit>,
) -> Option<WorkspaceEdit> {
    let Some(mut canonical) = canonical else {
        return authored.map(order_edits_by_position);
    };
    let Some(authored_edits) = authored
        .and_then(|edit| edit.changes)
        .and_then(|mut changes| changes.remove(ctx.uri))
        .filter(|edits| !edits.is_empty())
    else {
        return Some(order_edits_by_position(canonical));
    };

    if workspace_edit_touch_count(&canonical, ctx.uri) > 1 {
        return Some(order_edits_by_position(canonical));
    }

    let missing = authored_edits
        .into_iter()
        .filter(|edit| !workspace_edit_overlaps_range(&canonical, ctx.uri, edit.range))
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return Some(order_edits_by_position(canonical));
    }

    append_missing_text_edits(ctx, &mut canonical, missing);
    Some(order_edits_by_position(canonical))
}

fn append_missing_text_edits(
    ctx: &IdeContext<'_>,
    edit: &mut WorkspaceEdit,
    authored_edits: Vec<TextEdit>,
) {
    // Clients prefer `documentChanges` whenever it is present, so authored
    // supplemental edits have to land in the same container as the canonical
    // edits.
    match edit.document_changes.as_mut() {
        Some(DocumentChanges::Edits(edits)) => {
            append_to_document_edits(ctx, edits, authored_edits);
        }
        Some(DocumentChanges::Operations(operations)) => {
            append_to_operations(ctx, operations, authored_edits);
        }
        None => {
            let entry = edit
                .changes
                .get_or_insert_with(Default::default)
                .entry(ctx.uri.clone())
                .or_default();
            append_text_edits(entry, authored_edits);
        }
    }
}

fn append_to_document_edits(
    ctx: &IdeContext<'_>,
    edits: &mut Vec<TextDocumentEdit>,
    authored_edits: Vec<TextEdit>,
) {
    for edit in edits.iter_mut() {
        if edit.text_document.uri == *ctx.uri {
            append_annotatable_edits(&mut edit.edits, authored_edits);
            return;
        }
    }
    edits.push(authored_document_edit(ctx, authored_edits));
}

fn append_to_operations(
    ctx: &IdeContext<'_>,
    operations: &mut Vec<DocumentChangeOperation>,
    authored_edits: Vec<TextEdit>,
) {
    for operation in operations.iter_mut() {
        if let DocumentChangeOperation::Edit(edit) = operation
            && edit.text_document.uri == *ctx.uri
        {
            append_annotatable_edits(&mut edit.edits, authored_edits);
            return;
        }
    }
    operations.push(DocumentChangeOperation::Edit(authored_document_edit(
        ctx,
        authored_edits,
    )));
}

fn authored_document_edit(ctx: &IdeContext<'_>, authored_edits: Vec<TextEdit>) -> TextDocumentEdit {
    TextDocumentEdit {
        text_document: OptionalVersionedTextDocumentIdentifier {
            uri: ctx.uri.clone(),
            version: None,
        },
        edits: authored_edits.into_iter().map(OneOf::Left).collect(),
    }
}

fn append_text_edits(edits: &mut Vec<TextEdit>, authored_edits: Vec<TextEdit>) {
    for edit in authored_edits {
        if !edits.iter().any(|kept| kept.range == edit.range) {
            edits.push(edit);
        }
    }
}

fn append_annotatable_edits(
    edits: &mut Vec<OneOf<TextEdit, AnnotatedTextEdit>>,
    authored_edits: Vec<TextEdit>,
) {
    for edit in authored_edits {
        if !edits
            .iter()
            .any(|kept| annotatable_edit_range(kept) == edit.range)
        {
            edits.push(OneOf::Left(edit));
        }
    }
}

fn workspace_edit_overlaps_range(edit: &WorkspaceEdit, uri: &Url, range: Range) -> bool {
    if edit
        .changes
        .as_ref()
        .and_then(|changes| changes.get(uri))
        .is_some_and(|edits| edits.iter().any(|edit| ranges_overlap(edit.range, range)))
    {
        return true;
    }

    match edit.document_changes.as_ref() {
        Some(DocumentChanges::Edits(edits)) => edits.iter().any(|edit| {
            edit.text_document.uri == *uri
                && edit
                    .edits
                    .iter()
                    .any(|edit| ranges_overlap(annotatable_edit_range(edit), range))
        }),
        Some(DocumentChanges::Operations(operations)) => operations.iter().any(|operation| {
            if let DocumentChangeOperation::Edit(edit) = operation {
                edit.text_document.uri == *uri
                    && edit
                        .edits
                        .iter()
                        .any(|edit| ranges_overlap(annotatable_edit_range(edit), range))
            } else {
                false
            }
        }),
        None => false,
    }
}

fn workspace_edit_touch_count(edit: &WorkspaceEdit, uri: &Url) -> usize {
    let mut count = edit
        .changes
        .as_ref()
        .and_then(|changes| changes.get(uri))
        .map_or(0, Vec::len);

    match edit.document_changes.as_ref() {
        Some(DocumentChanges::Edits(edits)) => {
            count += edits
                .iter()
                .filter(|edit| edit.text_document.uri == *uri)
                .map(|edit| edit.edits.len())
                .sum::<usize>();
        }
        Some(DocumentChanges::Operations(operations)) => {
            count += operations
                .iter()
                .filter_map(|operation| {
                    if let DocumentChangeOperation::Edit(edit) = operation
                        && edit.text_document.uri == *uri
                    {
                        Some(edit.edits.len())
                    } else {
                        None
                    }
                })
                .sum::<usize>();
        }
        None => {}
    }

    count
}

fn order_edits_by_position(mut edit: WorkspaceEdit) -> WorkspaceEdit {
    if let Some(changes) = edit.changes.as_mut() {
        for edits in changes.values_mut() {
            edits.sort_by(|a, b| compare_ranges(&a.range, &b.range));
        }
    }

    match edit.document_changes.as_mut() {
        Some(DocumentChanges::Edits(edits)) => {
            for edit in edits.iter_mut() {
                sort_annotatable_edits(&mut edit.edits);
            }
        }
        Some(DocumentChanges::Operations(operations)) => {
            for operation in operations.iter_mut() {
                if let DocumentChangeOperation::Edit(edit) = operation {
                    sort_annotatable_edits(&mut edit.edits);
                }
            }
        }
        None => {}
    }

    edit
}

fn sort_annotatable_edits(edits: &mut [OneOf<TextEdit, AnnotatedTextEdit>]) {
    edits.sort_by(|a, b| compare_ranges(&annotatable_edit_range(a), &annotatable_edit_range(b)));
}

fn compare_ranges(a: &Range, b: &Range) -> Ordering {
    compare_positions(a.start, b.start).then(compare_positions(a.end, b.end))
}

fn ranges_overlap(a: Range, b: Range) -> bool {
    compare_positions(a.start, b.end).is_lt() && compare_positions(b.start, a.end).is_lt()
}

fn compare_positions(
    a: tower_lsp::lsp_types::Position,
    b: tower_lsp::lsp_types::Position,
) -> Ordering {
    a.line.cmp(&b.line).then(a.character.cmp(&b.character))
}

fn annotatable_edit_range(edit: &OneOf<TextEdit, AnnotatedTextEdit>) -> Range {
    match edit {
        OneOf::Left(edit) => edit.range,
        OneOf::Right(edit) => edit.text_edit.range,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use tower_lsp::lsp_types::{Position, Range, TextEdit, Url, WorkspaceEdit};

    use super::merge_missing_authored_rename;
    use crate::{ide::IdeContext, server::ServerState};

    const SOURCE: &str = "<script setup lang=\"ts\">\nconst total = 1\n</script>\n<template>{{ total }}</template>\n";

    #[test]
    fn appends_missing_authored_ranges_after_canonical_rename() {
        let state = ServerState::new();
        let uri = Url::parse("file:///workspace/Scenario.vue").unwrap();
        state
            .documents
            .open(uri.clone(), SOURCE.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, SOURCE);
        let ctx = IdeContext::new(&state, &uri, SOURCE.find("total").unwrap()).unwrap();

        let script_edit = edit(1, 6, 1, 11);
        let template_edit = edit(3, 13, 3, 18);
        let canonical = changes(&uri, vec![template_edit.clone()]);
        let authored = changes(&uri, vec![script_edit.clone(), template_edit.clone()]);

        let merged = merge_missing_authored_rename(&ctx, Some(canonical), Some(authored))
            .expect("merged edit");
        let merged = merged.changes.expect("merged changes");

        assert_eq!(merged[&uri], vec![script_edit, template_edit]);
    }

    #[test]
    fn keeps_canonical_text_for_overlapping_authored_ranges() {
        let state = ServerState::new();
        let uri = Url::parse("file:///workspace/Scenario.vue").unwrap();
        state
            .documents
            .open(uri.clone(), SOURCE.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, SOURCE);
        let ctx = IdeContext::new(&state, &uri, SOURCE.find("total").unwrap()).unwrap();

        let mut canonical_edit = edit(1, 0, 1, 11);
        canonical_edit.new_text = "const quantity".to_string();
        let mut authored_edit = edit(1, 6, 1, 11);
        authored_edit.new_text = "renamed".to_string();

        let merged = merge_missing_authored_rename(
            &ctx,
            Some(changes(&uri, vec![canonical_edit.clone()])),
            Some(changes(&uri, vec![authored_edit])),
        )
        .expect("merged edit");
        let merged = merged.changes.expect("merged changes");

        assert_eq!(merged[&uri], vec![canonical_edit]);
    }

    #[test]
    fn keeps_multi_range_canonical_rename_authoritative() {
        let state = ServerState::new();
        let uri = Url::parse("file:///workspace/Scenario.vue").unwrap();
        state
            .documents
            .open(uri.clone(), SOURCE.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, SOURCE);
        let ctx = IdeContext::new(&state, &uri, SOURCE.find("total").unwrap()).unwrap();

        let canonical = changes(&uri, vec![edit(1, 6, 1, 11), edit(3, 13, 3, 18)]);
        let authored = changes(
            &uri,
            vec![edit(1, 6, 1, 11), edit(3, 13, 3, 18), edit(3, 36, 3, 41)],
        );

        let merged = merge_missing_authored_rename(&ctx, Some(canonical.clone()), Some(authored))
            .expect("merged edit");

        assert_eq!(merged, canonical);
    }

    fn changes(uri: &Url, edits: Vec<TextEdit>) -> WorkspaceEdit {
        WorkspaceEdit {
            changes: Some(HashMap::from([(uri.clone(), edits)])),
            document_changes: None,
            change_annotations: None,
        }
    }

    fn edit(start_line: u32, start_character: u32, end_line: u32, end_character: u32) -> TextEdit {
        TextEdit {
            range: Range::new(
                Position::new(start_line, start_character),
                Position::new(end_line, end_character),
            ),
            new_text: "quantity".to_string(),
        }
    }
}
