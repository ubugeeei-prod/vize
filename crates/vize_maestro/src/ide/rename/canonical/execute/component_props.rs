use tower_lsp::lsp_types::{DocumentChangeOperation, DocumentChanges, OneOf, Range, WorkspaceEdit};
use vize_canon::LspLocation;
use vize_carton::{FxHashSet, String};

use crate::ide::{IdeContext, corsa_support};

pub(super) fn retain_component_prop_edits(
    ctx: &IdeContext<'_>,
    document: &corsa_support::CanonicalVirtualDocument,
    edit: &mut WorkspaceEdit,
    names: &FxHashSet<String>,
) {
    if let Some(changes) = edit.changes.as_mut() {
        changes.retain(|uri, edits| {
            edits.retain(|edit| component_prop_edit_matches(ctx, document, uri, edit.range, names));
            !edits.is_empty()
        });
    }
    if let Some(document_changes) = edit.document_changes.as_mut() {
        match document_changes {
            DocumentChanges::Edits(edits) => {
                for edit in edits.iter_mut() {
                    edit.edits.retain(|entry| {
                        component_prop_edit_matches(
                            ctx,
                            document,
                            &edit.text_document.uri,
                            annotatable_range(entry),
                            names,
                        )
                    });
                }
                edits.retain(|edit| !edit.edits.is_empty());
            }
            DocumentChanges::Operations(operations) => {
                for operation in operations.iter_mut() {
                    if let DocumentChangeOperation::Edit(edit) = operation {
                        edit.edits.retain(|entry| {
                            component_prop_edit_matches(
                                ctx,
                                document,
                                &edit.text_document.uri,
                                annotatable_range(entry),
                                names,
                            )
                        });
                    }
                }
                operations.retain(|operation| match operation {
                    DocumentChangeOperation::Edit(edit) => !edit.edits.is_empty(),
                    DocumentChangeOperation::Op(_) => true,
                });
            }
        }
    }
}

fn component_prop_edit_matches(
    ctx: &IdeContext<'_>,
    document: &corsa_support::CanonicalVirtualDocument,
    uri: &tower_lsp::lsp_types::Url,
    range: Range,
    names: &FxHashSet<String>,
) -> bool {
    let raw = LspLocation {
        uri: uri.to_string(),
        range: corsa_support::tower_range(range),
    };
    let Some(authored) = corsa_support::map_canonical_corsa_location(ctx, document, &raw) else {
        return false;
    };
    corsa_support::component_prop_location_matches(ctx, document, &authored, names)
}

fn annotatable_range(
    edit: &OneOf<tower_lsp::lsp_types::TextEdit, tower_lsp::lsp_types::AnnotatedTextEdit>,
) -> Range {
    match edit {
        OneOf::Left(edit) => edit.range,
        OneOf::Right(edit) => edit.text_edit.range,
    }
}
