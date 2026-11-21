use super::*;
use crate::lang::js::{FileProducer, ProgramInput, ProgramScope};
use alloc::vec::Vec;
use core::cell::RefCell;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot, Span};

struct Events<'f> {
    label: u8,
    fail: bool,
    rows: Vec<(&'static str, Span)>,
    rollback_order: &'f RefCell<Vec<u8>>,
}

impl<'a> FileObserver<'a> for Events<'_> {
    type Checkpoint = usize;
    fn unit(&mut self, unit: &ScriptUnit) {
        self.rows.push(("unit", unit.span));
    }
    fn statement(&mut self, event: StatementEvent<'_, 'a>) {
        self.rows.push(("statement", event.span));
    }
    fn declared(&mut self, event: DeclaredEvent<'_, 'a>) {
        self.rows.push(("declared", event.declaration.span));
    }
    fn checkpoint(&self) -> usize {
        self.rows.len()
    }
    fn syntax(&mut self, event: SyntaxEvent<'a>) -> Result<(), ResolutionErrorKind> {
        self.rows.push(("syntax", event.span));
        Ok(())
    }
    fn call(&mut self, event: CallEvent<'_, 'a>) -> Result<(), ResolutionErrorKind> {
        self.rows.push(("call", event.span));
        if self.fail {
            Err(ResolutionErrorKind::UnsupportedSyntax)
        } else {
            Ok(())
        }
    }
    fn invocation(&mut self, event: InvocationEvent<'_, 'a>) -> Result<(), ResolutionErrorKind> {
        self.rows.push(("invocation", event.span));
        Ok(())
    }
    fn rollback(&mut self, checkpoint: usize) {
        self.rollback_order.borrow_mut().push(self.label);
        self.rows.truncate(checkpoint);
    }
}

#[test]
fn both_observers_see_the_same_events_and_rollback_their_own_subtree() {
    let arena = Allocator::default();
    let source = "import C from 'module'; const x = C(1); new C(); <C/>;";
    let root = SourceRoot::new(source).unwrap();
    let syntax = Parser::new(&arena, source, SourceType::jsx().with_module(true)).parse_observed();
    for rejecting in [None, Some(0), Some(1)] {
        let rollback_order = RefCell::default();
        let observer = |label| Events {
            label,
            fail: rejecting == Some(label),
            rows: Vec::new(),
            rollback_order: &rollback_order,
        };
        let mut pair = (observer(0), observer(1));
        let mut producer = FileProducer::new(&arena, source).unwrap();
        producer
            .program_observed(
                ProgramInput::checked(syntax.admitted().unwrap(), root.whole_block(), 0).unwrap(),
                ProgramScope::Module,
                &mut pair,
            )
            .unwrap();
        let file = producer.finish().unwrap();
        assert_eq!(pair.0.rows, pair.1.rows);
        for family in ["unit", "declared", "statement", "invocation", "syntax"] {
            assert!(
                pair.0.rows.iter().any(|(name, _)| *name == family),
                "{family}"
            );
        }
        assert_eq!(
            pair.0.rows.iter().any(|(name, _)| *name == "call"),
            rejecting.is_none()
        );
        assert_eq!(file.is_complete(), rejecting.is_none());
        if rejecting.is_some() {
            assert_eq!(*rollback_order.borrow(), [1, 0]);
        } else {
            assert!(rollback_order.borrow().is_empty());
        }
    }
}
