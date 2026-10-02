//! Read-only policy hooks in the existing declaration/reference traversal.

use crate::file::{Declaration, ScopeId, ScriptUnit, ScriptUnitId};
use crate::resolution::ResolutionErrorKind;
use oxc_ast::ast::{CallExpression, Expression, Statement};
use vize_l0::Span;

pub struct StatementEvent<'s, 'a> {
    pub unit: ScriptUnitId,
    pub scope: ScopeId,
    pub span: Span,
    pub statement: &'s Statement<'a>,
}
pub struct DeclaredEvent<'s, 'a> {
    pub declaration: &'s Declaration,
    pub initializer: Option<(&'s Expression<'a>, Span)>,
}
pub struct CallEvent<'s, 'a> {
    pub unit: ScriptUnitId,
    pub scope: ScopeId,
    pub span: Span,
    pub call: &'s CallExpression<'a>,
}

/// Diagnostic observation grants no native file admission or insertion capability.
pub trait FileObserver<'a> {
    type Checkpoint;
    fn unit(&mut self, unit: &ScriptUnit);
    fn statement(&mut self, event: StatementEvent<'_, 'a>);
    fn declared(&mut self, event: DeclaredEvent<'_, 'a>);
    fn checkpoint(&self) -> Self::Checkpoint;
    fn call(&mut self, event: CallEvent<'_, 'a>) -> Result<(), ResolutionErrorKind>;
    fn rollback(&mut self, checkpoint: Self::Checkpoint);
}

pub(crate) struct NoObserver;
impl<'a> FileObserver<'a> for NoObserver {
    type Checkpoint = ();
    fn unit(&mut self, _: &ScriptUnit) {}
    fn statement(&mut self, _: StatementEvent<'_, 'a>) {}
    fn declared(&mut self, _: DeclaredEvent<'_, 'a>) {}
    fn checkpoint(&self) {}
    fn call(&mut self, _: CallEvent<'_, 'a>) -> Result<(), ResolutionErrorKind> {
        Ok(())
    }
    fn rollback(&mut self, _: ()) {}
}
