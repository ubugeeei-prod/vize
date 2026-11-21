//! Version-checked publication of source-ordered diagnostic fixes.
use super::{SnapshotKey, SourceEditRefusal, SourceSnapshot};
use crate::document::DocumentStore;
use tower_lsp::lsp_types::{
    AnnotatedTextEdit, DocumentChanges, OneOf, OptionalVersionedTextDocumentIdentifier, Position,
    Range, TextDocumentEdit, TextEdit, WorkspaceEdit,
};
use vize_l0::SourceRoot;
use vize_l1::edit::{EditSet, VersionedSource};

mod line_endings;

impl SourceSnapshot {
    /// Publish the same selection the CLI and Node bindings can apply. The
    /// edit set belongs to its original file, revision, version and buffer.
    pub fn workspace_edit_for_fixes(
        &self,
        documents: &DocumentStore,
        edits: EditSet<'_, '_, '_, SnapshotKey<'_>>,
    ) -> Result<WorkspaceEdit, SourceEditRefusal> {
        self.with_current(documents, |_| {
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
            let raw = checked.edits();
            if let Some(last) = raw.last() {
                let end = last.span().end as usize;
                if end <= 1
                    && source.as_bytes().first() != Some(&b'\r')
                    && let Some((range, replacement)) = checked.single()
                {
                    return Ok(self.document_change(vec![wire(range, replacement)]));
                }
                // Fixed-size masks classify CR, non-ASCII and all line starts
                // together on x86-64. Other targets use the existing line cursor.
                #[cfg(target_arch = "x86_64")]
                if (2..=256).contains(&end)
                    && let Some(lines) =
                        vize_l0::line_index::ShortAsciiLines::new(source.split_at(end).0)
                {
                    let at = |edit: &vize_l1::edit::SpanEdit<'_, '_, SnapshotKey<'_>>| {
                        let span = edit.span();
                        let range = vize_l0::line_index::Range::new(
                            lines.position(span.start),
                            lines.position(span.end),
                        );
                        wire(range, edit.replacement())
                    };
                    if let [only] = raw {
                        return Ok(self.document_change(vec![at(only)]));
                    }
                    return Ok(self.document_change(raw.iter().map(at).collect()));
                }
            }
            if line_endings::needs_projection(source, checked.edits()) {
                return Ok(self.document_change(line_endings::project(source, checked.edits())));
            }
            if let Some((range, replacement)) = checked.single() {
                return Ok(self.document_change(vec![wire(range, replacement)]));
            }
            let edits = checked.ranges();
            let mut text_edits = Vec::with_capacity(edits.len());
            for (range, replacement) in edits {
                text_edits.push(wire(range, replacement));
            }
            Ok(self.document_change(text_edits))
        })
        .map_err(SourceEditRefusal::Host)?
    }

    #[inline]
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
