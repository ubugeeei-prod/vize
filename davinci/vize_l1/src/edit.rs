//! Immutable edits against one retained, host-versioned authored snapshot.
//!
//! The host supplies its real snapshot key and client version together with
//! the original parsed buffer. A key must distinguish documents and revisions
//! even when client versions reset or bytes are shared. These types do not
//! mint document identities or authenticate a language/provider. Applying an
//! edit also requires the host to supply its current snapshot under the host's
//! ordinary synchronization; constructing a frame cannot query a document.

use vize_l0::{SourceRoot, Span, String};

use crate::embed::{EmbedSource, SourceError};

/// A refusal leaves the authored snapshot untouched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditError {
    SnapshotMismatch,
    StaleVersion,
    SourceMismatch,
    InvalidRange,
    NonUtf8Boundary,
    Projection(SourceError),
    Unordered,
    Overlap,
    OutputTooLarge,
    AllocationFailed,
}

/// A real host snapshot identity, version and validated original source.
///
/// For example, a caller may borrow its actual URI and include its actual
/// document revision in `K`; the version alone is insufficient on reopen.
/// Buffer identity compares pointer and length, not equal text. Shared/empty
/// buffers still require a distinct host key. Keep the parsed source alive;
/// an independently copied equal-text string is a different buffer.
#[derive(Debug, Clone, Copy)]
pub struct VersionedSource<'source, K> {
    key: K,
    version: i32,
    root: SourceRoot<'source>,
}

impl<'source, K: Copy + Eq> VersionedSource<'source, K> {
    #[must_use]
    pub const fn new(key: K, version: i32, root: SourceRoot<'source>) -> Self {
        Self { key, version, root }
    }

    #[must_use]
    pub const fn key(self) -> K {
        self.key
    }

    #[must_use]
    pub const fn version(self) -> i32 {
        self.version
    }

    #[must_use]
    pub const fn root(self) -> SourceRoot<'source> {
        self.root
    }

    /// Check an authored, end-exclusive UTF-8 byte range without allocation.
    pub fn edit<'replacement>(
        self,
        span: Span,
        replacement: &'replacement str,
    ) -> Result<SpanEdit<'source, 'replacement, K>, EditError> {
        let source = self.root.source();
        if span.start > span.end || span.end as usize > source.len() {
            return Err(EditError::InvalidRange);
        }
        if !source.is_char_boundary(span.start as usize)
            || !source.is_char_boundary(span.end as usize)
        {
            return Err(EditError::NonUtf8Boundary);
        }
        Ok(SpanEdit {
            snapshot: self,
            span,
            replacement,
        })
    }

    /// Project an already-prepared embed exactly, retaining this host snapshot.
    /// No decoding, AST walk or conservative diagnostic covering is performed.
    /// Entity interiors and foreign equal-text preparation inputs are refused.
    pub fn project_edit<'replacement>(
        self,
        source: EmbedSource<'_>,
        relative: Span,
        replacement: &'replacement str,
    ) -> Result<SpanEdit<'source, 'replacement, K>, EditError> {
        if !same_buffer(self.root.source(), source.authored_root()) {
            return Err(EditError::SourceMismatch);
        }
        let span = source
            .authored_span(relative)
            .map_err(EditError::Projection)?;
        self.edit(span, replacement)
    }

    fn check(self, current: VersionedSource<'_, K>) -> Result<(), EditError> {
        if self.key != current.key {
            return Err(EditError::SnapshotMismatch);
        }
        if self.version != current.version {
            return Err(EditError::StaleVersion);
        }
        if !same_buffer(self.root.source(), current.root.source()) {
            return Err(EditError::SourceMismatch);
        }
        Ok(())
    }
}

fn same_buffer(left: &str, right: &str) -> bool {
    left.as_ptr() == right.as_ptr() && left.len() == right.len()
}

