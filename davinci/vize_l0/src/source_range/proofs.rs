//! Kani checks the shipped `SourceMap::to_generated` lookup against the
//! first mapping in source order that contains the offset. `SourceMap` sorts
//! with `sort_by_key` and searches with `find_map`, which Creusot 0.14 does
//! not specify, so this harness stays here. Creusot proves the range and
//! mapping laws the lookup is built on (`tests/formal/creusot/coordinates`).
//! The lookup is bounded to three mappings of any shape: empty, inverted and
//! overlapping source ranges included. Every endpoint and offset ranges over
//! the whole `u32` domain.
use super::{SourceMap, SourceMapping, SourceRange};

fn any_range() -> SourceRange {
    SourceRange::new(kani::any(), kani::any())
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

#[kani::proof]
#[kani::unwind(4)]
fn source_map_to_source_shifts_into_the_file_without_overflow() {
    // Any two mappings and any block offset. Kani's arithmetic checks
    // fail the harness on any overflow inside the lookup.
    let mappings: [SourceMapping; 2] =
        core::array::from_fn(|_| SourceMapping::new(any_range(), any_range()));
    let gen_offset: u32 = kani::any();
    let block_offset: u32 = kani::any();
    let mut map = SourceMap::from_mappings(mappings.to_vec());
    map.set_block_offset(block_offset);
    // By the documentation: the first mapping, in the map's order, whose
    // generated range holds the offset carries it into the block; the block
    // offset moves it into the file, and a file offset past `u32::MAX` has no
    // `u32` coordinate.
    let in_block = map
        .mappings()
        .iter()
        .find(|mapping| mapping.generated.start <= gen_offset && gen_offset < mapping.generated.end)
        .and_then(|mapping| mapping.generated_to_source(gen_offset));
    let expected = in_block
        .map(|offset| u64::from(offset) + u64::from(block_offset))
        .and_then(|file| u32::try_from(file).ok());
    let found = map.to_source(gen_offset);
    assert_eq!(found, expected);
    kani::cover!(
        in_block.is_some() && found.is_none(),
        "a block offset that carries the source past u32::MAX"
    );
    kani::cover!(
        found.is_some() && block_offset > 0,
        "a shifted source offset"
    );
    kani::cover!(in_block.is_none(), "an offset no mapping holds");
}
