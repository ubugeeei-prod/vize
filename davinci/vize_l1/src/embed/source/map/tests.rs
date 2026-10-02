use super::{DecodeMap, DecodeSegment, DecodeSegmentKind as Kind, SourceError};
use alloc::vec;
use vize_l0::Span;

fn segment(decoded: (u32, u32), authored: (u32, u32), kind: Kind) -> DecodeSegment {
    DecodeSegment::new(
        Span::new(decoded.0, decoded.1),
        Span::new(authored.0, authored.1),
        kind,
    )
}

#[test]
fn coverage_rejects_gaps_overlaps_empty_and_incomplete_maps() {
    let invalid = [
        vec![],
        vec![segment((1, 2), (0, 2), Kind::Identity)],
        vec![segment((0, 2), (1, 2), Kind::Identity)],
        vec![segment((0, 1), (0, 1), Kind::Identity)],
        vec![segment((0, 0), (0, 1), Kind::Identity)],
        vec![segment((0, 1), (0, 0), Kind::Identity)],
        vec![
            segment((0, 1), (0, 1), Kind::Identity),
            segment((0, 2), (1, 2), Kind::Identity),
        ],
        vec![
            segment((0, 1), (0, 1), Kind::Identity),
            segment((1, 2), (0, 2), Kind::Identity),
        ],
        vec![segment((0, 2), (0, 3), Kind::Identity)],
        vec![
            segment((0, 2), (0, 2), Kind::Identity),
            segment((2, 1), (2, 2), Kind::Identity),
        ],
    ];
    for segments in invalid {
        assert_eq!(
            DecodeMap::checked("ab", Span::new(0, 2), "ab", &segments).unwrap_err(),
            SourceError::InvalidMapCoverage
        );
    }
    assert_eq!(
        DecodeMap::checked("", Span::new(0, 0), "", &[]).unwrap_err(),
        SourceError::InvalidMapCoverage
    );
}

#[test]
fn coverage_checks_utf8_and_identity_bytes() {
    let segments = [segment((0, 2), (0, 2), Kind::Identity)];
    assert_eq!(
        DecodeMap::checked("ab", Span::new(0, 2), "ac", &segments).unwrap_err(),
        SourceError::InvalidIdentitySegment
    );
    let segments = [
        segment((0, 1), (0, 2), Kind::Identity),
        segment((1, 2), (2, 3), Kind::Identity),
    ];
    assert_eq!(
        DecodeMap::checked("éa", Span::new(0, 3), "é", &segments).unwrap_err(),
        SourceError::InvalidDecodedSpan
    );
    let segments = [
        segment((0, 2), (0, 1), Kind::Identity),
        segment((2, 3), (1, 3), Kind::Identity),
    ];
    assert_eq!(
        DecodeMap::checked("éa", Span::new(0, 3), "éa", &segments).unwrap_err(),
        SourceError::InvalidAuthoredSpan
    );
    assert_eq!(
        DecodeMap::checked("é", Span::new(1, 2), "", &[]).unwrap_err(),
        SourceError::InvalidAuthoredSpan
    );
    let segments = [segment((0, 1), (0, 1), Kind::Entity)];
    assert_eq!(
        DecodeMap::checked("a", Span::new(0, 1), "a", &segments).unwrap_err(),
        SourceError::InvalidEntitySegment
    );
}

#[test]
fn exact_and_covering_projections_are_distinct_at_entity_interiors() {
    let segments = [segment((0, 2), (2, 9), Kind::Entity)];
    let map = DecodeMap::checked("xx&fjlig;z", Span::new(2, 9), "fj", &segments).unwrap();
    for span in [Span::new(0, 1), Span::new(1, 2), Span::new(1, 1)] {
        assert_eq!(
            map.project(span, false),
            Err(SourceError::PartialEntityBoundary)
        );
        assert_eq!(map.project(span, true), Ok(Span::new(2, 9)));
    }
    for (decoded, authored) in [(0, 2), (2, 9)] {
        for covering in [false, true] {
            assert_eq!(
                map.project(Span::new(decoded, decoded), covering),
                Ok(Span::new(authored, authored))
            );
        }
    }
    assert_eq!(map.project(Span::new(0, 2), false), Ok(Span::new(2, 9)));
}

#[test]
fn equal_length_entity_is_never_treated_as_identity() {
    let segments = [segment((0, 5), (0, 5), Kind::Entity)];
    let map = DecodeMap::checked("&acE;", Span::new(0, 5), "\u{223e}\u{0333}", &segments).unwrap();
    assert_eq!(
        map.project(Span::new(0, 3), false),
        Err(SourceError::PartialEntityBoundary)
    );
    assert_eq!(map.project(Span::new(3, 3), true), Ok(Span::new(0, 5)));
}

#[test]
fn adjacent_entity_and_identity_endpoints_are_deterministic() {
    let segments = [
        segment((0, 1), (1, 2), Kind::Identity),
        segment((1, 3), (2, 9), Kind::Entity),
        segment((3, 4), (9, 14), Kind::Entity),
        segment((4, 6), (14, 16), Kind::Identity),
    ];
    let map = DecodeMap::checked("xa&fjlig;&amp;éz", Span::new(1, 16), "afj&é", &segments).unwrap();
    for (decoded, authored) in [(0, 1), (1, 2), (3, 9), (4, 14), (6, 16)] {
        for covering in [false, true] {
            assert_eq!(
                map.project(Span::new(decoded, decoded), covering),
                Ok(Span::new(authored, authored))
            );
        }
    }
    assert_eq!(map.project(Span::new(2, 6), true), Ok(Span::new(2, 16)));
    assert_eq!(map.project(Span::new(0, 2), true), Ok(Span::new(1, 9)));
}
