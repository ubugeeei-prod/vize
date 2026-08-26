use std::collections::HashMap;

use tower_lsp::lsp_types::{
    AnnotatedTextEdit, DocumentChangeOperation, DocumentChanges, OneOf, TextEdit, WorkspaceEdit,
};

use crate::ide::IdeContext;

pub(crate) fn map_corsa_workspace_edit(
    ctx: &IdeContext<'_>,
    mut edit: WorkspaceEdit,
) -> Option<WorkspaceEdit> {
    if let Some(changes) = edit.changes.take() {
        let mut mapped_changes = HashMap::with_capacity(changes.len());

        for (uri, edits) in changes {
            if let Some(target) = super::match_virtual_document(ctx, uri.as_str()) {
                let entry = mapped_changes
                    .entry(target.uri().clone())
                    .or_insert_with(Vec::new);
                entry.extend(
                    edits
                        .into_iter()
                        .filter_map(|edit| map_text_edit(&target, edit)),
                );
            } else if !super::is_virtual_document_uri(uri.as_str()) {
                mapped_changes.insert(uri, edits);
            }
        }

        if !mapped_changes.is_empty() {
            edit.changes = Some(mapped_changes);
        }
    }

    if let Some(document_changes) = edit.document_changes.take() {
        let mapped_document_changes = match document_changes {
            DocumentChanges::Edits(edits) => {
                let edits = edits
                    .into_iter()
                    .filter_map(|edit| map_document_edit(ctx, edit))
                    .collect::<Vec<_>>();

                if edits.is_empty() {
                    None
                } else {
                    Some(DocumentChanges::Edits(edits))
                }
            }
            DocumentChanges::Operations(operations) => {
                let operations = operations
                    .into_iter()
                    .filter_map(|operation| map_document_change_operation(ctx, operation))
                    .collect::<Vec<_>>();

                if operations.is_empty() {
                    None
                } else {
                    Some(DocumentChanges::Operations(operations))
                }
            }
        };

        if let Some(document_changes) = mapped_document_changes {
            edit.document_changes = Some(document_changes);
        }
    }

    if workspace_edit_is_empty(&edit) {
        None
    } else {
        Some(edit)
    }
}

fn workspace_edit_is_empty(edit: &WorkspaceEdit) -> bool {
    let changes_empty = edit
        .changes
        .as_ref()
        .is_none_or(|changes| changes.values().all(Vec::is_empty));
    let document_changes_empty =
        edit.document_changes
            .as_ref()
            .is_none_or(|changes| match changes {
                DocumentChanges::Edits(edits) => edits.is_empty(),
                DocumentChanges::Operations(operations) => operations.is_empty(),
            });

    changes_empty && document_changes_empty
}

fn map_document_change_operation(
    ctx: &IdeContext<'_>,
    operation: DocumentChangeOperation,
) -> Option<DocumentChangeOperation> {
    match operation {
        DocumentChangeOperation::Edit(edit) => {
            map_document_edit(ctx, edit).map(DocumentChangeOperation::Edit)
        }
        DocumentChangeOperation::Op(op) => Some(DocumentChangeOperation::Op(op)),
    }
}

fn map_document_edit(
    ctx: &IdeContext<'_>,
    mut edit: tower_lsp::lsp_types::TextDocumentEdit,
) -> Option<tower_lsp::lsp_types::TextDocumentEdit> {
    let target = super::match_virtual_document(ctx, edit.text_document.uri.as_str());

    if let Some(target) = target {
        edit.text_document.uri = target.uri().clone();
        edit.edits = edit
            .edits
            .into_iter()
            .filter_map(|entry| match entry {
                OneOf::Left(text_edit) => map_text_edit(&target, text_edit).map(OneOf::Left),
                OneOf::Right(annotated) => {
                    map_annotated_text_edit(&target, annotated).map(OneOf::Right)
                }
            })
            .collect();
    } else if super::is_virtual_document_uri(edit.text_document.uri.as_str()) {
        return None;
    }

    if edit.edits.is_empty() {
        None
    } else {
        Some(edit)
    }
}

fn map_annotated_text_edit(
    target: &super::MatchedVirtualDocument<'_>,
    mut edit: AnnotatedTextEdit,
) -> Option<AnnotatedTextEdit> {
    edit.text_edit = map_text_edit(target, edit.text_edit)?;
    Some(edit)
}

