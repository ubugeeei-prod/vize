use super::{lower, observe};
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot};
use vize_l2::file::FileIssueKind;
use vize_l2::lang::js::{FileProducer, ModuleSourceErrorKind, ProgramInput, ProgramScope};

#[test]
fn equal_text_foreign_allocations_and_same_buffer_reparses_cannot_replace_original_body() {
    let arena = Allocator::default();
    let source = std::string::String::from("import 'same';");
    let foreign = std::string::String::from(source.as_str());
    let block = SourceRoot::new(&source).unwrap().whole_block();
    let original = observe(&arena, block, SourceType::mjs());
    let file = lower(&arena, &original, block, 0);
    let foreign_block = SourceRoot::new(&foreign).unwrap().whole_block();
    let foreign_owner = observe(&arena, foreign_block, SourceType::mjs());
    assert!(
        matches!(file.original_module_sources(ProgramInput::checked(foreign_owner.admitted().unwrap(), foreign_block, 0).unwrap()), Err(error) if error.kind == ModuleSourceErrorKind::Source)
    );
    assert!(
        matches!(ProgramInput::checked(foreign_owner.admitted().unwrap(), block, 0), Err(error) if error.kind == FileIssueKind::InvalidSource)
    );
    let reparsed = observe(&arena, block, SourceType::mjs());
    assert!(
        matches!(file.original_module_sources(ProgramInput::checked(reparsed.admitted().unwrap(), block, 0).unwrap()), Err(error) if error.kind == ModuleSourceErrorKind::ProgramOrigin)
    );
    let other_arena = Allocator::default();
    let foreign_arena_owner = observe(&other_arena, block, SourceType::mjs());
    assert!(
        matches!(file.original_module_sources(ProgramInput::checked(foreign_arena_owner.admitted().unwrap(), block, 0).unwrap()), Err(error) if error.kind == ModuleSourceErrorKind::ProgramOrigin)
    );
    let foreign_file = lower(&arena, &reparsed, block, 0);
    assert_eq!(file.units()[0].id, foreign_file.units()[0].id);
    assert!(
        matches!(foreign_file.original_module_sources(ProgramInput::checked(original.admitted().unwrap(), block, 0).unwrap()), Err(error) if error.kind == ModuleSourceErrorKind::ProgramOrigin)
    );
}

#[test]
fn real_sibling_units_and_root_extents_do_not_join_by_content_or_numeric_id() {
    let arena = Allocator::default();
    let source = "<one>import 'same';</one><two>import 'same';</two>";
    let root = SourceRoot::new(source).unwrap();
    let first_start = source.find("import").unwrap();
    let second_start = source.rfind("import").unwrap();
    let text_len = "import 'same';".len();
    let first = root
        .block(
            source.get(first_start..first_start + text_len).unwrap(),
            first_start as u32,
        )
        .unwrap();
    let second = root
        .block(
            source.get(second_start..second_start + text_len).unwrap(),
            second_start as u32,
        )
        .unwrap();
    let original = observe(&arena, first, SourceType::mjs());
    let sibling = observe(&arena, second, SourceType::mjs());
    let mut producer = FileProducer::new(&arena, source).unwrap();
    for (owner, block, index) in [(&original, first, 2), (&sibling, second, 5)] {
        producer
            .program(
                ProgramInput::checked(owner.admitted().unwrap(), block, index).unwrap(),
                ProgramScope::Module,
            )
            .unwrap();
    }
    let file = producer.finish().unwrap();
    assert!(file.is_complete());
    assert!(
        matches!(file.original_module_sources(ProgramInput::checked(sibling.admitted().unwrap(), second, 2).unwrap()), Err(error) if error.kind == ModuleSourceErrorKind::Source)
    );
    assert!(
        matches!(file.original_module_sources(ProgramInput::checked(original.admitted().unwrap(), first, 9).unwrap()), Err(error) if error.kind == ModuleSourceErrorKind::MissingUnit)
    );
    let smaller_root = SourceRoot::new(first.source()).unwrap().whole_block();
    assert!(
        matches!(file.original_module_sources(ProgramInput::checked(original.admitted().unwrap(), smaller_root, 2).unwrap()), Err(error) if error.kind == ModuleSourceErrorKind::Source)
    );
    let mut count = 0;
    for (owner, block, index) in [(&original, first, 2), (&sibling, second, 5)] {
        file.original_module_sources(
            ProgramInput::checked(owner.admitted().unwrap(), block, index).unwrap(),
        )
        .unwrap()
        .for_each(|row| {
            assert_eq!(row.decoded_request(), "same");
            assert!(block.span().start <= row.quoted_span().start);
            assert!(row.quoted_span().end <= block.span().end);
            count += 1;
        })
        .unwrap();
    }
    assert_eq!(count, 2);
}

#[test]
fn duplicate_unit_refusal_preserves_original_rows_and_cannot_mint_complete_sources() {
    let arena = Allocator::default();
    let source = "import 'a'; import 'a';";
    let block = SourceRoot::new(source).unwrap().whole_block();
    let original = observe(&arena, block, SourceType::mjs());
    let mut producer = FileProducer::new(&arena, source).unwrap();
    producer
        .program(
            ProgramInput::checked(original.admitted().unwrap(), block, 0).unwrap(),
            ProgramScope::Module,
        )
        .unwrap();
    assert!(
        matches!(producer.program(ProgramInput::checked(original.admitted().unwrap(), block, 0).unwrap(), ProgramScope::Module), Err(error) if error.kind == FileIssueKind::DuplicateUnit)
    );
    let file = producer.finish().unwrap();
    assert_eq!(file.imports().len(), 2);
    assert_eq!(file.units().len(), 1);
    assert!(
        matches!(file.original_module_sources(ProgramInput::checked(original.admitted().unwrap(), block, 0).unwrap()), Err(error) if error.kind == ModuleSourceErrorKind::IncompleteFile)
    );
    let clean = lower(&arena, &original, block, 0);
    let mut previous = None;
    let mut count = 0;
    clean
        .original_module_sources(
            ProgramInput::checked(original.admitted().unwrap(), block, 0).unwrap(),
        )
        .unwrap()
        .for_each(|row| {
            assert_eq!(row.decoded_request(), "a");
            assert_ne!(Some(row.quoted_span()), previous);
            previous = Some(row.quoted_span());
            count += 1;
        })
        .unwrap();
    assert_eq!(
        count, 2,
        "distinct authored literals are not deduplicated by decoded request"
    );
}
