use super::{lower, observe};
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot};
use vize_l2::lang::js::{ModuleSourceErrorKind, ProgramInput};

#[test]
fn every_real_empty_reexport_and_dynamic_import_gap_refuses_before_static_prefix() {
    for (source, expected) in [
        (
            "import 'before'; export {} from 'unrepresented';",
            ModuleSourceErrorKind::EmptySourceExport,
        ),
        (
            "import 'before'; import('dynamic');",
            ModuleSourceErrorKind::DynamicImport,
        ),
        (
            "import 'before'; function run() { return import('nested'); }",
            ModuleSourceErrorKind::DynamicImport,
        ),
    ] {
        let arena = Allocator::default();
        let block = SourceRoot::new(source).unwrap().whole_block();
        let original = observe(&arena, block, SourceType::mjs());
        let file = lower(&arena, &original, block, 0);
        assert_eq!(file.imports().len(), 1);
        if expected == ModuleSourceErrorKind::EmptySourceExport {
            assert!(
                file.exports().is_empty(),
                "no fabricated source/export binding row"
            );
        }
        let mut visited = 0;
        match file.original_module_sources(
            ProgramInput::checked(original.admitted().unwrap(), block, 0).unwrap(),
        ) {
            Err(error) => assert_eq!(error.kind, expected),
            Ok(view) => {
                view.for_each(|_| visited += 1).unwrap();
                panic!("uncovered source family admitted")
            }
        }
        assert_eq!(visited, 0);
    }
}

#[test]
fn lone_surrogate_requests_refuse_without_publishing_a_late_partial_result() {
    let arena = Allocator::default();
    let source = "import 'before'; import '\\uD800';";
    let block = SourceRoot::new(source).unwrap().whole_block();
    let original = observe(&arena, block, SourceType::mjs());
    let file = lower(&arena, &original, block, 0);
    let view = file
        .original_module_sources(
            ProgramInput::checked(original.admitted().unwrap(), block, 0).unwrap(),
        )
        .unwrap();
    let mut visited = 0;
    let result = view.for_each(|_| visited += 1);
    assert!(
        matches!(result, Err(error) if error.kind == ModuleSourceErrorKind::LoneSurrogateRequest)
    );
    assert_eq!(
        visited, 1,
        "on error the consumer must discard the visited prefix"
    );
    assert_eq!(original.diagnostics().len(), 0);
    assert_eq!(
        file.imports().len(),
        2,
        "typed read refusal preserves ordinary source rows"
    );
}

#[test]
fn authentic_nonempty_local_programs_have_no_invented_module_source() {
    for source in [
        "const lone = 1;",
        "/* no dependencies */ const local = 1;",
        "const local = 1; export {local}; export default 2;",
    ] {
        let arena = Allocator::default();
        let block = SourceRoot::new(source).unwrap().whole_block();
        let original = observe(&arena, block, SourceType::mjs());
        let file = lower(&arena, &original, block, 0);
        let mut count = 0;
        file.original_module_sources(
            ProgramInput::checked(original.admitted().unwrap(), block, 0).unwrap(),
        )
        .unwrap()
        .for_each(|_| count += 1)
        .unwrap();
        assert_eq!(count, 0);
    }
}

#[test]
fn actual_body_empty_programs_refuse_without_fabricating_owner_identity() {
    for source in [
        "",
        "/* source retained */",
        "\"use strict\";",
        "#!/usr/bin/env node\n",
    ] {
        let arena = Allocator::default();
        let block = SourceRoot::new(source).unwrap().whole_block();
        let original = observe(&arena, block, SourceType::mjs());
        assert!(original.admitted().unwrap().program().body.is_empty());
        let file = lower(&arena, &original, block, 0);
        assert!(
            matches!(file.original_module_sources(ProgramInput::checked(original.admitted().unwrap(), block, 0).unwrap()), Err(error) if error.kind == ModuleSourceErrorKind::EmptyProgram)
        );
        assert!(file.imports().is_empty());
        assert!(file.exports().is_empty());
        assert!(
            file.is_complete(),
            "neutral completion is preserved without original read admission"
        );
        assert!(original.diagnostics().is_empty());
    }
}

#[test]
fn same_buffer_and_foreign_arena_empty_reparses_share_sentinel_but_never_admit() {
    for (source, comment_count) in [("", 0), ("/* kept */", 1), ("\"use strict\";", 0)] {
        let arena = Allocator::default();
        let foreign_arena = Allocator::default();
        let block = SourceRoot::new(source).unwrap().whole_block();
        let original = observe(&arena, block, SourceType::mjs());
        let same_buffer = observe(&arena, block, SourceType::mjs());
        let foreign = observe(&foreign_arena, block, SourceType::mjs());
        let file = lower(&arena, &original, block, 0);
        let pointer = original.admitted().unwrap().program().body.as_ptr();
        for owner in [&same_buffer, &foreign] {
            assert_eq!(
                owner.admitted().unwrap().program().body.as_ptr(),
                pointer,
                "pinned empty Vec uses shared dangling storage"
            );
            assert!(
                matches!(file.original_module_sources(ProgramInput::checked(owner.admitted().unwrap(), block, 0).unwrap()), Err(error) if error.kind == ModuleSourceErrorKind::EmptyProgram)
            );
        }
        assert!(file.is_complete());
        assert_eq!(file.units().len(), 1);
        assert_eq!(original.comments().len(), comment_count);
    }
}
