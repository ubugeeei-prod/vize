//! Version-checked publication of source-ordered diagnostic fixes.
use super::{SnapshotKey, SourceEditRefusal, SourceSnapshot};
use crate::document::DocumentStore;
use tower_lsp::lsp_types::{
    AnnotatedTextEdit, DocumentChanges, OneOf, OptionalVersionedTextDocumentIdentifier, Position,
    Range, TextDocumentEdit, TextDocumentIdentifier, TextEdit, WorkspaceEdit,
};
use vize_l0::SourceRoot;
use vize_l1::edit::{EditError, EditSet, VersionedSource};

mod line_endings;

impl PartialEq for SnapshotKey<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.revision == other.revision
            && (core::ptr::eq(self.uri, other.uri) || self.uri == other.uri)
    }
}

impl SourceSnapshot {
    /// Publish the same selection the CLI and Node bindings can apply. The
    /// edit set belongs to its original file, revision, version and buffer.
    /// Move the matching request document into the resulting LSP change.
    pub fn workspace_edit_for_fixes(
        &self,
        documents: &DocumentStore,
        document: TextDocumentIdentifier,
        edits: EditSet<'_, '_, '_, SnapshotKey<'_>>,
    ) -> Result<WorkspaceEdit, SourceEditRefusal> {
        self.with_current(documents, |_| {
            if document.uri != *self.uri() {
                return Err(SourceEditRefusal::Edit(EditError::SnapshotMismatch));
            }
            let source = self.source();
            let current = VersionedSource::new(
                self.key(),
                self.version(),
                SourceRoot::new(source).map_err(SourceEditRefusal::Frame)?,
            );
            let checked = edits
                .positioned_edits(current)
                .map_err(SourceEditRefusal::Edit)?;
            let wire = |range: vize_l0::line_index::Range, replacement: &str| {
                OneOf::Left(TextEdit {
                    range: Range::new(
                        Position::new(range.start.line, range.start.character),
                        Position::new(range.end.line, range.end.character),
                    ),
                    new_text: replacement.into(),
                })
            };
            if let [only] = checked.edits()
                && only.span().end <= 256
            {
                if let Some(range) = line_endings::single_range(source, only.span()) {
                    return Ok(
                        self.document_change(document, vec![wire(range, only.replacement())])
                    );
                }
                return Ok(
                    self.document_change(document, line_endings::project(source, checked.edits()))
                );
            }
            if line_endings::needs_projection(source, checked.edits()) {
                return Ok(
                    self.document_change(document, line_endings::project(source, checked.edits()))
                );
            }
            if let Some((range, replacement)) = checked.single() {
                return Ok(self.document_change(document, vec![wire(range, replacement)]));
            }
            let edits = checked.ranges();
            let mut text_edits = Vec::with_capacity(edits.len());
            for (range, replacement) in edits {
                text_edits.push(wire(range, replacement));
            }
            Ok(self.document_change(document, text_edits))
        })
        .map_err(SourceEditRefusal::Host)?
    }

    #[inline]
    fn document_change(
        &self,
        document: TextDocumentIdentifier,
        text_edits: Vec<OneOf<TextEdit, AnnotatedTextEdit>>,
    ) -> WorkspaceEdit {
        WorkspaceEdit {
            changes: None,
            document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                text_document: OptionalVersionedTextDocumentIdentifier {
                    uri: document.uri,
                    version: Some(self.version()),
                },
                edits: text_edits,
            }])),
            change_annotations: None,
        }
    }
}
