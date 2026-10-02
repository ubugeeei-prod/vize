use vize_l0::{Allocator, Span};

use super::{DecodeSegmentKind, EmbedSource, SourceError, prepare_attribute_value};

#[test]
fn identity_slices_borrow_exact_utf8_and_preserve_file_offsets_without_allocating() {
    let allocator = Allocator::default();
    let source = EmbedSource::authored("zzαβγzz", Span::new(2, 8)).unwrap();
    let allocated = allocator.allocated_bytes();
    let piece = source.slice_in(&allocator, Span::new(2, 4)).unwrap();
    assert_eq!(piece.text(), "β");
    assert_eq!(piece.span(), Span::new(4, 6));
    assert!(piece.decode_map().is_none());
    assert_eq!(
        piece.text().as_ptr(),
        source.text().get(2..4).unwrap().as_ptr()
    );
    assert_eq!(piece.authored_span(Span::new(0, 2)), Ok(Span::new(4, 6)));
    assert_eq!(allocator.allocated_bytes(), allocated);
}

#[test]
fn derived_map_shortens_identity_segments_and_keeps_complete_entity_atoms() {
    let allocator = Allocator::default();
    let source =
        prepare_attribute_value(&allocator, "zzab&fjlig;c&acE;dezz", Span::new(2, 19)).unwrap();
    let piece = source.slice_in(&allocator, Span::new(1, 11)).unwrap();
    assert_eq!(piece.text(), "bfjc∾̳d");
    assert_eq!(piece.span(), Span::new(3, 18));
    let map = piece.decode_map().unwrap();
    let expected = [
        (
            Span::new(0, 1),
            Span::new(3, 4),
            DecodeSegmentKind::Identity,
        ),
        (Span::new(1, 3), Span::new(4, 11), DecodeSegmentKind::Entity),
        (
            Span::new(3, 4),
            Span::new(11, 12),
            DecodeSegmentKind::Identity,
        ),
        (
            Span::new(4, 9),
            Span::new(12, 17),
            DecodeSegmentKind::Entity,
        ),
        (
            Span::new(9, 10),
            Span::new(17, 18),
            DecodeSegmentKind::Identity,
        ),
    ];
    assert_eq!(map.segments().len(), expected.len());
    for (segment, (decoded, authored, kind)) in map.segments().iter().zip(expected) {
        assert_eq!(segment.decoded(), decoded);
        assert_eq!(segment.authored(), authored);
        assert_eq!(segment.kind(), kind);
    }
    let nested = piece.slice_in(&allocator, Span::new(1, 9)).unwrap();
    assert_eq!(nested.text(), "fjc∾̳");
    assert_eq!(nested.span(), Span::new(4, 17));
    assert_eq!(nested.authored_span(Span::new(0, 2)), Ok(Span::new(4, 11)));
    assert_eq!(nested.authored_span(Span::new(3, 8)), Ok(Span::new(12, 17)));
}

#[test]
fn full_map_and_identity_only_or_empty_pieces_avoid_new_storage() {
    let allocator = Allocator::default();
    let source =
        prepare_attribute_value(&allocator, "zzabc&fjlig;defzz", Span::new(2, 15)).unwrap();
    let allocated = allocator.allocated_bytes();
    let full = source.slice_in(&allocator, Span::new(0, 8)).unwrap();
    assert_eq!(
        full.decode_map().unwrap().segments().as_ptr(),
        source.decode_map().unwrap().segments().as_ptr()
    );
    let identity = source.slice_in(&allocator, Span::new(1, 3)).unwrap();
    assert_eq!(identity.text(), "bc");
    assert_eq!(identity.span(), Span::new(3, 5));
    assert!(identity.decode_map().is_none());
    let empty = source.slice_in(&allocator, Span::new(3, 3)).unwrap();
    assert_eq!(empty.text(), "");
    assert_eq!(empty.span(), Span::new(5, 5));
    assert!(empty.decode_map().is_none());
    assert_eq!(allocator.allocated_bytes(), allocated);
}

#[test]
fn cuts_reject_partial_entities_and_invalid_utf8_ranges() {
    let allocator = Allocator::default();
    let source = prepare_attribute_value(&allocator, "zz&fjlig;zz", Span::new(2, 9)).unwrap();
    for span in [Span::new(0, 1), Span::new(1, 1), Span::new(1, 2)] {
        assert_eq!(
            source.slice_in(&allocator, span).unwrap_err(),
            SourceError::PartialEntityBoundary
        );
    }
    for span in [Span::new(2, 1), Span::new(0, u32::MAX)] {
        assert_eq!(
            source.slice_in(&allocator, span).unwrap_err(),
            SourceError::InvalidDecodedSpan
        );
    }
    let unicode = prepare_attribute_value(&allocator, "zz&acE;zz", Span::new(2, 7)).unwrap();
    assert_eq!(
        unicode.slice_in(&allocator, Span::new(1, 2)).unwrap_err(),
        SourceError::InvalidDecodedSpan
    );
    assert_eq!(
        unicode.slice_in(&allocator, Span::new(3, 3)).unwrap_err(),
        SourceError::PartialEntityBoundary
    );
    assert_eq!(
        unicode.slice_in(&allocator, Span::new(0, 3)).unwrap_err(),
        SourceError::PartialEntityBoundary
    );
}

#[test]
fn a_piece_never_decodes_a_previously_decoded_reference_again() {
    let allocator = Allocator::default();
    let source = prepare_attribute_value(&allocator, "zza&amp;amp;bzz", Span::new(2, 13)).unwrap();
    let piece = source.slice_in(&allocator, Span::new(1, 6)).unwrap();
    assert_eq!(piece.text(), "&amp;");
    assert_eq!(piece.span(), Span::new(3, 12));
    assert_eq!(piece.authored_span(Span::new(0, 1)), Ok(Span::new(3, 8)));
    assert_eq!(piece.authored_span(Span::new(1, 5)), Ok(Span::new(8, 12)));
}

#[test]
fn every_piece_utf8_range_preserves_parent_exact_and_covering_projections() {
    let allocator = Allocator::default();
    let source =
        prepare_attribute_value(&allocator, "zza&fjlig;b&acE;czz", Span::new(2, 17)).unwrap();
    let piece = source.slice_in(&allocator, Span::new(1, 9)).unwrap();
    for start in 0..=piece.text().len() {
        for end in start..=piece.text().len() {
            let child = Span::new(start as u32, end as u32);
            let parent = Span::new(start as u32 + 1, end as u32 + 1);
            assert_eq!(piece.authored_span(child), source.authored_span(parent));
            assert_eq!(
                piece.authored_covering_span(child),
                source.authored_covering_span(parent)
            );
        }
    }
}
