//! Read-only policy hooks in the existing declaration/reference traversal.

use crate::file::{Declaration, ScopeId, ScriptUnit, ScriptUnitId};
use crate::resolution::ResolutionErrorKind;
use crate::resolution::{SyntaxEdge, SyntaxKind};
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

/// Original constructor, tagged template or dynamic import in the existing walk.
/// This diagnostic event grants no factory or native admission capability.
pub struct InvocationEvent<'s, 'a> {
    pub unit: ScriptUnitId,
    pub scope: ScopeId,
    pub span: Span,
    pub expression: &'s Expression<'a>,
}

/// Checked original-walk structure and the real pending reference boundary.
/// This event grants no owner construction or semantic insertion capability.
pub struct SyntaxEvent<'a> {
    pub unit: ScriptUnitId,
    pub scope: ScopeId,
    pub span: Span,
    pub kind: SyntaxKind<'a>,
    pub edge: SyntaxEdge,
    pub reference_boundary: usize,
}

/// Diagnostic observation grants no native file admission or insertion capability.
pub trait FileObserver<'a> {
    type Checkpoint;
    fn unit(&mut self, unit: &ScriptUnit);
    fn statement(&mut self, event: StatementEvent<'_, 'a>);
    fn declared(&mut self, event: DeclaredEvent<'_, 'a>);
    fn checkpoint(&self) -> Self::Checkpoint;
    fn call(&mut self, event: CallEvent<'_, 'a>) -> Result<(), ResolutionErrorKind>;
    fn invocation(&mut self, _: InvocationEvent<'_, 'a>) -> Result<(), ResolutionErrorKind> {
        Ok(())
    }
    fn syntax(&mut self, _: SyntaxEvent<'a>) -> Result<(), ResolutionErrorKind> {
        Ok(())
    }
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
