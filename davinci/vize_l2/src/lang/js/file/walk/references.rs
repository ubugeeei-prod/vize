//! Transactional use rows from the shared resolver's sole AST traversal.

use super::{ProgramInput, Walk};
use crate::file::build::Facts;
use crate::file::{FileIssueKind, Namespace, Reference, ReferenceTarget, ScopeId, ScriptUnitId};
use crate::lang::js::file::observer::{CallEvent, FileObserver, InvocationEvent, SyntaxEvent};
use crate::resolution::sink::{ReferenceEvent, ReferenceSink};
use crate::resolution::{ResolutionError, ResolutionErrorKind};
use oxc_ast::ast::{CallExpression, Expression, ModuleExportName};
use vize_l0::{Span, String};

struct Pending<'f, 'p, 'a, O> {
    input: &'f ProgramInput<'p, 'a>,
    facts: &'f mut Facts<'a>,
    unit: ScriptUnitId,
    scope: ScopeId,
    namespace: Namespace,
    observer: &'f mut O,
}

impl<'a, O: FileObserver<'a>> ReferenceSink<'a> for Pending<'_, '_, 'a, O> {
    type Checkpoint = (usize, O::Checkpoint);

    fn checkpoint(&self) -> Self::Checkpoint {
        (self.facts.references.len(), self.observer.checkpoint())
    }

    fn observe_syntax(
        &mut self,
        kind: crate::resolution::SyntaxKind<'a>,
        edge: crate::resolution::SyntaxEdge,
        span: Span,
    ) -> Result<(), ResolutionErrorKind> {
        let span = self
            .input
            .references
            .authored_span(span)
            .ok_or(ResolutionErrorKind::InvalidSpan)?;
        self.observer.syntax(SyntaxEvent {
            unit: self.unit,
            scope: self.scope,
            span,
            kind,
            edge,
            reference_boundary: self.facts.references.len(),
        })
    }

    fn reference(&mut self, event: ReferenceEvent<'a>) -> Result<(), ResolutionErrorKind> {
        let span = self
            .input
            .references
            .authored_span(event.span())
            .ok_or(ResolutionErrorKind::InvalidSpan)?;
        self.facts.references.push(Reference {
            unit: self.unit,
            scope: self.scope,
            name: String::from(event.name()),
            span,
            usage: event.usage(),
            shorthand: event.shorthand(),
            constructor: event.constructor(),
            namespace: self.namespace,
            target: ReferenceTarget::Unresolved,
        });
        Ok(())
    }

    fn observe_invocation(
        &mut self,
        expression: &Expression<'a>,
        span: Span,
    ) -> Result<(), ResolutionErrorKind> {
        let span = self
            .input
            .references
            .authored_span(span)
            .ok_or(ResolutionErrorKind::InvalidSpan)?;
        if let Some(unit) = self
            .facts
            .units
            .iter_mut()
            .find(|unit| unit.id == self.unit)
        {
            unit.origin.has_call = true;
        }
        self.observer.invocation(InvocationEvent {
            unit: self.unit,
            scope: self.scope,
            span,
            expression,
        })
    }

    fn observe_call(
        &mut self,
        call: &CallExpression<'a>,
        span: Span,
    ) -> Result<(), ResolutionErrorKind> {
        let span = self
            .input
            .references
            .authored_span(span)
            .ok_or(ResolutionErrorKind::InvalidSpan)?;
        if let Some(unit) = self
            .facts
            .units
            .iter_mut()
            .find(|unit| unit.id == self.unit)
        {
            unit.origin.has_call = true;
        }
        self.observer.call(CallEvent {
            unit: self.unit,
            scope: self.scope,
            span,
            call,
        })
    }

    fn rollback(&mut self, (references, observer): Self::Checkpoint) {
        self.facts.references.truncate(references);
        self.observer.rollback(observer);
    }
}

impl<'a, O: FileObserver<'a>> Walk<'_, '_, 'a, O> {
    pub(super) fn expression(&mut self, expression: &Expression<'a>) {
        let mut sink = Pending {
            input: self.input,
            facts: self.facts,
            unit: self.unit,
            scope: self.scope,
            namespace: Namespace::Value,
            observer: self.observer,
        };
        if let Err(error) = self.input.references.expression(expression, &mut sink) {
            self.reference_error(error);
        }
    }

    pub(super) fn export_reference(
        &mut self,
        local: &ModuleExportName<'a>,
        namespace: Namespace,
    ) -> Option<usize> {
        let first = self.facts.references.len();
        let mut sink = Pending {
            input: self.input,
            facts: self.facts,
            unit: self.unit,
            scope: self.scope,
            namespace,
            observer: self.observer,
        };
        match self.input.references.export_local(local, &mut sink) {
            Ok(()) => Some(first),
            Err(error) => {
                self.reference_error(error);
                None
            }
        }
    }

    fn reference_error(&mut self, error: ResolutionError) {
        let span = self
            .input
            .references
            .authored_span(error.span)
            .unwrap_or(self.input.block.span());
        self.facts.issue(
            self.unit,
            span,
            match error.kind {
                ResolutionErrorKind::InvalidSpan => FileIssueKind::InvalidSpan,
                ResolutionErrorKind::TraversalLimit => FileIssueKind::BindingLimit,
                ResolutionErrorKind::MissingBinding => FileIssueKind::UnresolvedReference,
                ResolutionErrorKind::UnsupportedSyntax => FileIssueKind::UnsupportedSyntax,
            },
        );
    }
}
