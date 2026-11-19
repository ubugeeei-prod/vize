use super::syntax;
use crate::module_source_cases::{lower, observe};
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot};
use vize_l1::embed::Lang;
use vize_l2::file::FileIssueKind;
use vize_l2::lang::js::{FileProducer, ModuleSourceErrorKind, ProgramInput, ProgramScope};

#[test]
fn jsx_profile_views_reject_wrong_profile_original_reparse_and_equal_foreign_buffer() {
    let arena = Allocator::default();
    let source = "import 'dep';";
    let block = SourceRoot::new(source).unwrap().whole_block();
    let original = observe(&arena, block, SourceType::mjs().with_jsx(true));
    let file = lower(&arena, &original, block, 0);
    for profile in [SourceType::mjs(), SourceType::tsx().with_module(true)] {
        let foreign = observe(&arena, block, profile);
        assert!(
            matches!(file.original_module_sources(ProgramInput::checked(foreign.admitted().unwrap(), block, 0).unwrap()), Err(error) if error.kind == ModuleSourceErrorKind::Profile)
        );
    }
    let reparsed = observe(&arena, block, SourceType::mjs().with_jsx(true));
    assert!(
        matches!(file.original_module_sources(ProgramInput::checked(reparsed.admitted().unwrap(), block, 0).unwrap()), Err(error) if error.kind == ModuleSourceErrorKind::ProgramOrigin)
    );
    let foreign_source = vize_l0::String::from(source);
    let foreign_block = SourceRoot::new(foreign_source.as_str())
        .unwrap()
        .whole_block();
    let foreign = observe(&arena, foreign_block, SourceType::mjs().with_jsx(true));
    assert!(
        matches!(file.original_module_sources(ProgramInput::checked(foreign.admitted().unwrap(), foreign_block, 0).unwrap()), Err(error) if error.kind == ModuleSourceErrorKind::Source)
    );
}

#[test]
fn unsupported_jsx_and_tsx_subtrees_refuse_before_any_static_source_prefix() {
    for (source, lang) in [
        ("import 'before'; const view = <svg:path/>;", Lang::Js),
        (
            "import Widget from 'before'; const view = <Widget<number>/>;",
            Lang::Ts,
        ),
    ] {
        let arena = Allocator::default();
        let block = SourceRoot::new(source).unwrap().whole_block();
        let original = syntax(&arena, block, lang).unwrap();
        assert_eq!(original.diagnostics().count(), 0);
        let mut producer = FileProducer::new(&arena, source).unwrap();
        producer
            .program(
                ProgramInput::checked(original.admitted_program().unwrap(), block, 0).unwrap(),
                ProgramScope::Module,
            )
            .unwrap();
        let file = producer.finish().unwrap();
        assert!(!file.is_complete());
        assert!(
            file.issues()
                .iter()
                .any(|issue| issue.kind == FileIssueKind::UnsupportedSyntax)
        );
        assert!(
            matches!(file.original_module_sources(ProgramInput::checked(original.admitted_program().unwrap(), block, 0).unwrap()), Err(error) if error.kind == ModuleSourceErrorKind::IncompleteFile)
        );
    }
}

#[test]
fn genuine_jsx_dynamic_import_and_empty_reexport_still_refuse_whole_module_view() {
    for (source, expected) in [
        (
            "import Widget from 'before'; const view = <Widget>{import('dynamic')}</Widget>;",
            ModuleSourceErrorKind::DynamicImport,
        ),
        (
            "import Widget from 'before'; const view = <Widget/>; export {} from 'unrepresented';",
            ModuleSourceErrorKind::EmptySourceExport,
        ),
    ] {
        let arena = Allocator::default();
        let block = SourceRoot::new(source).unwrap().whole_block();
        let original = syntax(&arena, block, Lang::Js).unwrap();
        let mut producer = FileProducer::new(&arena, source).unwrap();
        producer
            .program(
                ProgramInput::checked(original.admitted_program().unwrap(), block, 0).unwrap(),
                ProgramScope::Module,
            )
            .unwrap();
        let file = producer.finish().unwrap();
        assert!(file.is_complete());
        assert!(
            matches!(file.original_module_sources(ProgramInput::checked(original.admitted_program().unwrap(), block, 0).unwrap()), Err(error) if error.kind == expected)
        );
    }
}

#[test]
fn jsx_empty_body_and_lone_surrogate_keep_original_typed_refusals() {
    for (source, expected) in [
        (
            "/* original empty JSX */",
            ModuleSourceErrorKind::EmptyProgram,
        ),
        (
            "import '\\uD800'; const view = <div/>;",
            ModuleSourceErrorKind::LoneSurrogateRequest,
        ),
    ] {
        let arena = Allocator::default();
        let block = SourceRoot::new(source).unwrap().whole_block();
        let original = syntax(&arena, block, Lang::Js).unwrap();
        let mut producer = FileProducer::new(&arena, source).unwrap();
        producer
            .program(
                ProgramInput::checked(original.admitted_program().unwrap(), block, 0).unwrap(),
                ProgramScope::Module,
            )
            .unwrap();
        let file = producer.finish().unwrap();
        assert!(file.is_complete());
        match file.original_module_sources(
            ProgramInput::checked(original.admitted_program().unwrap(), block, 0).unwrap(),
        ) {
            Err(error) => assert_eq!(error.kind, expected),
            Ok(view) => {
                let mut visited = 0;
                let error = view.for_each(|_| visited += 1).unwrap_err();
                assert_eq!(error.kind, expected);
                assert_eq!(visited, 0);
            }
        }
    }
}

#[test]
fn recovered_jsx_syntax_cannot_supply_original_program_admission() {
    let arena = Allocator::default();
    let source = "/* kept */ import Widget from 'before'; const view = <Widget></Other>;";
    let block = SourceRoot::new(source).unwrap().whole_block();
    let original = syntax(&arena, block, Lang::Js).unwrap();
    assert!(original.admitted_program().is_none());
    assert!(original.program().is_none());
    assert!(original.diagnostics().count() > 0);
    assert_eq!(original.comments().count(), 1);
    assert_eq!(original.source().text(), source);
}
