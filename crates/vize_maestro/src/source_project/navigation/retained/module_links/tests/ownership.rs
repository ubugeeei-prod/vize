use super::{
    Allocator, EmbedSource, Lang, ModuleOperandRefusal, ModuleSourceErrorKind, NavigationRefusal,
    ProgramInput, ProgramOptions, SnapshotRefusal, SourceRoot, collect, parse_program_once, range,
    with_original,
};
use std::cell::Cell;

#[test]
fn module_link_same_file_view_retains_original_occurrence_and_private_source_join() {
    with_original("import './a.ts';", |query, admission| {
        let view = admission.as_ref().unwrap();
        let mut visited = 0;
        view.view
            .for_each(|original| {
                assert!(core::ptr::eq(original.file(), query.file));
                assert!(core::ptr::eq(
                    original.file().artifact().source(),
                    query.snapshot.source()
                ));
                assert!(core::ptr::eq(
                    original.raw(),
                    query.snapshot.source().get(7..15).unwrap()
                ));
                assert_eq!(original.quoted_span(), vize_l0::Span::new(7, 15));
                assert_eq!(original.decoded_request(), "./a.ts");
                visited += 1;
            })
            .unwrap();
        assert_eq!(visited, 1);
        let operands = collect(query, view, || false).unwrap();
        assert_eq!(operands.operands[0].quoted, vize_l0::Span::new(7, 15));
        assert_eq!(operands.operands[0].range, range((0, 7), (0, 15)));
    });
}

#[test]
fn module_link_equal_foreign_file_and_source_cannot_enter_local_owned_reply() {
    for source in ["import './a.ts';", "const value=1;"] {
        with_original(source, |local, _| {
            with_original(source, |foreign, admission| {
                assert_eq!(local.snapshot.source(), foreign.snapshot.source());
                assert!(!core::ptr::eq(
                    local.snapshot.source(),
                    foreign.snapshot.source()
                ));
                assert!(!core::ptr::eq(local.file, foreign.file));
                assert_eq!(
                    collect(local, admission.as_ref().unwrap(), || false),
                    Err(ModuleOperandRefusal::Navigation(
                        NavigationRefusal::Projection
                    ))
                );
            });
        });
    }
}

#[test]
fn module_link_fresh_input_errors_and_same_buffer_reparse_refusals_stay_distinct() {
    with_original("import './a.ts';", |query, _| {
        let arena = Allocator::default();
        let block = SourceRoot::new(query.snapshot.source())
            .unwrap()
            .whole_block();
        let foreign = vize_l0::String::from(query.snapshot.source());
        let foreign_block = SourceRoot::new(foreign.as_str()).unwrap().whole_block();
        let foreign_syntax = parse_program_once(
            &arena,
            EmbedSource::authored(foreign.as_str(), foreign_block.span()).unwrap(),
            ProgramOptions::module(Lang::Js),
        );
        let Err(input_error) =
            ProgramInput::checked(foreign_syntax.admitted_program().unwrap(), block, 0)
        else {
            panic!("foreign allocation must refuse checked input")
        };
        assert!(
            matches!(super::super::admit(&foreign_syntax, block, query.file), Err(ModuleOperandRefusal::Input(error)) if error == input_error)
        );
        assert!(
            matches!(super::super::admit(&foreign_syntax, foreign_block, query.file), Err(ModuleOperandRefusal::Sources(error)) if error.kind == ModuleSourceErrorKind::Source)
        );
        let reparsed = parse_program_once(
            &arena,
            EmbedSource::authored(query.snapshot.source(), block.span()).unwrap(),
            ProgramOptions::module(Lang::Js),
        );
        assert!(
            matches!(super::super::admit(&reparsed, block, query.file), Err(ModuleOperandRefusal::Sources(error)) if error.kind == ModuleSourceErrorKind::ProgramOrigin)
        );
    });
}

#[test]
fn module_link_mid_collection_cancellation_discards_genuine_owned_prefix() {
    with_original("import './a.ts'; import './b.ts';", |query, admission| {
        let calls = Cell::new(0);
        let result = collect(query, admission.as_ref().unwrap(), || {
            let previous = calls.get();
            calls.set(previous + 1);
            previous > 0
        });
        assert_eq!(
            result,
            Err(ModuleOperandRefusal::Navigation(NavigationRefusal::Host(
                SnapshotRefusal::Cancelled
            )))
        );
        assert_eq!(calls.get(), 3);
        assert_eq!(
            collect(query, admission.as_ref().unwrap(), || false)
                .unwrap()
                .len(),
            2
        );
    });
}

#[test]
fn module_link_no_source_collection_still_observes_final_cancellation() {
    with_original("const value=1;", |query, admission| {
        assert_eq!(
            collect(query, admission.as_ref().unwrap(), || true),
            Err(ModuleOperandRefusal::Navigation(NavigationRefusal::Host(
                SnapshotRefusal::Cancelled
            )))
        );
    });
}

#[test]
fn module_link_local_borrowed_join_and_sendable_operands_have_bounded_storage() {
    use super::super::{ModuleOperands, Operand, RetainedSources};
    use core::mem::size_of;
    use vize_l2::{file::FileArtifact, lang::js::OriginalModuleSources};

    assert_eq!(
        size_of::<RetainedSources<'static, 'static, 'static>>(),
        size_of::<OriginalModuleSources<'static, 'static, 'static>>()
            + size_of::<&FileArtifact<'static>>()
    );
    assert_eq!(size_of::<ModuleOperands>(), size_of::<Vec<Operand>>());
    fn require_send<T: Send>() {}
    require_send::<ModuleOperands>();
    require_send::<ModuleOperandRefusal>();
}
