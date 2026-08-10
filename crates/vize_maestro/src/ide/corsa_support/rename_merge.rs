use std::cmp::Ordering;

use tower_lsp::lsp_types::{
    AnnotatedTextEdit, DocumentChangeOperation, DocumentChanges, OneOf,
    OptionalVersionedTextDocumentIdentifier, Range, TextDocumentEdit, TextEdit, WorkspaceEdit,
};

use crate::ide::IdeContext;

/// Combine the Corsa rename with the authored one for the SFC under the cursor.
///
/// Corsa renames the single virtual document the request opened, so a
/// script-side rename carries the script occurrences but never the template
/// ones that the same binding drives (nor the reverse). Folding the authored
/// edits back in keeps both blocks moving together, while Corsa keeps the
/// spans it already rewrote here plus every other file it touched.
pub(crate) fn merge_authored_rename(
    ctx: &IdeContext<'_>,
    corsa: Option<WorkspaceEdit>,
    authored: Option<WorkspaceEdit>,
) -> Option<WorkspaceEdit> {
    let merged = match (corsa, authored) {
        (Some(corsa), Some(authored)) => Some(merge_into_corsa_edit(ctx, corsa, authored)),
        (Some(corsa), None) => Some(corsa),
        (None, authored) => authored,
    };

    // Neither side answers in document order on its own: Corsa maps each block
    // back from its own virtual document, and the authored sweep only leads for
    // the block under the cursor. Ordering every per-file list here keeps a
    // template-side rename from reporting the template hit before the
    // declaration that sits above it.
    merged.map(order_edits_by_position)
}

