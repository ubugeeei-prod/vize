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

    /// Recheck the retained snapshot against the host's current snapshot.
    ///
    /// Adapters that publish the checked edits without materializing a new
    /// source buffer must still perform this check while holding their normal
    /// host synchronization boundary.
    pub fn check_current(self, current: VersionedSource<'_, K>) -> Result<(), EditError> {
        self.snapshot.check(current)
    }

    /// Check the current host snapshot, then produce one new authored string.
    /// Validation and reservation precede emission; errors never mutate source.
    /// The host must serialize this check/application with its document changes.
    pub fn apply(self, current: VersionedSource<'_, K>) -> Result<String, EditError> {
        self.check_current(current)?;
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
mod tests {
    use super::{EditError, EditSet, VersionedSource};
    use crate::embed::{
        Embed, Grammar, Lang, Shape, SourceError, prepare_attribute_value,
        source::prepare_vue_interpolation_in, syntax::parse_once,
    };
    use oxc_span::GetSpan;
    use vize_l0::{Allocator, SourceRoot, Span, String};

    fn snapshot(source: &str) -> VersionedSource<'_, (&str, u64)> {
        // These are explicit caller inputs for the law, not a product URI authority.
        VersionedSource::new(
            ("retained document", 42),
            7,
            SourceRoot::new(source).unwrap(),
        )
    }

    #[test]
    fn checked_spans_reject_reversed_out_of_bounds_and_non_utf8_ranges() {
        let source = snapshot("αβ");
        for range in [
            Span::new(3, 2),
            Span::new(0, 5),
            Span::new(u32::MAX, u32::MAX),
        ] {
            assert_eq!(
                source.edit(range, "x").unwrap_err(),
                EditError::InvalidRange
            );
        }
        for range in [Span::new(1, 2), Span::new(0, 3), Span::new(3, 3)] {
            assert_eq!(
                source.edit(range, "x").unwrap_err(),
                EditError::NonUtf8Boundary
            );
        }
        let edit = source.edit(Span::new(0, 2), "Ω").unwrap();
        assert_eq!(edit.span(), Span::new(0, 2));
        assert_eq!(edit.replacement(), "Ω");
        assert_eq!(edit.snapshot().key(), ("retained document", 42));
        assert_eq!(edit.snapshot().version(), 7);
    }

    #[test]
    fn ordered_adjacent_edits_and_boundary_insertions_apply_once_to_original_bytes() {
        let root = "αβ";
        let frame = snapshot(root);
        let edits = [
            frame.edit(Span::new(0, 2), "Ω").unwrap(),
            frame.edit(Span::new(2, 2), "/").unwrap(),
            frame.edit(Span::new(2, 4), "γ").unwrap(),
            frame.edit(Span::new(4, 4), "!").unwrap(),
        ];
        let set = EditSet::new(frame, &edits).unwrap();
        assert_eq!(set.edits().as_ptr(), edits.as_ptr());
        assert_eq!(set.snapshot().root().source().as_ptr(), root.as_ptr());
        assert_eq!(set.apply(frame).unwrap(), "Ω/γ!");
        assert_eq!(root, "αβ");
        assert_eq!(
            EditSet::new(frame, &[]).unwrap().apply(frame).unwrap(),
            root
        );
    }

    #[test]
    fn conflicts_and_unsorted_edits_are_explicit_refusals_without_source_mutation() {
        let root = "abcdef";
        let frame = snapshot(root);
        for spans in [
            [Span::new(0, 4), Span::new(2, 5)],
            [Span::new(1, 4), Span::new(2, 2)],
            [Span::new(1, 4), Span::new(1, 1)],
            [Span::new(2, 2), Span::new(2, 2)],
            [Span::new(1, 3), Span::new(1, 3)],
        ] {
            let edits = spans.map(|range| frame.edit(range, "x").unwrap());
            assert_eq!(EditSet::new(frame, &edits).unwrap_err(), EditError::Overlap);
        }
        let edits = [
            frame.edit(Span::new(4, 5), "x").unwrap(),
            frame.edit(Span::new(0, 1), "y").unwrap(),
        ];
        assert_eq!(
            EditSet::new(frame, &edits).unwrap_err(),
            EditError::Unordered
        );
        assert_eq!(root, "abcdef");
    }

    #[test]
    fn stale_version_reopened_key_and_equal_foreign_buffer_never_apply() {
        let mut root = String::with_capacity(64);
        root.push_str("α original");
        let mut equal = String::with_capacity(64);
        equal.push_str(root.as_str());
        assert_eq!(root, equal);
        assert_ne!(root.as_ptr(), equal.as_ptr());
        let frame = snapshot(&root);
        let edits = [frame.edit(Span::new(0, 2), "B").unwrap()];
        let set = EditSet::new(frame, &edits).unwrap();
        assert_eq!(set.check_current(frame), Ok(()));
        let candidates = [
            (
                VersionedSource::new(frame.key(), 8, frame.root()),
                EditError::StaleVersion,
            ),
            (
                VersionedSource::new(("retained document", 43), 7, frame.root()),
                EditError::SnapshotMismatch,
            ),
            (
                VersionedSource::new(("another document", 42), 7, frame.root()),
                EditError::SnapshotMismatch,
            ),
            (
                VersionedSource::new(frame.key(), 7, SourceRoot::new(&equal).unwrap()),
                EditError::SourceMismatch,
            ),
        ];
        for (current, error) in candidates {
            assert_eq!(set.check_current(current), Err(error));
            assert_eq!(set.apply(current), Err(error));
            let mixed = [edits[0], current.edit(Span::new(2, 2), "!").unwrap()];
            assert_eq!(EditSet::new(frame, &mixed).unwrap_err(), error);
        }
        assert_eq!(root, "α original");
        assert_eq!(equal, "α original");
    }

    #[test]
    fn native_expression_span_projects_from_retained_preparation_without_reparse() {
        let arena = Allocator::default();
        let root = "<div :value=\"α + &fjlig;\" />";
        let raw = "α + &fjlig;";
        let start = root.find(raw).unwrap() as u32;
        let prepared =
            prepare_attribute_value(&arena, root, Span::new(start, start + raw.len() as u32))
                .unwrap();
        let syntax = parse_once(
            &arena,
            Embed {
                grammar: Grammar {
                    shape: Shape::Expr,
                    lang: Lang::Js,
                },
                source: prepared,
            },
        );
        let oxc_ast::ast::Expression::BinaryExpression(binary) = syntax.expression().unwrap()
        else {
            panic!("native binary expression expected");
        };
        let relative = syntax.decoded_span(binary.right.span()).unwrap();
        assert_eq!(
            syntax
                .source()
                .text()
                .get(relative.start as usize..relative.end as usize),
            Some("fj")
        );
        let frame = snapshot(root);
        let edit = frame
            .project_edit(syntax.source(), relative, "count")
            .unwrap();
        assert_eq!(
            root.get(edit.span().start as usize..edit.span().end as usize),
            Some("&fjlig;")
        );
        let edits = [edit];
        assert_eq!(
            EditSet::new(frame, &edits).unwrap().apply(frame).unwrap(),
            "<div :value=\"α + count\" />"
        );
        assert_eq!(
            frame
                .project_edit(
                    syntax.source(),
                    Span::new(relative.start, relative.start + 1),
                    "f"
                )
                .unwrap_err(),
            EditError::Projection(SourceError::PartialEntityBoundary)
        );
    }

    #[test]
    fn exact_projection_rejects_entity_interior_even_when_diagnostic_cover_is_valid() {
        let arena = Allocator::default();
        let root = "xx&acE;yy";
        let source = prepare_attribute_value(&arena, root, Span::new(2, 7)).unwrap();
        let point = Span::new(3, 3);
        assert_eq!(source.authored_covering_span(point), Ok(Span::new(2, 7)));
        assert_eq!(
            snapshot(root).project_edit(source, point, "!").unwrap_err(),
            EditError::Projection(SourceError::PartialEntityBoundary)
        );
        assert_eq!(
            snapshot(root)
                .project_edit(source, Span::new(1, 1), "!")
                .unwrap_err(),
            EditError::Projection(SourceError::InvalidDecodedSpan)
        );
        let edit = snapshot(root)
            .project_edit(source, Span::new(0, 5), "Ω")
            .unwrap();
        assert_eq!(
            EditSet::new(snapshot(root), &[edit])
                .unwrap()
                .apply(snapshot(root))
                .unwrap(),
            "xxΩyy"
        );
    }

    #[test]
    fn prepared_identity_slices_interpolation_and_foreign_equal_roots_stay_bound() {
        let arena = Allocator::default();
        let mut root = String::with_capacity(64);
        root.push_str("xx α&amp;β yy");
        let mut equal = String::with_capacity(64);
        equal.push_str(root.as_str());
        let source =
            prepare_vue_interpolation_in(&arena, &root, Span::new(2, root.len() as u32 - 2))
                .unwrap();
        let piece = source.slice_in(&arena, Span::new(0, 2)).unwrap();
        assert!(piece.decode_map().is_none());
        let frame = snapshot(&root);
        let edit = frame.project_edit(piece, Span::new(0, 2), "Ω").unwrap();
        assert_eq!(
            EditSet::new(frame, &[edit]).unwrap().apply(frame).unwrap(),
            "xx Ω&amp;β yy"
        );
        assert_eq!(
            snapshot(&equal)
                .project_edit(source, Span::new(0, 5), "x")
                .unwrap_err(),
            EditError::SourceMismatch
        );
    }

    #[test]
    fn shared_and_empty_bytes_still_require_actual_distinct_snapshot_keys() {
        for root in ["", "shared"] {
            let first = snapshot(root);
            let other = VersionedSource::new(("another document", 42), 7, first.root());
            let edits = [first.edit(Span::new(0, 0), "!").unwrap()];
            let set = EditSet::new(first, &edits).unwrap();
            assert_eq!(set.apply(other), Err(EditError::SnapshotMismatch));
            assert_eq!(set.apply(first).unwrap(), String::from("!") + root);
        }
    }
}
