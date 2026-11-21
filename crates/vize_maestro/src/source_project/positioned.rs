//! Version-checked publication of source-ordered diagnostic fixes.
use super::{SnapshotKey, SourceEditRefusal, SourceSnapshot};
use crate::document::DocumentStore;
use tower_lsp::lsp_types::{
    AnnotatedTextEdit, DocumentChanges, OneOf, OptionalVersionedTextDocumentIdentifier, Position,
    Range, TextDocumentEdit, TextEdit, WorkspaceEdit,
};
use vize_l1::edit::EditSet;

impl SourceSnapshot {
    /// Publish the same selection the CLI and Node bindings can apply. The
    /// edit set belongs to its original file, revision, version and buffer.
    pub fn workspace_edit_for_fixes(
        &self,
        documents: &DocumentStore,
        edits: EditSet<'_, '_, '_, SnapshotKey<'_>>,
    ) -> Result<WorkspaceEdit, SourceEditRefusal> {
        self.with_current(documents, |_| {
            let current = self.versioned_source().map_err(SourceEditRefusal::Frame)?;
            let wire = |range: vize_l0::line_index::Range, replacement: &str| {
                OneOf::Left(TextEdit {
                    range: Range::new(
                        Position::new(range.start.line, range.start.character),
                        Position::new(range.end.line, range.end.character),
                    ),
                    new_text: replacement.into(),
                })
            };
            if let [only] = edits.edits() {
                let range = only
                    .positioned_range(current)
                    .map_err(SourceEditRefusal::Edit)?;
                return Ok(self.document_change(vec![wire(range, only.replacement())]));
            }
            let edits = edits
                .positioned_edits(current)
                .map_err(SourceEditRefusal::Edit)?;
            let mut text_edits = Vec::with_capacity(edits.len());
            for (range, replacement) in edits {
                text_edits.push(wire(range, replacement));
            }
            Ok(self.document_change(text_edits))
        })
        .map_err(SourceEditRefusal::Host)?
    }

    fn document_change(
        &self,
        text_edits: Vec<OneOf<TextEdit, AnnotatedTextEdit>>,
    ) -> WorkspaceEdit {
        WorkspaceEdit {
            changes: None,
            document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                text_document: OptionalVersionedTextDocumentIdentifier {
                    uri: self.uri().clone(),
                    version: Some(self.version()),
                },
                edits: text_edits,
            }])),
            change_annotations: None,
        }
    }
}
