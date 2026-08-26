use std::ops::Range as OffsetRange;

use tower_lsp::lsp_types::{
    DocumentChangeOperation, DocumentChanges, OneOf, TextEdit, Url, WorkspaceEdit,
};
use vize_s0::cstr;

use super::{RenameKind, component_event_ranges, model, offset_range};
use crate::ide::{IdeContext, pascal_to_kebab};

pub(super) fn edits(
    ctx: &IdeContext<'_>,
    edit: &mut WorkspaceEdit,
    semantic_name: &str,
    kind: RenameKind,
) {
    if let Some(changes) = edit.changes.as_mut() {
        for (uri, edits) in changes {
            rewrite_text_edits(ctx, uri, edits, semantic_name, kind);
        }
    }
    if let Some(changes) = edit.document_changes.as_mut() {
        match changes {
            DocumentChanges::Edits(edits) => {
                for edit in edits {
                    rewrite_annotatable_edits(
                        ctx,
                        &edit.text_document.uri,
                        &mut edit.edits,
                        semantic_name,
                        kind,
                    );
                }
            }
            DocumentChanges::Operations(operations) => {
                for operation in operations {
                    if let DocumentChangeOperation::Edit(edit) = operation {
                        rewrite_annotatable_edits(
                            ctx,
                            &edit.text_document.uri,
                            &mut edit.edits,
                            semantic_name,
                            kind,
                        );
                    }
                }
            }
        }
    }
}

fn rewrite_annotatable_edits(
    ctx: &IdeContext<'_>,
    uri: &Url,
    edits: &mut [OneOf<TextEdit, tower_lsp::lsp_types::AnnotatedTextEdit>],
    semantic_name: &str,
    kind: RenameKind,
) {
    let Some(source) = source_for_uri(ctx, uri) else {
        return;
    };
    let ranges = RewriteRanges::new(&source, uri.path(), kind);
    for edit in edits {
        match edit {
            OneOf::Left(edit) => ranges.rewrite(&source, edit, semantic_name),
            OneOf::Right(edit) => ranges.rewrite(&source, &mut edit.text_edit, semantic_name),
        }
    }
}

fn rewrite_text_edits(
    ctx: &IdeContext<'_>,
    uri: &Url,
    edits: &mut [TextEdit],
    semantic_name: &str,
    kind: RenameKind,
) {
    let Some(source) = source_for_uri(ctx, uri) else {
        return;
    };
    let ranges = RewriteRanges::new(&source, uri.path(), kind);
    for edit in edits {
        ranges.rewrite(&source, edit, semantic_name);
    }
}

struct RewriteRanges {
    events: Vec<OffsetRange<usize>>,
    model_usages: Vec<OffsetRange<usize>>,
    model_declarations: Vec<OffsetRange<usize>>,
}

impl RewriteRanges {
    fn new(source: &str, filename: &str, kind: RenameKind) -> Self {
        let is_model = matches!(kind, RenameKind::Model);
        let (model_usages, model_declarations) = if is_model {
            (
                model::usage_ranges(source, filename),
                model::declaration_ranges(source, filename),
            )
        } else {
            (Vec::new(), Vec::new())
        };
        Self {
            events: component_event_ranges(source, filename),
            model_usages,
            model_declarations,
        }
    }

    fn rewrite(&self, source: &str, edit: &mut TextEdit, semantic_name: &str) {
        let Some(start) = crate::ide::position_to_offset(
            source,
            edit.range.start.line,
            edit.range.start.character,
        ) else {
            return;
        };
        if let Some(range) = containing(&self.events, start) {
            edit.range = offset_range(source, range.clone());
            let prefix = source
                .get(range.clone())
                .filter(|name| name.starts_with("update:"))
                .map_or("", |_| "update:");
            edit.new_text = cstr!("{prefix}{}", pascal_to_kebab(semantic_name)).into();
        } else if let Some(range) = containing(&self.model_usages, start) {
            edit.range = offset_range(source, range.clone());
            edit.new_text = pascal_to_kebab(semantic_name);
        } else if let Some(range) = self
            .model_declarations
            .iter()
            .find(|range| start >= range.start && start <= range.end)
        {
            edit.range = offset_range(source, range.clone());
            edit.new_text = semantic_name.to_string();
        }
    }
}

fn containing(ranges: &[OffsetRange<usize>], offset: usize) -> Option<&OffsetRange<usize>> {
    ranges
        .iter()
        .find(|range| offset >= range.start && offset < range.end)
}

fn source_for_uri(ctx: &IdeContext<'_>, uri: &Url) -> Option<String> {
    ctx.state
        .documents
        .text(uri)
        .map(|source| source.to_string())
        .or_else(|| {
            uri.to_file_path()
                .ok()
                .and_then(|path| std::fs::read_to_string(path).ok())
        })
}
