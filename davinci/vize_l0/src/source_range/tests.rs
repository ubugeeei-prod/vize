//! Unit checks of the range, mapping and source-map lookups.
use super::{SourceMap, SourceMapping, SourceRange};

#[test]
fn test_source_range_contains() {
    let range = SourceRange::new(10, 20);
    assert!(!range.contains(9));
    assert!(range.contains(10));
    assert!(range.contains(15));
    assert!(range.contains(19));
    assert!(!range.contains(20));
}

#[test]
fn test_source_range_intersects() {
    let a = SourceRange::new(10, 20);
    let b = SourceRange::new(15, 25);
    let c = SourceRange::new(25, 30);

    assert!(a.intersects(&b));
    assert!(b.intersects(&a));
    assert!(!a.intersects(&c));
    assert!(!c.intersects(&a));
}

#[test]
fn test_source_range_offset_saturates() {
    assert_eq!(SourceRange::new(10, 20).offset(5), SourceRange::new(15, 25));
    assert_eq!(SourceRange::new(10, 20).offset(-5), SourceRange::new(5, 15));
    assert_eq!(SourceRange::new(10, 20).offset(-50), SourceRange::new(0, 0));
    assert_eq!(
        SourceRange::new(u32::MAX - 1, u32::MAX).offset(10),
        SourceRange::new(u32::MAX, u32::MAX),
    );
}

#[test]
fn test_mapping_source_to_generated() {
    let mapping = SourceMapping::new(SourceRange::new(10, 20), SourceRange::new(100, 110));

    assert_eq!(mapping.source_to_generated(10), Some(100));
    assert_eq!(mapping.source_to_generated(15), Some(105));
    assert_eq!(mapping.source_to_generated(19), Some(109));
    assert_eq!(mapping.source_to_generated(9), None);
    assert_eq!(mapping.source_to_generated(20), None);
}

#[test]
fn test_source_map() {
    let mut map = SourceMap::new();
    map.add_simple(10, 20, 100, 110);
    map.add_simple(30, 40, 200, 210);

    assert_eq!(map.to_generated(15), Some(105));
    assert_eq!(map.to_generated(35), Some(205));
    assert_eq!(map.to_generated(25), None);

    assert_eq!(map.to_source(105), Some(15));
    assert_eq!(map.to_source(205), Some(35));
}

#[test]
fn to_generated_finds_the_mapping_beside_an_empty_or_inverted_range() {
    // Kani's counterexample: an empty range inserted after a mapping with
    // the same start hid the mapping that contains the offset.
    let mut map = SourceMap::new();
    map.add_simple(438_272_201, 2_585_755_849, 0, 10);
    map.add_simple(438_272_201, 438_272_201, 20, 30);
    map.add_simple(4_280_271_103, 4_293_705_983, 40, 50);
    assert_eq!(map.to_generated(2_466_300_108), Some(9));

    let mut map = SourceMap::new();
    map.add_simple(10, 20, 100, 110);
    map.add_simple(12, 4, 200, 210);
    map.add_simple(30, 40, 300, 310);
    assert_eq!(map.to_generated(15), Some(105));

    // Overlapping ranges: the first in source order wins.
    let mut map = SourceMap::new();
    map.add_simple(1, 2, 500, 501);
    map.add_simple(0, 10, 100, 110);
    assert_eq!(map.to_generated(5), Some(105));
    assert_eq!(map.to_generated(1), Some(101));
}

#[test]
fn to_source_has_no_coordinate_past_the_u32_file_range() {
    let mut map = SourceMap::new();
    map.add_simple(10, 20, 100, 110);
    map.set_block_offset(u32::MAX - 15);
    assert_eq!(map.to_source(105), Some(u32::MAX));
    assert_eq!(map.to_source(106), None);
    assert_eq!(map.to_source(99), None);
}
