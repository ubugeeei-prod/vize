use super::observe;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot};
use vize_l2::file::{FileIssueKind, ScriptUnit};
use vize_l2::lang::js::{
    CallEvent, DeclaredEvent, FileObserver, FileProducer, InvocationEvent, ModuleSourceErrorKind,
    ProgramInput, ProgramScope, StatementEvent,
};
use vize_l2::resolution::ResolutionErrorKind;

enum Fault {
    Declaration,
    Invocation,
    RejectInvocation,
}
struct Observer(Fault);
impl<'a> FileObserver<'a> for Observer {
    type Checkpoint = ();
    fn unit(&mut self, _: &ScriptUnit) {}
    fn statement(&mut self, _: StatementEvent<'_, 'a>) {}
    fn declared(&mut self, _: DeclaredEvent<'_, 'a>) {
        if matches!(self.0, Fault::Declaration) {
            std::panic::resume_unwind(Box::new(()));
        }
    }
    fn checkpoint(&self) {}
    fn call(&mut self, _: CallEvent<'_, 'a>) -> Result<(), ResolutionErrorKind> {
        Ok(())
    }
    fn invocation(&mut self, _: InvocationEvent<'_, 'a>) -> Result<(), ResolutionErrorKind> {
        match self.0 {
            Fault::Invocation => std::panic::resume_unwind(Box::new(())),
            Fault::RejectInvocation => Err(ResolutionErrorKind::UnsupportedSyntax),
            Fault::Declaration => Ok(()),
        }
    }
    fn rollback(&mut self, _: ()) {}
}

#[test]
fn original_import_rows_and_dynamic_observations_survive_unwind_without_completion() {
    for (source, fault) in [
        ("/* kept */ import {value} from 'dep';", Fault::Declaration),
        (
            "/* kept */ import 'before'; import('dynamic');",
            Fault::Invocation,
        ),
    ] {
        let arena = Allocator::default();
        let block = SourceRoot::new(source).unwrap().whole_block();
        let original = observe(&arena, block, SourceType::mjs());
        let input = ProgramInput::checked(original.admitted().unwrap(), block, 0).unwrap();
        let mut producer = FileProducer::new(&arena, source).unwrap();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            producer.program_observed(input, ProgramScope::Module, &mut Observer(fault))
        }));
        assert!(result.is_err());
        let file = producer.finish().unwrap();
        assert_eq!(
            file.imports().len(),
            1,
            "parked original import row survives callback unwind"
        );
        assert_eq!(original.comments().len(), 1);
        assert!(original.diagnostics().is_empty());
        assert_eq!(file.interrupted_programs().count(), 1);
        assert!(file.issues().is_empty());
        assert!(
            matches!(file.original_module_sources(ProgramInput::checked(original.admitted().unwrap(), block, 0).unwrap()), Err(error) if error.kind == ModuleSourceErrorKind::IncompleteFile)
        );
    }
}

#[test]
fn rejected_original_dynamic_callback_keeps_refusal_after_resolution_rollback() {
    let arena = Allocator::default();
    let source = "import 'before'; import('dynamic');";
    let block = SourceRoot::new(source).unwrap().whole_block();
    let original = observe(&arena, block, SourceType::mjs());
    let mut producer = FileProducer::new(&arena, source).unwrap();
    producer
        .program_observed(
            ProgramInput::checked(original.admitted().unwrap(), block, 0).unwrap(),
            ProgramScope::Module,
            &mut Observer(Fault::RejectInvocation),
        )
        .unwrap();
    let file = producer.finish().unwrap();
    assert!(
        file.issues()
            .iter()
            .any(|issue| issue.kind == FileIssueKind::UnsupportedSyntax)
    );
    assert!(file.references().is_empty());
    assert_eq!(file.imports().len(), 1);
    assert!(file.units()[0].has_invocations());
    assert!(
        matches!(file.original_module_sources(ProgramInput::checked(original.admitted().unwrap(), block, 0).unwrap()), Err(error) if error.kind == ModuleSourceErrorKind::IncompleteFile)
    );
}
