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
        prepare_attribute_value(&arena, root, Span::new(start, start + raw.len() as u32)).unwrap();
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
    let oxc_ast::ast::Expression::BinaryExpression(binary) = syntax.expression().unwrap() else {
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
        prepare_vue_interpolation_in(&arena, &root, Span::new(2, root.len() as u32 - 2)).unwrap();
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
