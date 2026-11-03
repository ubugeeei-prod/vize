#![expect(
    clippy::panic_in_result_fn,
    reason = "test assertions report genuine stock-profile and full-source admission refusals"
)]

mod support {
    pub mod file_jsx;
}
use oxc_parser::Parser;
use oxc_span::SourceType;
use support::file_jsx::{LawResult, Required, lower};
use vize_l0::{Allocator, SourceRoot, Span, String};
use vize_l2::file::FileIssueKind;
use vize_l2::lang::js::{FileProducer, ProgramInput, ProgramScope};

#[test]
fn copied_and_foreign_whole_source_are_rejected_before_any_unit_or_binding_mint() -> LawResult {
    let arena = Allocator::default();
    let source = "const Comp = 1; const view = <Comp/>;";
    let parsed = Parser::new(&arena, source, SourceType::jsx()).parse_observed();
    let copy = String::from(source);
    assert_eq!(source, copy.as_str());
    let copied = ProgramInput::checked(
        parsed.admitted().required()?,
        SourceRoot::new(copy.as_str()).required()?.whole_block(),
        0,
    );
    assert_eq!(copied.err().required()?.kind, FileIssueKind::InvalidSource);
    let input = ProgramInput::checked(
        parsed.admitted().required()?,
        SourceRoot::new(source).required()?.whole_block(),
        0,
    )
    .required()?;
    assert_eq!(input.source_type(), SourceType::jsx());
    let mut foreign = FileProducer::new(&arena, copy.as_str()).required()?;
    assert_eq!(
        foreign
            .program(input, ProgramScope::Module)
            .err()
            .required()?
            .kind,
        FileIssueKind::InvalidSource
    );
    let file = foreign.finish().required()?;
    assert_eq!(file.units().len(), 0);
    assert_eq!(file.bindings().count(), 0);
    assert_eq!(file.references().len(), 0);
    Ok(())
}

#[test]
fn short_source_blocks_do_not_rebrand_a_complete_original_jsx_program() -> LawResult {
    let arena = Allocator::default();
    let source = "const Comp = 1; const view = <Comp/>;";
    let parsed = Parser::new(&arena, source, SourceType::jsx()).parse_observed();
    let short = source.get(..14).required()?;
    let block = SourceRoot::new(source)
        .required()?
        .block(short, 0)
        .required()?;
    assert_eq!(
        ProgramInput::checked(parsed.admitted().required()?, block, 0)
            .err()
            .required()?
            .kind,
        FileIssueKind::InvalidSource
    );
    assert!(core::ptr::eq(
        parsed.admitted().required()?.program().source_text,
        source
    ));
    Ok(())
}

#[test]
fn original_nonmodule_jsx_profile_remains_an_explicit_profile_refusal() -> LawResult {
    let arena = Allocator::default();
    let source = "const Comp = 1; const view = <Comp/>;";
    let profile = SourceType::jsx().with_script(true);
    let file = lower(&arena, source, Span::new(0, source.len() as u32), profile)?;
    assert!(!file.is_complete());
    let unit = file.units().first().required()?;
    assert!(unit.profile.jsx && !unit.profile.module);
    assert_eq!(
        file.issues()
            .iter()
            .map(|issue| issue.kind)
            .collect::<Vec<_>>(),
        [FileIssueKind::InvalidProfile]
    );
    assert_eq!(file.bindings().count(), 0);
    assert_eq!(file.references().len(), 0);
    Ok(())
}

#[test]
fn a_genuine_js_profile_cannot_acquire_jsx_admission_from_discarded_parse_errors() -> LawResult {
    let arena = Allocator::default();
    let source = "const Comp = 1; const view = <Comp/>;";
    let parsed = Parser::new(&arena, source, SourceType::mjs()).parse_observed();
    assert!(parsed.admitted().is_none());
    assert!(!parsed.diagnostics().is_empty());
    Ok(())
}

#[test]
fn default_unambiguous_tsx_profile_keeps_its_original_admission_refusal() -> LawResult {
    let arena = Allocator::default();
    let source = "const Comp = 1; const view = <Comp/>;";
    let parsed = Parser::new(&arena, source, SourceType::tsx()).parse_observed();
    let input = ProgramInput::checked(
        parsed.admitted().required()?,
        SourceRoot::new(source).required()?.whole_block(),
        0,
    );
    assert_eq!(input.err().required()?.kind, FileIssueKind::InvalidProfile);
    Ok(())
}
