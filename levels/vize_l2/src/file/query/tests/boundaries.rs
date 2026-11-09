use super::{
    Allocator, Error, FileProducer, ProgramScope, SourceRoot, SourceType, Span, append, offset,
};

#[test]
fn authored_utf8_boundaries_nonzero_origins_and_half_open_ends_are_checked() {
    let arena = Allocator::default();
    let source = "😀 opaque\nconst é = 1; é;\nopaque";
    let start = offset(source, "const");
    let end = offset(source, "\nopaque");
    let block = SourceRoot::new(source)
        .unwrap()
        .block(source.get(start as usize..end as usize).unwrap(), start)
        .unwrap();
    let mut producer = FileProducer::new(&arena, source).unwrap();
    append(
        &arena,
        &mut producer,
        block,
        7,
        SourceType::mjs(),
        ProgramScope::Nested,
    );
    let owner = producer.finish().unwrap();
    assert!(owner.is_complete());
    let name = offset(source, "é =");
    let declaration = owner.binding_at_offset(name).unwrap().unwrap();
    assert_eq!(
        declaration.declaration().unwrap().span,
        Span::new(name, name + 2)
    );
    assert!(matches!(
        owner.binding_at_offset(name + 1),
        Err(Error::NotCharBoundary)
    ));
    assert!(matches!(
        owner.reference_at_offset(1),
        Err(Error::NotCharBoundary)
    ));
    assert!(owner.binding_at_offset(name + 2).unwrap().is_none());
    let use_at = source.rfind('é').unwrap() as u32;
    assert_eq!(
        owner
            .reference_at_offset(use_at)
            .unwrap()
            .unwrap()
            .reference()
            .span,
        Span::new(use_at, use_at + 2)
    );
    assert!(owner.reference_at_offset(use_at + 2).unwrap().is_none());
    assert!(owner.scope_at_offset(start).unwrap().is_some());
    for at in [0, start - 1, end, source.len() as u32] {
        assert!(owner.scope_at_offset(at).unwrap().is_none());
        assert!(owner.binding_at_offset(at).unwrap().is_none());
        assert!(owner.reference_at_offset(at).unwrap().is_none());
    }
    let invalid = source.len() as u32 + 1;
    assert!(matches!(
        owner.scope_at_offset(invalid),
        Err(Error::OutOfBounds)
    ));
    assert!(matches!(
        owner.binding_at_offset(invalid),
        Err(Error::OutOfBounds)
    ));
    assert!(matches!(
        owner.reference_at_offset(invalid),
        Err(Error::OutOfBounds)
    ));
}

#[test]
fn units_can_be_recorded_in_reverse_source_order_without_crossing_opaque_gaps() {
    let arena = Allocator::default();
    let source = "const earlier = 1; earlier;\nopaque\nconst later = 2; later;";
    let root = SourceRoot::new(source).unwrap();
    let first_end = offset(source, "\nopaque");
    let second_start = offset(source, "const later");
    let mut producer = FileProducer::new(&arena, source).unwrap();
    for (start, end, index) in [(second_start, source.len() as u32, 9), (0, first_end, 4)] {
        append(
            &arena,
            &mut producer,
            root.block(source.get(start as usize..end as usize).unwrap(), start)
                .unwrap(),
            index,
            SourceType::mjs(),
            ProgramScope::Nested,
        );
    }
    let owner = producer.finish().unwrap();
    assert!(owner.is_complete());
    for (needle, unit) in [("earlier =", 4), ("later =", 9)] {
        let binding = owner
            .binding_at_offset(offset(source, needle))
            .unwrap()
            .unwrap();
        assert_eq!(binding.declaration().unwrap().unit.index(), unit);
        assert_eq!(
            owner
                .scope_at_offset(offset(source, needle))
                .unwrap()
                .unwrap()
                .scope()
                .id,
            binding.declaration().unwrap().scope
        );
    }
    assert!(
        owner
            .scope_at_offset(offset(source, "opaque"))
            .unwrap()
            .is_none()
    );
}

#[test]
fn overlapping_original_unit_observations_are_ambiguous_even_on_a_non_name_byte() {
    let arena = Allocator::default();
    let source = "const local = 1; local;";
    let block = SourceRoot::new(source).unwrap().whole_block();
    let mut producer = FileProducer::new(&arena, source).unwrap();
    for index in [3, 8] {
        append(
            &arena,
            &mut producer,
            block,
            index,
            SourceType::mjs(),
            ProgramScope::Nested,
        );
    }
    let owner = producer.finish().unwrap();
    assert!(owner.is_complete());
    assert_eq!(owner.units().len(), 2);
    for at in [
        0,
        offset(source, "local ="),
        source.rfind("local").unwrap() as u32,
    ] {
        assert!(matches!(
            owner.scope_at_offset(at),
            Err(Error::AmbiguousSite)
        ));
        assert!(matches!(
            owner.binding_at_offset(at),
            Err(Error::AmbiguousSite)
        ));
        assert!(matches!(
            owner.reference_at_offset(at),
            Err(Error::AmbiguousSite)
        ));
    }
}