/// A checked replacement; private fields prevent unchecked or later mutation.
///
/// ```compile_fail
/// use vize_l0::{SourceRoot, Span};
/// use vize_l1::edit::VersionedSource;
/// let frame = VersionedSource::new(42_u64, 7, SourceRoot::new("α").unwrap());
/// let mut edit = frame.edit(Span::new(0, 2), "a").unwrap();
/// edit.span = Span::new(1, 2); // Cannot bypass checked UTF-8 construction.
/// ```
#[derive(Debug, Clone, Copy)]
pub struct SpanEdit<'source, 'replacement, K> {
    snapshot: VersionedSource<'source, K>,
    span: Span,
    replacement: &'replacement str,
}

impl<'source, 'replacement, K: Copy + Eq> SpanEdit<'source, 'replacement, K> {
    #[must_use]
    pub const fn snapshot(self) -> VersionedSource<'source, K> {
        self.snapshot
    }

    #[must_use]
    pub const fn span(self) -> Span {
        self.span
    }

    #[must_use]
    pub const fn replacement(self) -> &'replacement str {
        self.replacement
    }
}

/// An immutable, source-ordered set; construction borrows and allocates nothing.
///
/// Adjacent ranges and insertion at a replacement boundary are allowed, in
/// supplied order. Interior overlaps and duplicate insertions at one point are
/// refused. Consumers choose order explicitly; this type does not clone/sort.
/// At a replacement's start, insertions precede it; later insertions may use
/// its end. An insertion back at its consumed start is an overlap.
#[derive(Debug, Clone, Copy)]
pub struct EditSet<'edits, 'source, 'replacement, K> {
    snapshot: VersionedSource<'source, K>,
    edits: &'edits [SpanEdit<'source, 'replacement, K>],
    output_len: usize,
}

impl<'edits, 'source, 'replacement, K: Copy + Eq> EditSet<'edits, 'source, 'replacement, K> {
    pub fn new(
        snapshot: VersionedSource<'source, K>,
        edits: &'edits [SpanEdit<'source, 'replacement, K>],
    ) -> Result<Self, EditError> {
        let mut previous: Option<Span> = None;
        let mut removed = 0usize;
        let mut added = 0usize;
        for edit in edits {
            snapshot.check(edit.snapshot)?;
            if let Some(prior) = previous {
                if edit.span.start < prior.start {
                    return Err(EditError::Unordered);
                }
                if edit.span.start < prior.end || (prior.start == prior.end && edit.span == prior) {
                    return Err(EditError::Overlap);
                }
            }
            removed = removed
                .checked_add((edit.span.end - edit.span.start) as usize)
                .ok_or(EditError::OutputTooLarge)?;
            added = added
                .checked_add(edit.replacement.len())
                .ok_or(EditError::OutputTooLarge)?;
            previous = Some(edit.span);
        }
        let output_len = snapshot
            .root
            .source()
            .len()
            .checked_sub(removed)
            .and_then(|length| length.checked_add(added))
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

    #[must_use]
    pub const fn snapshot(self) -> VersionedSource<'source, K> {
        self.snapshot
    }

    #[must_use]
    pub const fn edits(self) -> &'edits [SpanEdit<'source, 'replacement, K>] {
        self.edits
    }

    /// Check the current host snapshot, then produce one new authored string.
    /// Validation and reservation precede emission; errors never mutate source.
    /// The host must serialize this check/application with its document changes.
    pub fn apply(self, current: VersionedSource<'_, K>) -> Result<String, EditError> {
        self.snapshot.check(current)?;
        let source = self.snapshot.root.source();
        let mut output =
            String::try_with_capacity(self.output_len).map_err(|_| EditError::AllocationFailed)?;
        let mut cursor = 0usize;
        for edit in self.edits {
            let unchanged = source
                .get(cursor..edit.span.start as usize)
                .ok_or(EditError::InvalidRange)?;
            output.push_str(unchanged);
            output.push_str(edit.replacement);
            cursor = edit.span.end as usize;
        }
        output.push_str(source.get(cursor..).ok_or(EditError::InvalidRange)?);
        Ok(output)
    }
}

#[cfg(test)]
mod tests;
