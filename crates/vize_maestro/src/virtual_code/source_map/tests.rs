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
fn test_mapping_source_to_generated() {
    let mapping = SourceMapping::new(SourceRange::new(10, 20), SourceRange::new(100, 110));

    assert_eq!(mapping.source_to_generated(10), Some(100));
    assert_eq!(mapping.source_to_generated(15), Some(105));
    assert_eq!(mapping.source_to_generated(19), Some(109));
    assert_eq!(mapping.source_to_generated(9), None);
    assert_eq!(mapping.source_to_generated(20), None);
}

#[test]
fn test_mapping_generated_to_source() {
    let mapping = SourceMapping::new(SourceRange::new(10, 20), SourceRange::new(100, 110));

    assert_eq!(mapping.generated_to_source(100), Some(10));
    assert_eq!(mapping.generated_to_source(105), Some(15));
    assert_eq!(mapping.generated_to_source(109), Some(19));
    assert_eq!(mapping.generated_to_source(99), None);
    assert_eq!(mapping.generated_to_source(110), None);
}

#[test]
fn test_source_map_to_generated() {
    let mut map = SourceMap::new();
    map.add_simple(10, 20, 100, 110);
    map.add_simple(30, 40, 200, 210);

    assert_eq!(map.to_generated(15), Some(105));
    assert_eq!(map.to_generated(35), Some(205));
    assert_eq!(map.to_generated(25), None);
}

#[test]
fn test_source_map_to_source() {
    let mut map = SourceMap::new();
    map.add_simple(10, 20, 100, 110);
    map.add_simple(30, 40, 200, 210);

    assert_eq!(map.to_source(105), Some(15));
    assert_eq!(map.to_source(205), Some(35));
    assert_eq!(map.to_source(150), None);
}

#[test]
fn test_source_map_with_block_offset() {
    let mut map = SourceMap::new();
    map.set_block_offset(50); // Template starts at offset 50 in SFC
    map.add_simple(10, 20, 100, 110);

    assert_eq!(map.to_source(105), Some(65)); // 15 + 50
}