/// Put every per-file edit list back in document order.
///
/// Rename answers are assembled block by block (or, on the canonical path,
/// query by query), so the edit lists arrive in whatever order the pieces were
/// stitched together. Clients and oracles read a workspace edit as a document,
/// so the list has to be sorted before it leaves the rename provider.
pub(crate) fn order_edits_by_position(mut edit: WorkspaceEdit) -> WorkspaceEdit {
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

fn merge_into_corsa_edit(
    ctx: &IdeContext<'_>,
    mut corsa: WorkspaceEdit,
    authored: WorkspaceEdit,
) -> WorkspaceEdit {
    let Some(authored_edits) = authored
        .changes
        .and_then(|mut changes| changes.remove(ctx.uri))
        .filter(|edits| !edits.is_empty())
    else {
        return corsa;
    };

    // Clients prefer `documentChanges` whenever it is present, so the authored
    // edits have to land in whichever container Corsa actually populated.
    match corsa.document_changes.as_mut() {
        Some(DocumentChanges::Edits(edits)) => {
            merge_into_document_edits(ctx, edits, authored_edits)
        }
        Some(DocumentChanges::Operations(operations)) => {
            merge_into_operations(ctx, operations, authored_edits);
        }
        None => {
            let entry = corsa
                .changes
                .get_or_insert_with(Default::default)
                .entry(ctx.uri.clone())
                .or_default();
            *entry = merge_text_edits(authored_edits, std::mem::take(entry));
        }
    }

    corsa
}

fn merge_into_document_edits(
    ctx: &IdeContext<'_>,
    edits: &mut Vec<TextDocumentEdit>,
    authored_edits: Vec<TextEdit>,
) {
    for edit in edits.iter_mut() {
        if edit.text_document.uri == *ctx.uri {
            edit.edits = merge_annotatable_edits(authored_edits, std::mem::take(&mut edit.edits));
            return;
        }
    }
    edits.push(authored_document_edit(ctx, authored_edits));
}

fn merge_into_operations(
    ctx: &IdeContext<'_>,
    operations: &mut Vec<DocumentChangeOperation>,
    authored_edits: Vec<TextEdit>,
) {
    for operation in operations.iter_mut() {
        if let DocumentChangeOperation::Edit(edit) = operation
            && edit.text_document.uri == *ctx.uri
        {
            edit.edits = merge_annotatable_edits(authored_edits, std::mem::take(&mut edit.edits));
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

/// Authored edits lead, and a Corsa edit is kept only when it rewrites a span
/// the authored rename missed, so a shared occurrence is never edited twice.
/// `order_edits_by_position` puts the survivors back in document order.
fn merge_text_edits(authored: Vec<TextEdit>, corsa: Vec<TextEdit>) -> Vec<TextEdit> {
    let mut merged = authored;
    for edit in corsa {
        if !merged.iter().any(|kept| kept.range == edit.range) {
            merged.push(edit);
        }
    }
    merged
}

fn merge_annotatable_edits(
    authored: Vec<TextEdit>,
    corsa: Vec<OneOf<TextEdit, AnnotatedTextEdit>>,
) -> Vec<OneOf<TextEdit, AnnotatedTextEdit>> {
    let mut merged = authored
        .into_iter()
        .map(OneOf::Left)
        .collect::<Vec<OneOf<TextEdit, AnnotatedTextEdit>>>();
    for entry in corsa {
        let range = annotatable_edit_range(&entry);
        if !merged
            .iter()
            .any(|kept| annotatable_edit_range(kept) == range)
        {
            merged.push(entry);
        }
    }
    merged
}

fn compare_ranges(a: &Range, b: &Range) -> Ordering {
    a.start
        .line
        .cmp(&b.start.line)
        .then(a.start.character.cmp(&b.start.character))
        .then(a.end.line.cmp(&b.end.line))
        .then(a.end.character.cmp(&b.end.character))
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

    use tower_lsp::lsp_types::{
        DocumentChanges, OneOf, OptionalVersionedTextDocumentIdentifier, Position, Range,
        TextDocumentEdit, TextEdit, Url, WorkspaceEdit,
    };

    use super::merge_authored_rename;
    use crate::{ide::IdeContext, server::ServerState};

    const SOURCE: &str = "<script setup lang=\"ts\">\nconst total = 1\n</script>\n<template>{{ total }}</template>\n";

    #[test]
    fn keeps_template_occurrences_corsa_never_saw() {
        let state = ServerState::new();
        let uri = Url::parse("file:///workspace/Scenario.vue").unwrap();
        state
            .documents
            .open(uri.clone(), SOURCE.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, SOURCE);
        let ctx = IdeContext::new(&state, &uri, SOURCE.find("total").unwrap()).unwrap();

        let script_edit = edit(1, 6, 1, 11);
        let template_edit = edit(3, 13, 3, 18);
        let corsa = changes(&uri, vec![script_edit.clone()]);
        let authored = changes(&uri, vec![script_edit.clone(), template_edit.clone()]);

        let merged = merge_authored_rename(&ctx, Some(corsa), Some(authored)).expect("merged edit");
        let merged = merged.changes.expect("merged changes");

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[&uri], vec![script_edit, template_edit]);
    }

    #[test]
    fn orders_merged_edits_by_position() {
        let state = ServerState::new();
        let uri = Url::parse("file:///workspace/Scenario.vue").unwrap();
        state
            .documents
            .open(uri.clone(), SOURCE.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, SOURCE);
        let ctx = IdeContext::new(&state, &uri, SOURCE.find("total").unwrap()).unwrap();

        // A template-side rename answers with the template hit, and only Corsa
        // knows about the declaration that sits above it.
        let script_edit = edit(1, 6, 1, 11);
        let template_edit = edit(3, 13, 3, 18);
        let corsa = changes(&uri, vec![script_edit.clone()]);
        let authored = changes(&uri, vec![template_edit.clone()]);

        let merged = merge_authored_rename(&ctx, Some(corsa), Some(authored)).expect("merged edit");
        let merged = merged.changes.expect("merged changes");

        assert_eq!(
            merged[&uri],
            vec![script_edit.clone(), template_edit.clone()]
        );

        // Corsa maps each block back from its own virtual document, so it can
        // answer for the whole SFC in block order rather than document order.
        let corsa_only = changes(&uri, vec![template_edit.clone(), script_edit.clone()]);
        let merged =
            merge_authored_rename(&ctx, Some(corsa_only), None).expect("corsa-only edit ordered");
        let merged = merged.changes.expect("corsa-only changes");

        assert_eq!(merged[&uri], vec![script_edit, template_edit]);
    }

    #[test]
    fn merges_into_document_changes_when_corsa_uses_them() {
        let state = ServerState::new();
        let uri = Url::parse("file:///workspace/Scenario.vue").unwrap();
        state
            .documents
            .open(uri.clone(), SOURCE.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, SOURCE);
        let ctx = IdeContext::new(&state, &uri, SOURCE.find("total").unwrap()).unwrap();

        let script_edit = edit(1, 6, 1, 11);
        let template_edit = edit(3, 13, 3, 18);
        let corsa = WorkspaceEdit {
            changes: None,
            document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                text_document: OptionalVersionedTextDocumentIdentifier {
                    uri: uri.clone(),
                    version: None,
                },
                edits: vec![OneOf::Left(script_edit.clone())],
            }])),
            change_annotations: None,
        };
        let authored = changes(&uri, vec![script_edit.clone(), template_edit.clone()]);

        let merged = merge_authored_rename(&ctx, Some(corsa), Some(authored)).expect("merged edit");
        let Some(DocumentChanges::Edits(document_edits)) = merged.document_changes else {
            panic!("expected merged document changes");
        };

        assert!(merged.changes.is_none(), "authored edits must not fork");
        assert_eq!(document_edits.len(), 1);
        assert_eq!(
            document_edits[0].edits,
            vec![OneOf::Left(script_edit), OneOf::Left(template_edit)]
        );
    }

    #[test]
    fn falls_back_to_either_side_alone() {
        let state = ServerState::new();
        let uri = Url::parse("file:///workspace/Scenario.vue").unwrap();
        state
            .documents
            .open(uri.clone(), SOURCE.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, SOURCE);
        let ctx = IdeContext::new(&state, &uri, SOURCE.find("total").unwrap()).unwrap();

        let authored = changes(&uri, vec![edit(1, 6, 1, 11)]);
        assert_eq!(
            merge_authored_rename(&ctx, None, Some(authored.clone())),
            Some(authored.clone())
        );
        assert_eq!(
            merge_authored_rename(&ctx, Some(authored.clone()), None),
            Some(authored)
        );
        assert_eq!(merge_authored_rename(&ctx, None, None), None);
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
