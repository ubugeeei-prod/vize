//! Host binding for the actual source-owned native edit model.

use tower_lsp::lsp_types::{
    AnnotatedTextEdit, DocumentChanges, OneOf, OptionalVersionedTextDocumentIdentifier, Position,
    Range, TextDocumentEdit, TextEdit, WorkspaceEdit,
};
use vize_l0::line_index::offset_to_line_col;
use vize_l0::{SourceFrameError, SourceRoot, String};
use vize_l1::edit::{EditError, EditSet, VersionedSource};

use crate::document::DocumentStore;

use super::{SnapshotKey, SnapshotRefusal, SourceSnapshot};

/// The host or original source-owned edit model refused to produce new text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceEditRefusal {
    Host(SnapshotRefusal),
    Frame(SourceFrameError),
    Edit(EditError),
}

impl SourceSnapshot {
    /// Borrow the once-captured physical buffer, not an equal-text reread.
    /// This frame is source/edit evidence only, never native File admission.
    pub fn versioned_source(
        &self,
    ) -> Result<VersionedSource<'_, SnapshotKey<'_>>, SourceFrameError> {
        Ok(VersionedSource::new(
            self.key(),
            self.version(),
            SourceRoot::new(self.source())?,
        ))
    }

    /// Validate against the actual current host under its read guard, then
    /// produce a new authored string. This does not mutate DocumentStore or
    /// claim an atomic editor application. Publish via SourceQueryResult to
    /// recheck cancellation and the host after intervening work.
    pub fn apply_edits(
        &self,
        documents: &DocumentStore,
        edits: EditSet<'_, '_, '_, SnapshotKey<'_>>,
    ) -> Result<String, SourceEditRefusal> {
        self.with_current(documents, |document| {
            let current = VersionedSource::new(
                SnapshotKey {
                    uri: &document.uri,
                    revision: document.revision(),
                },
                document.version,
                SourceRoot::new(self.source()).map_err(SourceEditRefusal::Frame)?,
            );
            edits.apply(current).map_err(SourceEditRefusal::Edit)
        })
        .map_err(SourceEditRefusal::Host)?
    }

    /// Publish checked authored edits as one versioned LSP document change.
    ///
    /// The current host identity, revision, client version and physical source
    /// are rechecked under the document read guard before any edit payload is
    /// emitted. The returned ranges are UTF-16 LSP coordinates derived from
    /// the retained authored buffer; the source remains untouched.
    pub fn workspace_edit(
        &self,
        documents: &DocumentStore,
        edits: EditSet<'_, '_, '_, SnapshotKey<'_>>,
    ) -> Result<WorkspaceEdit, SourceEditRefusal> {
        self.with_current(documents, |document| {
            let current = VersionedSource::new(
                SnapshotKey {
                    uri: &document.uri,
                    revision: document.revision(),
                },
                document.version,
                SourceRoot::new(self.source()).map_err(SourceEditRefusal::Frame)?,
            );
            edits
                .check_current(current)
                .map_err(SourceEditRefusal::Edit)?;

            let text_edits = edits
                .edits()
                .iter()
                .map(|edit| {
                    let span = edit.span();
                    let (start_line, start_character) =
                        offset_to_line_col(self.source(), span.start as usize);
                    let (end_line, end_character) =
                        offset_to_line_col(self.source(), span.end as usize);
                    OneOf::Left(TextEdit {
                        range: Range::new(
                            Position::new(start_line, start_character),
                            Position::new(end_line, end_character),
                        ),
                        new_text: edit.replacement().into(),
                    })
                })
                .collect();

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

#[cfg(feature = "experimental-versioned-fixes")]
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
            let edits = edits
                .positioned_edits(current)
                .map_err(SourceEditRefusal::Edit)?;
            let mut text_edits = Vec::with_capacity(edits.len());
            for (range, replacement) in edits {
                text_edits.push(OneOf::Left(TextEdit {
                    range: Range::new(
                        Position::new(range.start.line, range.start.character),
                        Position::new(range.end.line, range.end.character),
                    ),
                    new_text: replacement.into(),
                }));
            }
            Ok(self.document_change(text_edits))
        })
        .map_err(SourceEditRefusal::Host)?
    }
}
