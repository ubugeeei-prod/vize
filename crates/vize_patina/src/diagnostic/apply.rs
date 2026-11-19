//! Fix application. Every applier binds its edits to the exact input that was
//! linted and goes through one `vize_l1::edit` selection and edit set (#6876):
//! invalid ranges are dropped, edits are ordered by range with ties in
//! diagnostic order, and the first of overlapping edits wins.

use vize_l0::{SourceRoot, Span, String};
use vize_l1::edit::{EditSet, VersionedSource, select_first};

use super::types::TextEdit;
use crate::linter::LintResult;

/// Apply `edits` to `source`, or `None` when none of them is valid.
pub(super) fn apply_edits<'e>(
    source: &str,
    edits: impl Iterator<Item = &'e TextEdit>,
) -> Option<String> {
    // The splice runs synchronously on the buffer the edits were made for, so
    // the buffer itself is the snapshot identity.
    let frame = VersionedSource::new((), 0, SourceRoot::new(source).ok()?);
    let mut checked: Vec<_> = edits
        .filter_map(|edit| {
            frame
                .edit(Span::new(edit.start, edit.end), edit.new_text.as_str())
                .ok()
        })
        .collect();
    if checked.is_empty() {
        return None;
    }
    let kept = select_first(&mut checked);
    EditSet::new(frame, checked.get(..kept)?)
        .ok()?
        .apply(frame)
        .ok()
}

impl LintResult {
    /// Apply every diagnostic's fix to `source`, the input this result was
    /// produced from. `None` when no diagnostic carries a valid edit.
    pub fn apply_fixes(&self, source: &str) -> Option<String> {
        apply_edits(
            source,
            self.diagnostics
                .iter()
                .filter_map(|diagnostic| diagnostic.fix.as_ref())
                .flat_map(|fix| fix.edits.iter()),
        )
    }
}
