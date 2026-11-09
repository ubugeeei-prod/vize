use oxc_ast::ast::{Expression, Statement};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l2::file::{FileIssueKind, ScopeId, ScriptUnit, ScriptUnitId};
use vize_l2::lang::js::{
    CallEvent, DeclaredEvent, FileObserver, FileProducer, InvocationEvent, ProgramInput,
    ProgramScope, StatementEvent,
};
use vize_l2::resolution::ResolutionErrorKind;

const SOURCES: [&str; 3] = [
    "é<script>/* original */ function Factory() {} let value = new Factory(); let after = 1;</script>",
    "é<script>/* original */ function tag() { return 1; } let value = tag`text`; let after = 1;</script>",
    "é<script>/* original */ let value = import('dep'); let after = 1;</script>",
];

struct Observer {
    fail: Option<fn()>,
    reject: bool,
    event: Option<(usize, Span, ScriptUnitId, ScopeId)>,
    rollback: usize,
}
impl<'a> FileObserver<'a> for Observer {
    type Checkpoint = ();
    fn unit(&mut self, _: &ScriptUnit) {}
    fn statement(&mut self, _: StatementEvent<'_, 'a>) {}
    fn declared(&mut self, _: DeclaredEvent<'_, 'a>) {}
    fn checkpoint(&self) {}
    fn call(&mut self, _: CallEvent<'_, 'a>) -> Result<(), ResolutionErrorKind> {
        Ok(())
    }
    fn invocation(&mut self, event: InvocationEvent<'_, 'a>) -> Result<(), ResolutionErrorKind> {
        self.event = Some((
            event.expression as *const Expression<'a> as usize,
            event.span,
            event.unit,
            event.scope,
        ));
        if let Some(fail) = self.fail {
            fail();
        }
        if self.reject {
            Err(ResolutionErrorKind::UnsupportedSyntax)
        } else {
            Ok(())
        }
    }
    fn rollback(&mut self, _: ()) {
        self.rollback += 1;
    }
}

#[test]
fn actual_invocation_callback_unwind_retains_originals_and_interrupts_the_real_unit() {
    let arena = Allocator::default();
    for source in SOURCES {
        let start = source.find("/*").unwrap();
        let end = source.find("</script>").unwrap();
        let block = SourceRoot::new(source)
            .unwrap()
            .block(source.get(start..end).unwrap(), start as u32)
            .unwrap();
        let parsed = Parser::new(&arena, block.source(), SourceType::mjs()).parse_observed();
        let original = parsed.admitted().unwrap().program();
        let expression = original
            .body
            .iter()
            .find_map(|statement| {
                let Statement::VariableDeclaration(variable) = statement else {
                    return None;
                };
                variable
                    .declarations
                    .first()
                    .and_then(|declaration| declaration.init.as_ref())
            })
            .unwrap();
        let mut observer = Observer {
            fail: Some(|| std::panic::resume_unwind(Box::new("actual invocation interruption"))),
            reject: false,
            event: None,
            rollback: 0,
        };
        let mut producer = FileProducer::new(&arena, source).unwrap();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            producer.program_observed(
                ProgramInput::checked(parsed.admitted().unwrap(), block, 5).unwrap(),
                ProgramScope::Nested,
                &mut observer,
            )
        }));
        assert!(result.is_err());
        let file = producer.finish().unwrap();
        assert!(!file.is_complete());
        let [unit] = file.units() else {
            panic!("one original unit");
        };
        let issue = unit.interruption().unwrap();
        assert_eq!(issue.kind, FileIssueKind::InterruptedProgram);
        assert_eq!(issue.unit, unit.id);
        assert_eq!(issue.span, block.span());
        let (pointer, span, id, scope) = observer.event.unwrap();
        assert_eq!(pointer, expression as *const Expression<'_> as usize);
        assert_eq!(
            span,
            Span::new(
                start as u32 + expression.span().start,
                start as u32 + expression.span().end
            )
        );
        assert_eq!(id, unit.id);
        assert_eq!(scope, unit.scope);
        assert!(
            file.bindings()
                .any(|binding| binding.declaration().unwrap().name == "value")
        );
        assert!(
            file.bindings()
                .all(|binding| binding.declaration().unwrap().name != "after")
        );
        assert!(core::ptr::eq(
            parsed.admitted().unwrap().program(),
            original
        ));
        assert!(core::ptr::eq(file.artifact().source(), source));
        assert_eq!(original.comments.len(), 1);
        assert!(parsed.diagnostics().is_empty());
    }
}

#[test]
fn rejected_invocation_rolls_back_the_same_subtree_references_before_normal_unit_closure() {
    let arena = Allocator::default();
    for suffix in ["new Factory()", "tag`text`", "import('dep')"] {
        let source = vize_l0::cstr!(
            "function Factory() {{}} function tag() {{}} let known = 1; let value = known + {suffix};"
        );
        let parsed = Parser::new(&arena, &source, SourceType::mjs()).parse_observed();
        let mut producer = FileProducer::new(&arena, &source).unwrap();
        let mut observer = Observer {
            fail: None,
            reject: true,
            event: None,
            rollback: 0,
        };
        producer
            .program_observed(
                ProgramInput::checked(
                    parsed.admitted().unwrap(),
                    SourceRoot::new(&source).unwrap().whole_block(),
                    0,
                )
                .unwrap(),
                ProgramScope::Nested,
                &mut observer,
            )
            .unwrap();
        let file = producer.finish().unwrap();
        assert!(!file.is_complete());
        assert!(file.references().is_empty());
        assert!(file.units()[0].interruption().is_none());
        assert_eq!(file.issues()[0].kind, FileIssueKind::UnsupportedSyntax);
        assert_eq!(observer.rollback, 1);
        let (_, span, unit, scope) = observer.event.unwrap();
        assert_eq!(unit, file.units()[0].id);
        assert_eq!(scope, file.units()[0].scope);
        assert!(span.end <= source.len() as u32);
    }
}
