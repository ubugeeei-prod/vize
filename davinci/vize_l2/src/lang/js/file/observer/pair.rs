//! Compose read-only observers in the same walk. The producer owns rollback;
//! each observer receives its own checkpoint before either observes the subtree.

use super::{
    CallEvent, DeclaredEvent, FileObserver, InvocationEvent, ResolutionErrorKind, ScriptUnit,
    StatementEvent, SyntaxEvent,
};

impl<'a, A: FileObserver<'a>, B: FileObserver<'a>> FileObserver<'a> for (A, B) {
    type Checkpoint = (A::Checkpoint, B::Checkpoint);

    fn unit(&mut self, unit: &ScriptUnit) {
        self.0.unit(unit);
        self.1.unit(unit);
    }
    fn statement(&mut self, event: StatementEvent<'_, 'a>) {
        self.0.statement(StatementEvent { ..event });
        self.1.statement(event);
    }
    fn declared(&mut self, event: DeclaredEvent<'_, 'a>) {
        self.0.declared(DeclaredEvent { ..event });
        self.1.declared(event);
    }
    fn checkpoint(&self) -> Self::Checkpoint {
        (self.0.checkpoint(), self.1.checkpoint())
    }
    fn call(&mut self, event: CallEvent<'_, 'a>) -> Result<(), ResolutionErrorKind> {
        self.0.call(CallEvent { ..event })?;
        self.1.call(event)
    }
    fn invocation(&mut self, event: InvocationEvent<'_, 'a>) -> Result<(), ResolutionErrorKind> {
        self.0.invocation(InvocationEvent { ..event })?;
        self.1.invocation(event)
    }
    fn syntax(&mut self, event: SyntaxEvent<'a>) -> Result<(), ResolutionErrorKind> {
        self.0.syntax(SyntaxEvent { ..event })?;
        self.1.syntax(event)
    }
    fn rollback(&mut self, (first, second): Self::Checkpoint) {
        self.1.rollback(second);
        self.0.rollback(first);
    }
}

#[cfg(test)]
mod tests;
