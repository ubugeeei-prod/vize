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
#[path = "rename_missing_tests.rs"]
mod tests;
