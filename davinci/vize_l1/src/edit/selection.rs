//! Select diagnostic edits in their reporting order for one retained document.
//! Selection follows the CLI and NAPI policy. Publication and application go
//! through the same versioned EditSet as other authored edits.

use super::{EditError, EditSet, SpanEdit, VersionedSource};
use vize_l0::{SmallVec, Span};

/// Selected edits in byte order. Borrow the set for either publication or
/// one-pass application against the host's current version.
pub struct SelectedEdits<'source, 'replacement, K> {
    snapshot: VersionedSource<'source, K>,
    edits: SmallVec<[SpanEdit<'source, 'replacement, K>; 2]>,
    output_len: usize,
}

impl<'source, K: Copy + Eq> VersionedSource<'source, K> {
    /// Drop invalid authored ranges, sort by (start, end) while preserving
    /// diagnostic order for ties, and keep the first nonoverlapping edits.
    /// Multiple insertions at one offset retain their original order. When no
    /// candidate has a usable range, there is no batch to publish or apply.
    #[inline]
    pub fn select_edits<'replacement>(
        self,
        candidates: impl IntoIterator<Item = (Span, &'replacement str)>,
    ) -> Result<Option<SelectedEdits<'source, 'replacement, K>>, EditError> {
        let mut valid = candidates.into_iter().filter_map(|(span, replacement)| {
            self.root.contains_span(span).then_some(SpanEdit {
                snapshot: self,
                span,
                replacement,
            })
        });
        let Some(first) = valid.next() else {
            return Ok(None);
        };
        SelectedEdits::from_first(self, first, valid).map(Some)
    }
}

impl<'source, 'replacement, K: Copy + Eq> SelectedEdits<'source, 'replacement, K> {
    fn from_first(
        snapshot: VersionedSource<'source, K>,
        first: SpanEdit<'source, 'replacement, K>,
        rest: impl Iterator<Item = SpanEdit<'source, 'replacement, K>>,
    ) -> Result<Self, EditError> {
        let mut edits = SmallVec::<[SpanEdit<'source, 'replacement, K>; 2]>::new();
        edits.push(first);
        for edit in rest {
            edits
                .try_reserve(1)
                .map_err(|_| EditError::AllocationFailed)?;
            edits.push(edit);
        }
        edits.sort_by_key(|edit| (edit.span.start, edit.span.end));
        let mut last_end = 0;
        let mut removed = 0;
        let mut added = Some(0usize);
        edits.retain(|edit| {
            if edit.span.start < last_end {
                return false;
            }
            last_end = edit.span.end;
            // Selected ranges are disjoint, so removed bytes never exceed the source.
            removed += (edit.span.end - edit.span.start) as usize;
            added = added.and_then(|len| len.checked_add(edit.replacement.len()));
            true
        });
        let output_len = added
            .and_then(|n| (snapshot.root.source().len() - removed).checked_add(n))
            .ok_or(EditError::OutputTooLarge)?;
        if output_len > u32::MAX as usize {
            return Err(EditError::OutputTooLarge);
        }
        Ok(Self {
            snapshot,
            edits,
            output_len,
        })
    }

    /// Obtain the validated view, including all ordered same-offset inserts.
    /// The host supplies its current snapshot when publishing or applying.
    #[must_use]
    pub fn as_edit_set(&self) -> EditSet<'_, 'source, 'replacement, K> {
        EditSet {
            snapshot: self.snapshot,
            edits: &self.edits,
            output_len: self.output_len,
        }
    }
}

#[cfg(test)]
mod tests;
