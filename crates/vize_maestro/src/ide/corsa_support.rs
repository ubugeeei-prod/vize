//! Shared Corsa helpers for mapping virtual document responses back to Vue SFCs.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use std::collections::HashMap;

use tower_lsp::lsp_types::{
    AnnotatedTextEdit, DocumentChangeOperation, DocumentChanges, Location, OneOf,
    PrepareRenameResponse, Range, TextEdit, Url, WorkspaceEdit,
};
use vize_canon::LspLocation;
use vize_carton::{cstr, String};

use super::IdeContext;
use crate::virtual_code::{SourceRange, VirtualDocument};

enum CurrentVirtualDocument<'a> {
    Template(&'a VirtualDocument),
    Script(&'a VirtualDocument),
    ScriptSetup(&'a VirtualDocument),
}

impl<'a> CurrentVirtualDocument<'a> {
    fn document(&self) -> &'a VirtualDocument {
        match self {
            Self::Template(doc) | Self::Script(doc) | Self::ScriptSetup(doc) => doc,
        }
    }
}

/// Build the virtual template request path used for Corsa queries.
pub(crate) fn template_request_path(uri: &Url) -> String {
    cstr!("{}.template.ts", uri.path())
}

/// Build the virtual script request path used for Corsa queries.
pub(crate) fn script_request_path(uri: &Url, is_setup: bool) -> String {
    if is_setup {
        cstr!("{}.setup.ts", uri.path())
    } else {
        cstr!("{}.script.ts", uri.path())
    }
}

/// Normalize a filesystem path into the `file://` form expected by Corsa.
pub(crate) fn request_file_uri(path: &str) -> String {
    if path.starts_with("file://") {
        String::from(path)
    } else {
        cstr!("file://{path}")
    }
}

/// Map a batch of Corsa locations back onto the current Vue document.
pub(crate) fn map_corsa_locations(
    ctx: &IdeContext<'_>,
    locations: Vec<LspLocation>,
) -> Vec<Location> {
    locations
        .iter()
        .filter_map(|location| map_corsa_location(ctx, location))
        .collect()
}

/// Map a single Corsa location back to either the Vue SFC or a real file URI.
pub(crate) fn map_corsa_location(ctx: &IdeContext<'_>, location: &LspLocation) -> Option<Location> {
    if let Some(current_doc) = match_current_virtual_document(ctx, &location.uri) {
        let range = map_virtual_range(
            ctx,
            current_doc.document(),
            &Range {
                start: tower_lsp::lsp_types::Position {
                    line: location.range.start.line,
                    character: location.range.start.character,
                },
                end: tower_lsp::lsp_types::Position {
                    line: location.range.end.line,
                    character: location.range.end.character,
                },
            },
        )?;

        return Some(Location {
            uri: ctx.uri.clone(),
            range,
        });
    }

    let uri = Url::parse(&location.uri).ok()?;
    Some(Location {
        uri,
        range: Range {
            start: tower_lsp::lsp_types::Position {
                line: location.range.start.line,
                character: location.range.start.character,
            },
            end: tower_lsp::lsp_types::Position {
                line: location.range.end.line,
                character: location.range.end.character,
            },
        },
    })
}

/// Translate a Corsa prepare-rename payload into SFC coordinates.
pub(crate) fn map_corsa_prepare_rename(
    ctx: &IdeContext<'_>,
    request_uri: &str,
    response: PrepareRenameResponse,
) -> Option<PrepareRenameResponse> {
    let current_doc = match_current_virtual_document(ctx, request_uri)?;

    match response {
        PrepareRenameResponse::Range(range) => {
            map_virtual_range(ctx, current_doc.document(), &range).map(PrepareRenameResponse::Range)
        }
        PrepareRenameResponse::RangeWithPlaceholder { range, placeholder } => {
            map_virtual_range(ctx, current_doc.document(), &range)
                .map(|range| PrepareRenameResponse::RangeWithPlaceholder { range, placeholder })
        }
        PrepareRenameResponse::DefaultBehavior { default_behavior } => {
            Some(PrepareRenameResponse::DefaultBehavior { default_behavior })
        }
    }
}

/// Rewrite a workspace edit so virtual-document edits target the Vue source.
pub(crate) fn map_corsa_workspace_edit(
    ctx: &IdeContext<'_>,
    mut edit: WorkspaceEdit,
) -> Option<WorkspaceEdit> {
    if let Some(changes) = edit.changes.take() {
        let mut mapped_changes = HashMap::with_capacity(changes.len());

        for (uri, edits) in changes {
            if let Some(current_doc) = match_current_virtual_document(ctx, uri.as_str()) {
                let entry = mapped_changes
                    .entry(ctx.uri.clone())
                    .or_insert_with(Vec::new);
                entry.extend(
                    edits
                        .into_iter()
                        .filter_map(|edit| map_text_edit(ctx, current_doc.document(), edit)),
                );
            } else {
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
    let current_doc = match_current_virtual_document(ctx, edit.text_document.uri.as_str());

    if let Some(current_doc) = current_doc {
        edit.text_document.uri = ctx.uri.clone();
        edit.edits = edit
            .edits
            .into_iter()
            .filter_map(|entry| match entry {
                OneOf::Left(text_edit) => {
                    map_text_edit(ctx, current_doc.document(), text_edit).map(OneOf::Left)
                }
                OneOf::Right(annotated) => {
                    map_annotated_text_edit(ctx, current_doc.document(), annotated)
                        .map(OneOf::Right)
                }
            })
            .collect();
    }

    if edit.edits.is_empty() {
        None
    } else {
        Some(edit)
    }
}

fn map_annotated_text_edit(
    ctx: &IdeContext<'_>,
    document: &VirtualDocument,
    mut edit: AnnotatedTextEdit,
) -> Option<AnnotatedTextEdit> {
    edit.text_edit = map_text_edit(ctx, document, edit.text_edit)?;
    Some(edit)
}

fn map_text_edit(
    ctx: &IdeContext<'_>,
    document: &VirtualDocument,
    mut edit: TextEdit,
) -> Option<TextEdit> {
    edit.range = map_virtual_range(ctx, document, &edit.range)?;
    Some(edit)
}

fn map_virtual_range(
    ctx: &IdeContext<'_>,
    document: &VirtualDocument,
    range: &Range,
) -> Option<Range> {
    let generated_start =
        super::position_to_offset(&document.content, range.start.line, range.start.character)?;
    let generated_end =
        super::position_to_offset(&document.content, range.end.line, range.end.character)?;

    let source_range = if generated_end > generated_start {
        document
            .source_map
            .generated_range_to_source(SourceRange::new(
                generated_start as u32,
                generated_end as u32,
            ))?
    } else {
        let source_offset = document.source_map.to_source(generated_start as u32)?;
        SourceRange::new(source_offset, source_offset)
    };

    let (start_line, start_character) =
        super::offset_to_position(&ctx.content, source_range.start as usize);
    let (end_line, end_character) =
        super::offset_to_position(&ctx.content, source_range.end as usize);

    Some(Range {
        start: tower_lsp::lsp_types::Position {
            line: start_line,
            character: start_character,
        },
        end: tower_lsp::lsp_types::Position {
            line: end_line,
            character: end_character,
        },
    })
}

fn match_current_virtual_document<'a>(
    ctx: &'a IdeContext<'_>,
    uri: &str,
) -> Option<CurrentVirtualDocument<'a>> {
    let path = virtual_document_path(uri)?;
    let virtual_docs = ctx.virtual_docs.as_ref()?;

    if path == template_request_path(ctx.uri).as_str() {
        return virtual_docs
            .template
            .as_ref()
            .map(CurrentVirtualDocument::Template);
    }

    if path == script_request_path(ctx.uri, false).as_str() {
        return virtual_docs
            .script
            .as_ref()
            .map(CurrentVirtualDocument::Script);
    }

    if path == script_request_path(ctx.uri, true).as_str() {
        return virtual_docs
            .script_setup
            .as_ref()
            .map(CurrentVirtualDocument::ScriptSetup);
    }

    None
}

fn virtual_document_path(uri: &str) -> Option<String> {
    if let Ok(parsed) = Url::parse(uri) {
        return Some(parsed.path().to_string().into());
    }

    if let Some(path) = uri.strip_prefix("vize-virtual://") {
        return Some(path.to_string().into());
    }

    None
}
