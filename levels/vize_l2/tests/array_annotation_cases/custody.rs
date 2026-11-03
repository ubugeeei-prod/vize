use super::{
    Allocator, FileIssueKind, FileProducer, Parser, PositionQueryError, ProgramInput, ProgramScope,
    ScriptUnit, SourceRoot, SourceType, String, lower,
};
use oxc_parser::ParseOptions;
use vize_l2::lang::js::{CallEvent, DeclaredEvent, FileObserver, StatementEvent};
use vize_l2::resolution::ResolutionErrorKind;

#[test]
fn array_annotations_keep_source_options_and_original_module_authority() {
    let arena = Allocator::default();
    let source = String::from("const items:number[]=[];");
    let copied = source.clone();
    let block = SourceRoot::new(&source).unwrap().whole_block();
    let original =
        Parser::new(&arena, &source, SourceType::ts().with_module(true)).parse_observed();
    assert!(matches!(
        ProgramInput::checked(
            original.admitted().unwrap(),
            SourceRoot::new(&copied).unwrap().whole_block(),
            7,
        ),
        Err(issue) if issue.kind == FileIssueKind::InvalidSource
    ));
    let changed = Parser::new(&arena, &source, SourceType::ts().with_module(true))
        .with_options(ParseOptions {
            enable_ident_hashes: false,
            ..ParseOptions::default()
        })
        .parse_observed();
    assert!(matches!(
        ProgramInput::checked(changed.admitted().unwrap(), block, 7),
        Err(issue) if issue.kind == FileIssueKind::InvalidProfile
    ));
    for profile in [
        SourceType::ts().with_script(true),
        SourceType::tsx().with_script(true),
    ] {
        let observed = Parser::new(&arena, &source, profile).parse_observed();
        assert!(observed.diagnostics().is_empty());
        let file = lower(&arena, &source, block, &observed);
        assert!(!file.is_complete());
        assert!(
            file.issues()
                .iter()
                .any(|issue| issue.kind == FileIssueKind::InvalidProfile)
        );
    }
    let ambient = "let items:number[];";
    let observed =
        Parser::new(&arena, ambient, SourceType::d_ts().with_module(true)).parse_observed();
    assert!(observed.diagnostics().is_empty());
    let file = lower(
        &arena,
        ambient,
        SourceRoot::new(ambient).unwrap().whole_block(),
        &observed,
    );
    assert!(!file.is_complete());
    assert!(
        file.issues()
            .iter()
            .any(|issue| issue.kind == FileIssueKind::InvalidProfile)
    );
}

struct Interrupt;

impl<'a> FileObserver<'a> for Interrupt {
    type Checkpoint = ();
    fn unit(&mut self, _: &ScriptUnit) {}
    fn statement(&mut self, _: StatementEvent<'_, 'a>) {}
    fn declared(&mut self, _: DeclaredEvent<'_, 'a>) {
        panic!("actual array declaration event");
    }
    fn checkpoint(&self) {}
    fn call(&mut self, _: CallEvent<'_, 'a>) -> Result<(), ResolutionErrorKind> {
        Ok(())
    }
    fn rollback(&mut self, _: ()) {}
}

#[test]
fn interrupted_original_array_event_never_completes_the_file() {
    let arena = Allocator::default();
    let source = "const items:{id:number}[]=[{id:1}];";
    let observed = Parser::new(&arena, source, SourceType::ts().with_module(true)).parse_observed();
    let mut producer = FileProducer::new(&arena, source).unwrap();
    let block = SourceRoot::new(source).unwrap().whole_block();
    let interrupted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        producer
            .program_observed(
                ProgramInput::checked(observed.admitted().unwrap(), block, 7).unwrap(),
                ProgramScope::Module,
                &mut Interrupt,
            )
            .unwrap();
    }));
    assert!(interrupted.is_err());
    let file = producer.finish().unwrap();
    assert!(!file.is_complete());
    assert_eq!(file.interrupted_programs().count(), 1);
    assert_eq!(file.bindings().count(), 1);
    assert!(matches!(
        file.binding_at_offset(source.find("items").unwrap() as u32),
        Err(PositionQueryError::IncompleteFile)
    ));
}
