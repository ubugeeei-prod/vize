use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::embed::syntax::{EmbedHole, ProgramOptions, parse_program_once};
use vize_l1::embed::{EmbedSource, Lang};
use vize_l2::file::FileIssueKind;
use vize_l2::lang::js::{FileProducer, ProgramInput, ProgramScope};

#[test]
fn actual_nonfatal_recovery_cannot_supply_a_public_file_program_input() {
    let arena = Allocator::default();
    let source = "const value = /x/uv;";
    let observed = Parser::new(&arena, source, SourceType::mjs()).parse_observed();
    assert!(!observed.panicked());
    assert_eq!(observed.diagnostics().len(), 1);
    assert!(observed.admitted().is_none());
    let syntax = parse_program_once(
        &arena,
        EmbedSource::authored(source, Span::new(0, source.len() as u32)).unwrap(),
        ProgramOptions::module(Lang::Js),
    );
    assert_eq!(syntax.hole(), Some(EmbedHole::Syntax));
    assert!(syntax.admitted_program().is_none());
    assert_eq!(syntax.diagnostics().count(), 1);
    assert_eq!(syntax.source().text(), source);
}

#[test]
fn original_native_owner_supplies_the_live_short_admission_and_keeps_observations() {
    let arena = Allocator::default();
    let source = "é<script>/* keep */ const 日本語 = 1;</script>";
    let start = source.find("/* keep */").unwrap();
    let end = source.find("</script>").unwrap();
    let span = Span::new(start as u32, end as u32);
    let block = SourceRoot::new(source)
        .unwrap()
        .block(source.get(start..end).unwrap(), start as u32)
        .unwrap();
    let syntax = parse_program_once(
        &arena,
        EmbedSource::authored(source, span).unwrap(),
        ProgramOptions::module(Lang::Js),
    );
    let actual_ast = syntax.program().unwrap();
    assert!(core::ptr::eq(
        syntax.admitted_program().unwrap().program(),
        actual_ast
    ));
    let mut producer = FileProducer::new(&arena, source).unwrap();
    let unit = producer
        .program(
            ProgramInput::checked(syntax.admitted_program().unwrap(), block, 4).unwrap(),
            ProgramScope::Module,
        )
        .unwrap();
    let file = producer.finish().unwrap();
    assert!(file.is_complete());
    assert_eq!(unit.index(), 4);
    let declaration = file.bindings().next().unwrap().declaration().unwrap();
    assert_eq!(declaration.name, "日本語");
    assert_eq!(
        declaration.span.start as usize,
        source.find("日本語").unwrap()
    );
    assert_eq!(syntax.comments().count(), 1);
    assert_eq!(syntax.diagnostics().count(), 0);
    assert!(core::ptr::eq(syntax.program().unwrap(), actual_ast));
}

#[test]
fn nondefault_actual_options_cannot_be_substituted_with_a_compiler_profile() {
    let arena = Allocator::default();
    let source = "const value = 1;";
    let block = SourceRoot::new(source).unwrap().whole_block();
    for options in [
        ParseOptions {
            enable_ident_hashes: false,
            ..ParseOptions::default()
        },
        ParseOptions {
            allow_return_outside_function: true,
            ..ParseOptions::default()
        },
    ] {
        let observed = Parser::new(&arena, source, SourceType::mjs())
            .with_options(options)
            .parse_observed();
        assert!(observed.diagnostics().is_empty());
        assert!(matches!(
            ProgramInput::checked(observed.admitted().unwrap(), block, 0),
            Err(error) if error.kind == FileIssueKind::InvalidProfile
        ));
    }
}

#[test]
fn inferred_module_does_not_replace_the_original_unambiguous_profile() {
    let arena = Allocator::default();
    let source = "export const value = 1;";
    let observed = Parser::new(&arena, source, SourceType::unambiguous()).parse_observed();
    let admitted = observed.admitted().unwrap();
    assert!(admitted.program().source_type.is_module());
    assert!(admitted.source_type().is_unambiguous());
    assert!(matches!(
        ProgramInput::checked(admitted, SourceRoot::new(source).unwrap().whole_block(), 0),
        Err(error) if error.kind == FileIssueKind::InvalidProfile
    ));
}
