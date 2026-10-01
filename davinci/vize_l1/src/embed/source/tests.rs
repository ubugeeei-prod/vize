use super::{
    DecodeSegmentKind as Kind, EmbedSource, SourceError, checked_add, checked_len,
    prepare_attribute_value,
};
use alloc::vec::Vec;
use vize_l0::{Allocator, Span};

#[test]
fn unchanged_values_borrow_without_arena_allocation() {
    let allocator = Allocator::default();
    for raw in ["", "α + β", "&unknown;", "a&b", "&amp=1", "&#x;", "&#;"] {
        let authored = vize_l0::cstr!("x{raw}z");
        let source =
            prepare_attribute_value(&allocator, &authored, Span::new(1, 1 + raw.len() as u32))
                .unwrap();
        assert_eq!(source.text(), raw);
        assert_eq!(source.text().as_ptr(), authored[1..].as_ptr());
        assert!(source.decode_map().is_none());
        assert_eq!(allocator.allocated_bytes(), 0);
    }
    let source = prepare_attribute_value(&allocator, "&amp;", Span::new(0, 5)).unwrap();
    assert_eq!(source.text(), "&");
    assert!(allocator.allocated_bytes() > 0);
}

#[test]
fn native_decoder_retains_full_reference_values_and_does_not_decode_twice() {
    let allocator = Allocator::default();
    for (raw, decoded) in [
        ("&fjlig;", "fj"),
        ("&acE;", "\u{223e}\u{0333}"),
        ("&amp;#40;", "&#40;"),
        ("&#x80;", "€"),
        ("&#0;", "\u{fffd}"),
        ("&#40;value&#41;", "(value)"),
    ] {
        let source =
            prepare_attribute_value(&allocator, raw, Span::new(0, raw.len() as u32)).unwrap();
        assert_eq!(source.text(), decoded, "{raw}");
        assert!(source.decode_map().is_some());
        assert_eq!(
            source.authored_span(Span::new(0, decoded.len() as u32)),
            Ok(source.span())
        );
    }
}

#[test]
fn prepared_maps_cover_both_streams_and_preserve_literal_bytes() {
    let allocator = Allocator::default();
    let authored = "xxα&fjlig;&amp;β&unknown;yy";
    let span = Span::new(2, authored.len() as u32 - 2);
    let source = prepare_attribute_value(&allocator, authored, span).unwrap();
    assert_eq!(source.text(), "αfj&β&unknown;");
    let map = source.decode_map().unwrap();
    let (mut decoded, mut original) = (0, span.start);
    for segment in map.segments() {
        assert_eq!(segment.decoded().start, decoded);
        assert_eq!(segment.authored().start, original);
        let value =
            &source.text()[segment.decoded().start as usize..segment.decoded().end as usize];
        let raw = &authored[segment.authored().start as usize..segment.authored().end as usize];
        match segment.kind() {
            Kind::Identity => assert_eq!(raw, value),
            Kind::Entity => assert!(raw.starts_with('&')),
        }
        (decoded, original) = (segment.decoded().end, segment.authored().end);
    }
    assert_eq!((decoded, original), (source.text().len() as u32, span.end));
}

#[test]
fn exact_edits_reject_partial_entities_and_diagnostic_points_cover_them() {
    let allocator = Allocator::default();
    let source = prepare_attribute_value(&allocator, "xx&acE;z", Span::new(2, 7)).unwrap();
    let entity = source.decode_map().unwrap().segments()[0];
    assert_eq!(entity.kind(), Kind::Entity);
    assert_eq!(
        entity.decoded().end - entity.decoded().start,
        entity.authored().end - entity.authored().start
    );
    for partial in [Span::new(0, 3), Span::new(3, 5), Span::new(3, 3)] {
        assert_eq!(
            source.authored_span(partial),
            Err(SourceError::PartialEntityBoundary)
        );
        assert_eq!(source.authored_covering_span(partial), Ok(Span::new(2, 7)));
    }
    assert_eq!(source.authored_span(Span::new(0, 5)), Ok(Span::new(2, 7)));
    for (decoded, authored) in [(0, 2), (5, 7)] {
        assert_eq!(
            source.authored_covering_span(Span::new(decoded, decoded)),
            Ok(Span::new(authored, authored))
        );
    }
}

#[test]
fn every_utf8_boundary_range_has_a_valid_cover_and_exact_projection_is_inside_it() {
    let allocator = Allocator::default();
    let authored = "xxα&fjlig;&acE;β&amp;yy";
    let source = prepare_attribute_value(
        &allocator,
        authored,
        Span::new(2, authored.len() as u32 - 2),
    )
    .unwrap();
    let boundaries: Vec<u32> = (0..=source.text().len())
        .filter(|&n| source.text().is_char_boundary(n))
        .map(|n| n as u32)
        .collect();
    for &start in &boundaries {
        for &end in boundaries.iter().filter(|&&n| n >= start) {
            let span = Span::new(start, end);
            let cover = source.authored_covering_span(span).unwrap();
            assert!(
                authored
                    .get(cover.start as usize..cover.end as usize)
                    .is_some()
            );
            assert!(cover.start >= source.span().start && cover.end <= source.span().end);
            match source.authored_span(span) {
                Ok(exact) => {
                    assert!(cover.start <= exact.start && cover.end >= exact.end);
                }
                Err(error) => assert_eq!(error, SourceError::PartialEntityBoundary),
            }
        }
    }
}

#[test]
fn all_constructors_and_projections_reject_invalid_ranges_and_utf8_boundaries() {
    let allocator = Allocator::default();
    for span in [Span::new(1, 2), Span::new(0, 4), Span::new(3, 2)] {
        assert_eq!(
            EmbedSource::authored("éa", span).unwrap_err(),
            SourceError::InvalidAuthoredSpan
        );
        assert_eq!(
            prepare_attribute_value(&allocator, "éa", span).unwrap_err(),
            SourceError::InvalidAuthoredSpan
        );
    }
    let raw = EmbedSource::authored("xéay", Span::new(1, 4)).unwrap();
    let decoded = prepare_attribute_value(&allocator, "&acE;", Span::new(0, 5)).unwrap();
    for source in [raw, decoded] {
        for span in [
            Span::new(0, 1),
            Span::new(1, 1),
            Span::new(3, 2),
            Span::new(0, 8),
        ] {
            assert_eq!(
                source.authored_span(span),
                Err(SourceError::InvalidDecodedSpan)
            );
            assert_eq!(
                source.authored_covering_span(span),
                Err(SourceError::InvalidDecodedSpan)
            );
        }
    }
    assert_eq!(raw.authored_span(Span::new(2, 3)), Ok(Span::new(3, 4)));
    assert_eq!(raw.authored_span(Span::new(3, 3)), Ok(Span::new(4, 4)));
}

#[test]
fn checked_offsets_reject_u32_and_machine_overflow_without_large_allocations() {
    assert_eq!(checked_len(u32::MAX as usize), Ok(u32::MAX));
    assert_eq!(checked_add(usize::MAX, 1), Err(SourceError::SourceTooLarge));
    if let Some(over) = (u32::MAX as usize).checked_add(1) {
        assert_eq!(checked_len(over), Err(SourceError::SourceTooLarge));
        assert_eq!(
            checked_add(u32::MAX as usize, 1),
            Err(SourceError::SourceTooLarge)
        );
    }
}
