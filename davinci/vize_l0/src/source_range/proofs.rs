//! Kani checks the shipped `SourceRange`, `SourceMapping` and `SourceMap`
//! offset arithmetic against the statements in their documentation: a range
//! is the half-open set of offsets `[start, end)`, `union` is the smallest
//! range whose endpoints enclose both, `intersection` is the offsets in both,
//! and a mapping carries an offset by its distance from the range start,
//! clamped to the last offset of the other side.
//!
//! The range and mapping harnesses have no loop, so every endpoint, offset
//! and shift ranges over its whole `u32` or `i32` domain and the check is
//! exhaustive. The `SourceMap` lookup is bounded to three mappings of any
//! shape: empty, inverted and overlapping source ranges included.
use super::{SourceMap, SourceMapping, SourceRange};

fn any_range() -> SourceRange {
    SourceRange::new(kani::any(), kani::any())
}

/// `outer`'s endpoints enclose `inner`'s.
fn encloses(outer: SourceRange, inner: SourceRange) -> bool {
    outer.start <= inner.start && inner.end <= outer.end
}

fn covers(range: Option<SourceRange>, offset: u32) -> bool {
    range.is_some_and(|range| range.start <= offset && offset < range.end)
}

#[kani::proof]
fn source_range_union_is_the_least_enclosing_range() {
    let (a, b, c) = (any_range(), any_range(), any_range());
    let union = a.union(&b);
    assert_eq!(union, b.union(&a));
    assert_eq!(a.union(&a), a);
    assert_eq!(union.union(&c), a.union(&b.union(&c)));
    assert!(encloses(union, a) && encloses(union, b));
    // Every range that encloses both encloses the union.
    assert!(!(encloses(c, a) && encloses(c, b)) || encloses(c, union));
    kani::cover!(!a.intersects(&b) && !a.is_empty() && !b.is_empty(), "a gap");
}

#[kani::proof]
fn source_range_intersection_is_the_offsets_in_both() {
    let (a, b, c) = (any_range(), any_range(), any_range());
    let offset: u32 = kani::any();
    let both = a.intersection(&b);
    assert_eq!(
        covers(both, offset),
        covers(Some(a), offset) && covers(Some(b), offset)
    );
    assert_eq!(both.is_some(), a.intersects(&b));
    assert!(both.is_none_or(|range| !range.is_empty() && range.len() <= a.len()));
    assert_eq!(both, b.intersection(&a));
    assert_eq!(
        both.and_then(|both| both.intersection(&c)),
        b.intersection(&c).and_then(|right| a.intersection(&right))
    );
    assert_eq!(a.intersection(&a), (!a.is_empty()).then_some(a));
    assert_eq!(a.is_empty(), a.len() == 0);
    kani::cover!(
        both.is_some() && both != Some(a) && both != Some(b),
        "a proper overlap"
    );
}

#[kani::proof]
fn source_range_shifts_saturate_and_invert() {
    let range = any_range();
    let amount: i32 = kani::any();
    let shifted = range.offset(amount);
    // Each endpoint moves by `amount` in the integers, clamped into `u32`.
    let moved = |offset: u32| i64::from(offset) + i64::from(amount);
    let clamped = |offset: u32| moved(offset).clamp(0, i64::from(u32::MAX));
    assert_eq!(i64::from(shifted.start), clamped(range.start));
    assert_eq!(i64::from(shifted.end), clamped(range.end));
    let fits = |offset: u32| moved(offset) == clamped(offset);
    // Without saturation, the opposite shift restores the range.
    if fits(range.start) && fits(range.end) {
        assert_eq!(shifted.len(), range.len());
        if let Some(back) = amount.checked_neg() {
            assert_eq!(shifted.offset(back), range);
        }
    }
    let grown = range.extend(kani::any());
    assert!(encloses(grown, range));
    kani::cover!(
        amount < 0 && shifted.start == 0 && moved(range.start) < 0,
        "clamped at zero"
    );
    kani::cover!(
        amount > 0 && shifted.end == u32::MAX && moved(range.end) > i64::from(u32::MAX),
        "clamped at the top"
    );
    kani::cover!(
        amount == i32::MIN && fits(range.start),
        "the most negative shift"
    );
}

#[kani::proof]
fn source_mapping_carries_offsets_between_its_ranges() {
    let mapping = SourceMapping::new(any_range(), any_range());
    let (source, generated) = (mapping.source, mapping.generated);
    let offset: u32 = kani::any();

    let forward = mapping.source_to_generated(offset);
    assert_eq!(forward.is_some(), covers(Some(source), offset));
    let backward = mapping.generated_to_source(offset);
    assert_eq!(backward.is_some(), covers(Some(generated), offset));
    if let Some(target) = forward
        && !generated.is_empty()
    {
        assert!(covers(Some(generated), target));
        let distance = offset - source.start;
        assert_eq!(target - generated.start, distance.min(generated.len() - 1));
    }
    // An equal-length mapping is a bijection between its two ranges.
    if source.len() == generated.len() {
        if let Some(target) = forward {
            assert_eq!(mapping.generated_to_source(target), Some(offset));
        }
        if let Some(origin) = backward {
            assert_eq!(mapping.source_to_generated(origin), Some(offset));
        }
    }
    kani::cover!(
        forward.is_some() && generated.len() < source.len() && offset + 1 == source.end,
        "an offset clamped to a shorter generated range"
    );
    kani::cover!(
        forward.is_some() && source.len() == generated.len() && source.start != generated.start,
        "a bijection between moved ranges"
    );
}

#[kani::proof]
#[kani::unwind(5)]
fn source_map_to_generated_uses_the_mapping_that_contains_the_offset() {
    // Any three mappings, in any insertion order: empty, inverted and
    // overlapping source ranges included.
    let mappings: [SourceMapping; 3] =
        core::array::from_fn(|_| SourceMapping::new(any_range(), any_range()));
    let offset: u32 = kani::any();
    // By the documentation: the map keeps its mappings in source order, and
    // a lookup translates through the first mapping in that order whose
    // source range holds the offset. Source order is the smallest start,
    // then the earliest insertion among equal starts.
    let mut chosen: Option<usize> = None;
    for (index, mapping) in mappings.iter().enumerate() {
        let holds = mapping.source.start <= offset && offset < mapping.source.end;
        let earlier =
            chosen.is_some_and(|best| mappings[best].source.start <= mapping.source.start);
        if holds && !earlier {
            chosen = Some(index);
        }
    }
    let expected = chosen.and_then(|index| mappings[index].source_to_generated(offset));
    let map = SourceMap::from_mappings(mappings.to_vec());
    let found = map.to_generated(offset);
    assert_eq!(found, expected);
    kani::cover!(
        found.is_some()
            && mappings
                .iter()
                .any(|mapping| mapping.source.start > mapping.source.end),
        "an inverted range beside the mapping that holds the offset"
    );
    kani::cover!(
        found.is_some()
            && mappings
                .iter()
                .any(|mapping| mapping.source.is_empty() && mapping.source.start <= offset),
        "an empty range sorted before the offset"
    );
    kani::cover!(
        found.is_some()
            && mappings[0].contains_source(offset)
            && mappings[1].contains_source(offset),
        "overlapping mappings"
    );
    kani::cover!(found.is_none(), "an offset no mapping holds");
}