fn map_text_edit(
    target: &super::MatchedVirtualDocument<'_>,
    mut edit: TextEdit,
) -> Option<TextEdit> {
    edit.range =
        super::map_virtual_range_for_content(target.content(), target.document()?, &edit.range)?;
    Some(edit)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use tower_lsp::lsp_types::{
        DocumentChanges, OneOf, OptionalVersionedTextDocumentIdentifier, Position, Range,
        TextDocumentEdit, TextEdit, Url, WorkspaceEdit,
    };
    use vize_canon::{LspLocation, LspPosition, LspRange};
    use vize_s0::cstr;

    use super::map_corsa_workspace_edit;
    use crate::{
        ide::{IdeContext, corsa_support, offset_to_position},
        server::ServerState,
    };

    #[test]
    fn maps_cross_sfc_virtual_edits_and_references_to_authored_documents() {
        let state = ServerState::new();
        let child_uri = Url::parse("file:///workspace/Child%20View.vue").unwrap();
        let parent_uri = Url::parse("file:///workspace/Parent%20View.vue").unwrap();
        let child_source = sfc("child");
        let parent_source = sfc("parent");
        for (uri, source) in [
            (&child_uri, child_source.as_str()),
            (&parent_uri, parent_source.as_str()),
        ] {
            state
                .documents
                .open(uri.clone(), source.to_string(), 1, "vue".to_string());
            state.update_virtual_docs(uri, source);
        }

        let child_docs = state.get_virtual_docs(&child_uri).unwrap();
        let parent_docs = state.get_virtual_docs(&parent_uri).unwrap();
        let child_virtual = child_docs.template.as_ref().unwrap();
        let parent_virtual = parent_docs.template.as_ref().unwrap();
        let child_edit = virtual_edit(child_source.as_str(), child_virtual, "renamed");
        let parent_edit = virtual_edit(parent_source.as_str(), parent_virtual, "renamed");
        let child_request_uri = request_uri(&child_uri);
        let parent_request_uri = request_uri(&parent_uri);

        let mut changes = HashMap::new();
        changes.insert(child_request_uri.clone(), vec![child_edit]);
        changes.insert(parent_request_uri.clone(), vec![parent_edit.clone()]);
        let ctx =
            IdeContext::new(&state, &child_uri, child_source.find("message").unwrap()).unwrap();
        let mapped = map_corsa_workspace_edit(
            &ctx,
            WorkspaceEdit {
                changes: Some(changes),
                document_changes: None,
                change_annotations: None,
            },
        )
        .expect("authored workspace edit");
        let changes = mapped.changes.expect("authored changes");

        assert_eq!(changes.len(), 2, "synthetic changes leaked: {changes:#?}");
        for (uri, source) in [(&child_uri, &child_source), (&parent_uri, &parent_source)] {
            let edits = changes.get(uri).expect("authored SFC edit");
            assert_eq!(edits.len(), 1);
            assert_eq!(edits[0].new_text, "renamed");
            assert_eq!(authored_text(source, edits[0].range), "message");
        }
        assert!(!changes.contains_key(&child_request_uri));
        assert!(!changes.contains_key(&parent_request_uri));

        let mapped_document_changes = map_corsa_workspace_edit(
            &ctx,
            WorkspaceEdit {
                changes: None,
                document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                    text_document: OptionalVersionedTextDocumentIdentifier {
                        uri: parent_request_uri.clone(),
                        version: None,
                    },
                    edits: vec![OneOf::Left(parent_edit.clone())],
                }])),
                change_annotations: None,
            },
        )
        .expect("authored document changes");
        let Some(DocumentChanges::Edits(document_edits)) = mapped_document_changes.document_changes
        else {
            panic!("expected mapped text document edits");
        };
        assert_eq!(document_edits.len(), 1);
        assert_eq!(document_edits[0].text_document.uri, parent_uri);
        let OneOf::Left(edit) = &document_edits[0].edits[0] else {
            panic!("expected plain text edit");
        };
        assert_eq!(authored_text(&parent_source, edit.range), "message");

        let missing_virtual_uri = Url::parse("file:///workspace/Missing.vue.template.ts").unwrap();
        assert!(
            map_corsa_workspace_edit(
                &ctx,
                WorkspaceEdit {
                    changes: Some(HashMap::from([(
                        missing_virtual_uri.clone(),
                        vec![parent_edit.clone()],
                    )])),
                    document_changes: None,
                    change_annotations: None,
                },
            )
            .is_none(),
            "unmappable synthetic edits must not leak to the client"
        );

        let parent_reference = LspLocation {
            uri: parent_request_uri.to_string(),
            range: LspRange {
                start: LspPosition {
                    line: parent_edit.range.start.line,
                    character: parent_edit.range.start.character,
                },
                end: LspPosition {
                    line: parent_edit.range.end.line,
                    character: parent_edit.range.end.character,
                },
            },
        };
        let missing_reference = LspLocation {
            uri: missing_virtual_uri.to_string(),
            range: parent_reference.range.clone(),
        };
        let locations =
            corsa_support::map_corsa_locations(&ctx, vec![parent_reference, missing_reference]);
        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0].uri, parent_uri);
        assert_eq!(authored_text(&parent_source, locations[0].range), "message");
    }

    fn sfc(value: &str) -> String {
        cstr!(
            "<script setup lang=\"ts\">\nconst message = '{value}'\n</script>\n<template>😀 {{{{ message }}}}</template>\n"
        )
        .into()
    }

    fn request_uri(uri: &Url) -> Url {
        Url::parse(
            corsa_support::request_file_uri(&corsa_support::template_request_path(uri)).as_str(),
        )
        .unwrap()
    }

    fn virtual_edit(
        source: &str,
        document: &crate::virtual_code::VirtualDocument,
        new_text: &str,
    ) -> TextEdit {
        let source_start = source.rfind("message").unwrap() as u32;
        let source_start = source_start.saturating_sub(document.source_map.block_offset);
        let generated_start = document
            .source_map
            .to_generated_for(source_start, |features| features.rename)
            .expect("generated identifier") as usize;
        let generated_end = generated_start + "message".len();
        let (start_line, start_character) = offset_to_position(&document.content, generated_start);
        let (end_line, end_character) = offset_to_position(&document.content, generated_end);
        TextEdit {
            range: Range::new(
                Position::new(start_line, start_character),
                Position::new(end_line, end_character),
            ),
            new_text: new_text.to_string(),
        }
    }

    fn authored_text(source: &str, range: Range) -> &str {
        let start = crate::ide::position_to_offset(source, range.start.line, range.start.character)
            .unwrap();
        let end =
            crate::ide::position_to_offset(source, range.end.line, range.end.character).unwrap();
        &source[start..end]
    }
}
