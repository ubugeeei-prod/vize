//! Original authored edits bound to the actual open-document snapshot.

use tower_lsp::lsp_types::{
    CodeActionOrCommand, Diagnostic, DocumentChanges, OneOf,
    OptionalVersionedTextDocumentIdentifier, Position, Range, TextDocumentEdit, TextEdit,
    WorkspaceEdit,
};
use vize_l0::line_index::offset_to_line_col;
use vize_l0::{SourceRoot, Span};
use vize_l1::edit::{EditSet, VersionedSource};

use super::{CodeActionService, IdeContext};

/// Supplied together with the request's real URI and shared authored buffer.
#[derive(Clone, Copy)]
pub(super) struct LintEditStamp {
    pub(super) revision: u64,
    pub(super) version: i32,
}

impl CodeActionService {
    /// Only the server's advertised-client path supplies this captured stamp.
    pub(crate) fn code_actions_for_snapshot(
        ctx: &IdeContext,
        range: Range,
        diagnostics: &[Diagnostic],
        revision: u64,
        version: i32,
    ) -> Vec<CodeActionOrCommand> {
        Self::collect_actions(
            ctx,
            range,
            diagnostics,
            Some(LintEditStamp { revision, version }),
        )
    }
}

/// Byte offsets are from an original descriptor body, never decoded coordinates.
pub(super) fn workspace_edit(
    ctx: &IdeContext,
    stamp: LintEditStamp,
    body: &str,
    body_start: usize,
    edits: &[vize_patina::TextEdit],
) -> Option<WorkspaceEdit> {
    let body_end = body_start.checked_add(body.len())?;
    if ctx.content.get(body_start..body_end)? != body {
        return None;
    }
    let frame = VersionedSource::new(
        (ctx.uri, stamp.revision),
        stamp.version,
        SourceRoot::new(&ctx.content).ok()?,
    );
    let checked = edits
        .iter()
        .map(|edit| {
            let start = u32::try_from(body_start.checked_add(edit.start as usize)?).ok()?;
            let end = u32::try_from(body_start.checked_add(edit.end as usize)?).ok()?;
            // Refuse edits outside the actual body even if they fit the SFC.
            if edit.end as usize > body.len() {
                return None;
            }
            frame
                .edit(Span::new(start, end), edit.new_text.as_str())
                .ok()
        })
        .collect::<Option<Vec<_>>>()?;
    let set = EditSet::new(frame, &checked).ok()?;

    // No guard escapes this synchronous boundary or crosses an await.
    let document = ctx.state.documents.get(ctx.uri)?;
    let current_source = document.text_snapshot();
    let current = VersionedSource::new(
        (&document.uri, document.revision()),
        document.version,
        SourceRoot::new(&current_source).ok()?,
    );
    set.check_current(current).ok()?;
    let edits = set
        .edits()
        .iter()
        .map(|edit| {
            let span = edit.span();
            let (start_line, start_character) =
                offset_to_line_col(&ctx.content, span.start as usize);
            let (end_line, end_character) = offset_to_line_col(&ctx.content, span.end as usize);
            OneOf::Left(TextEdit {
                range: Range::new(
                    Position::new(start_line, start_character),
                    Position::new(end_line, end_character),
                ),
                new_text: edit.replacement().into(),
            })
        })
        .collect();
    Some(WorkspaceEdit {
        changes: None,
        document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
            text_document: OptionalVersionedTextDocumentIdentifier {
                uri: ctx.uri.clone(),
                version: Some(stamp.version),
            },
            edits,
        }])),
        change_annotations: None,
    })
}

#[cfg(test)]
mod tests;
