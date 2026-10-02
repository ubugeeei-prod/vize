use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot};
use vize_l2::file::{DeclarationKind, FileIssueKind, ScopeId, ScriptUnit};
use vize_l2::lang::js::{
    CallEvent, DeclaredEvent, FileObserver, FileProducer, ProgramInput, ProgramScope,
    StatementEvent,
};
use vize_l2::resolution::ResolutionErrorKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Point {
    Unit,
    Statement,
    Function,
    Parameter,
    Call,
}
struct Observer {
    point: Point,
    failure: fn(),
    scope: Option<ScopeId>,
}
impl Observer {
    fn observe(&mut self, point: Point, scope: ScopeId) {
        if point == self.point {
            self.scope = Some(scope);
            (self.failure)();
        }
    }
}
impl<'a> FileObserver<'a> for Observer {
    type Checkpoint = ();
    fn unit(&mut self, unit: &ScriptUnit) {
        self.observe(Point::Unit, unit.scope);
    }
    fn statement(&mut self, event: StatementEvent<'_, 'a>) {
        self.observe(Point::Statement, event.scope);
    }
    fn declared(&mut self, event: DeclaredEvent<'_, 'a>) {
        match event.declaration.kind {
            DeclarationKind::Function => self.observe(Point::Function, event.declaration.scope),
            DeclarationKind::Parameter => self.observe(Point::Parameter, event.declaration.scope),
            _ => {}
        }
    }
    fn checkpoint(&self) {}
    fn call(&mut self, event: CallEvent<'_, 'a>) -> Result<(), ResolutionErrorKind> {
        self.observe(Point::Call, event.scope);
        Ok(())
    }
    fn rollback(&mut self, _: ()) {}
}

#[test]
fn caught_observer_unwind_keeps_original_program_and_refuses_every_interrupted_unit_site() {
    let arena = Allocator::default();
    let source = "é<script>/* original */ function show(value) { return show(value); } const after = 1;</script>";
    let start = source.find("/*").unwrap();
    let end = source.find("</script>").unwrap();
    let block = SourceRoot::new(source)
        .unwrap()
        .block(source.get(start..end).unwrap(), start as u32)
        .unwrap();
    let parsed = Parser::new(&arena, block.source(), SourceType::mjs()).parse_observed();
    let program = parsed.admitted().unwrap().program();
    for point in [
        Point::Unit,
        Point::Statement,
        Point::Function,
        Point::Parameter,
        Point::Call,
    ] {
        let mut producer = FileProducer::new(&arena, source).unwrap();
        let mut observer = Observer {
            point,
            failure: || std::panic::resume_unwind(Box::new("actual observer interruption")),
            scope: None,
        };
        let interrupted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            producer.program_observed(
                ProgramInput::checked(parsed.admitted().unwrap(), block, 5).unwrap(),
                ProgramScope::Module,
                &mut observer,
            )
        }));
        assert!(interrupted.is_err(), "{point:?}");
        assert_eq!(producer.interrupted_programs().count(), 1);
        let file = producer.finish().unwrap();
        assert!(!file.is_complete(), "{point:?}");
        assert_eq!(file.units().len(), 1);
        assert_eq!(file.units()[0].id.index(), 5);
        assert!(file.issues().is_empty());
        let issue = file.units()[0].interruption().unwrap();
        assert_eq!(issue.kind, FileIssueKind::InterruptedProgram);
        assert_eq!(issue.unit, file.units()[0].id);
        assert_eq!(issue.span, block.span());
        assert_eq!(file.interrupted_programs().count(), 1);
        assert!(
            file.scopes()
                .iter()
                .any(|scope| Some(scope.id) == observer.scope)
        );
        assert!(core::ptr::eq(file.artifact().source(), source));
        assert!(core::ptr::eq(parsed.admitted().unwrap().program(), program));
        assert_eq!(program.body.len(), 2);
        assert_eq!(program.comments.len(), 1);
        assert!(parsed.diagnostics().is_empty());
        assert!(
            file.bindings()
                .all(|binding| binding.declaration().unwrap().name != "after")
        );
    }
    let mut producer = FileProducer::new(&arena, source).unwrap();
    producer
        .program(
            ProgramInput::checked(parsed.admitted().unwrap(), block, 5).unwrap(),
            ProgramScope::Module,
        )
        .unwrap();
    let complete = producer.finish().unwrap();
    assert!(complete.is_complete());
    assert!(complete.issues().is_empty());
    assert_eq!(complete.interrupted_programs().count(), 0);
    assert_eq!(complete.bindings().count(), 3);
    assert_eq!(complete.references().len(), 2);
}
