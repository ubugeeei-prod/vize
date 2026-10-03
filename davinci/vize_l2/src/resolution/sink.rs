//! Internal reference events; binding policy runs in the owning sink.

use alloc::vec::Vec;
use vize_l0::Span;

use super::{BindingLookup, Occurrence, ResolutionError, ResolutionErrorKind, Usage, walk};
use crate::expr::JsExpr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReferenceEvent<'a> {
    span: Span,
    name: &'a str,
    usage: Usage,
    shorthand: bool,
    constructor: bool,
}

impl<'a> ReferenceEvent<'a> {
    pub(super) fn new(
        span: Span,
        name: &'a str,
        usage: Usage,
        shorthand: bool,
        constructor: bool,
    ) -> Self {
        Self {
            span,
            name,
            usage,
            shorthand,
            constructor,
        }
    }

    #[must_use]
    pub(crate) const fn span(self) -> Span {
        self.span
    }
    #[must_use]
    pub(crate) const fn name(self) -> &'a str {
        self.name
    }
    #[must_use]
    pub(crate) const fn usage(self) -> Usage {
        self.usage
    }
    #[must_use]
    pub(crate) const fn shorthand(self) -> bool {
        self.shorthand
    }
    #[must_use]
    pub(crate) const fn constructor(self) -> bool {
        self.constructor
    }
}

pub(crate) trait ReferenceSink<'a> {
    type Checkpoint;
    fn checkpoint(&self) -> Self::Checkpoint;
    fn reference(&mut self, event: ReferenceEvent<'a>) -> Result<(), ResolutionErrorKind>;
    fn observe_syntax(
        &mut self,
        _kind: super::SyntaxKind<'a>,
        _edge: super::SyntaxEdge,
        _span: Span,
    ) -> Result<(), ResolutionErrorKind> {
        Ok(())
    }
    fn observe_invocation(
        &mut self,
        _expression: &oxc_ast::ast::Expression<'a>,
        _span: Span,
    ) -> Result<(), ResolutionErrorKind> {
        Ok(())
    }
    fn observe_call(
        &mut self,
        _call: &oxc_ast::ast::CallExpression<'a>,
        _span: Span,
    ) -> Result<(), ResolutionErrorKind> {
        Ok(())
    }
    fn rollback(&mut self, checkpoint: Self::Checkpoint);
}

pub(super) fn resolve<'a>(
    expression: &JsExpr<'a>,
    bindings: &impl BindingLookup,
) -> Result<Vec<Occurrence<'a>>, ResolutionError> {
    let mut sink = Immediate {
        bindings,
        occurrences: Vec::new(),
    };
    walk::expression(expression, &mut sink)?;
    Ok(sink.occurrences)
}

struct Immediate<'a, 'b, B> {
    bindings: &'b B,
    occurrences: Vec<Occurrence<'a>>,
}

impl<'a, B: BindingLookup> ReferenceSink<'a> for Immediate<'a, '_, B> {
    type Checkpoint = usize;
    fn checkpoint(&self) -> usize {
        self.occurrences.len()
    }
    fn reference(&mut self, event: ReferenceEvent<'a>) -> Result<(), ResolutionErrorKind> {
        let binding = self
            .bindings
            .lookup(event.name())
            .ok_or(ResolutionErrorKind::MissingBinding)?;
        self.occurrences.push(Occurrence {
            span: event.span(),
            name: event.name(),
            binding,
            usage: event.usage(),
            shorthand: event.shorthand(),
            constructor: event.constructor(),
        });
        Ok(())
    }
    fn rollback(&mut self, checkpoint: usize) {
        self.occurrences.truncate(checkpoint);
    }
}
