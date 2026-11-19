//! Deferred facts from a single original handler walk, never an AST second pass.

use super::{
    HandlerBinding, HandlerBindingRef, HandlerDeclaration, HandlerDeclarationKind, HandlerLocalId,
    HandlerReference, HandlerScope, HandlerScopeId, HandlerSyntax,
};
use crate::resolution::{
    BindingLookup, ResolutionError, ResolutionErrorKind,
    sink::{ReferenceEvent, ReferenceSink},
};
use alloc::vec::Vec;
use vize_l0::Span;

pub(in crate::resolution) trait HandlerScopeSink<'a>:
    ReferenceSink<'a>
{
    fn observe_body(&mut self, leading_declaration: bool);
    fn observe_return(&mut self, span: Span);
    fn enter_block(&mut self, span: Span) -> Result<(), ResolutionErrorKind>;
    fn leave_block(&mut self);
    fn declare(
        &mut self,
        name: &'a str,
        span: Span,
        kind: HandlerDeclarationKind,
    ) -> Result<(), ResolutionErrorKind>;
}

#[derive(Debug)]
pub(super) struct Tables<'a> {
    pub syntax: HandlerSyntax,
    pub scopes: Vec<HandlerScope>,
    pub bindings: Vec<HandlerBinding<'a>>,
    pub declarations: Vec<HandlerDeclaration<'a>>,
    pub references: Vec<HandlerReference<'a>>,
}

pub(super) struct Pending<'a> {
    syntax: HandlerSyntax,
    scopes: Vec<HandlerScope>,
    bindings: Vec<HandlerBinding<'a>>,
    declarations: Vec<HandlerDeclaration<'a>>,
    pending: Vec<(HandlerScopeId, ReferenceEvent<'a>)>,
    current: HandlerScopeId,
}

pub(super) struct Checkpoint {
    syntax: HandlerSyntax,
    scopes: usize,
    bindings: usize,
    declarations: usize,
    pending: usize,
    current: HandlerScopeId,
}

impl<'a> Pending<'a> {
    pub(super) fn new() -> Self {
        let root = HandlerScopeId(0);
        let scopes = alloc::vec![HandlerScope {
            id: root,
            parent: None,
            span: None
        }];
        let bindings = alloc::vec![HandlerBinding {
            id: HandlerLocalId(0),
            scope: root,
            name: "$event",
            kind: HandlerDeclarationKind::EventParameter,
        }];
        Self {
            syntax: HandlerSyntax {
                leading_declaration: false,
                first_return: None,
            },
            scopes,
            bindings,
            declarations: Vec::new(),
            pending: Vec::new(),
            current: root,
        }
    }

    fn parent(&self, scope: HandlerScopeId) -> Option<HandlerScopeId> {
        self.scopes.get(scope.0 as usize)?.parent
    }

    fn named(&self, scope: HandlerScopeId, name: &str) -> Option<HandlerBinding<'a>> {
        self.bindings
            .iter()
            .find(|binding| binding.scope == scope && binding.name == name)
            .copied()
    }

    fn within(&self, mut scope: HandlerScopeId, ancestor: HandlerScopeId) -> bool {
        loop {
            if scope == ancestor {
                return true;
            }
            let Some(parent) = self.parent(scope) else {
                return false;
            };
            scope = parent;
        }
    }

    fn local(&self, mut scope: HandlerScopeId, name: &str) -> Option<HandlerLocalId> {
        loop {
            if let Some(binding) = self.named(scope, name) {
                return Some(binding.id);
            }
            scope = self.parent(scope)?;
        }
    }

    pub(super) fn finish(self, outer: &impl BindingLookup) -> Result<Tables<'a>, ResolutionError> {
        let mut references = Vec::with_capacity(self.pending.len());
        for (scope, event) in &self.pending {
            let binding = self
                .local(*scope, event.name())
                .map(HandlerBindingRef::Local)
                .or_else(|| outer.lookup(event.name()).map(HandlerBindingRef::Outer))
                .ok_or(ResolutionError {
                    span: event.span(),
                    kind: ResolutionErrorKind::MissingBinding,
                })?;
            references.push(HandlerReference {
                scope: *scope,
                span: event.span(),
                name: event.name(),
                binding,
                usage: event.usage(),
                shorthand: event.shorthand(),
                constructor: event.constructor(),
            });
        }
        Ok(Tables {
            syntax: self.syntax,
            scopes: self.scopes,
            bindings: self.bindings,
            declarations: self.declarations,
            references,
        })
    }
}

impl<'a> ReferenceSink<'a> for Pending<'a> {
    type Checkpoint = Checkpoint;
    fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            syntax: self.syntax,
            scopes: self.scopes.len(),
            bindings: self.bindings.len(),
            declarations: self.declarations.len(),
            pending: self.pending.len(),
            current: self.current,
        }
    }
    fn reference(&mut self, event: ReferenceEvent<'a>) -> Result<(), ResolutionErrorKind> {
        self.pending.push((self.current, event));
        Ok(())
    }
    fn rollback(&mut self, checkpoint: Checkpoint) {
        self.syntax = checkpoint.syntax;
        self.scopes.truncate(checkpoint.scopes);
        self.bindings.truncate(checkpoint.bindings);
        self.declarations.truncate(checkpoint.declarations);
        self.pending.truncate(checkpoint.pending);
        self.current = checkpoint.current;
    }
}

impl<'a> HandlerScopeSink<'a> for Pending<'a> {
    fn observe_body(&mut self, leading_declaration: bool) {
        self.syntax.leading_declaration = leading_declaration;
    }
    fn observe_return(&mut self, span: Span) {
        self.syntax.first_return.get_or_insert(span);
    }
    fn enter_block(&mut self, span: Span) -> Result<(), ResolutionErrorKind> {
        let id = HandlerScopeId(
            u32::try_from(self.scopes.len()).map_err(|_| ResolutionErrorKind::TraversalLimit)?,
        );
        self.scopes.push(HandlerScope {
            id,
            parent: Some(self.current),
            span: Some(span),
        });
        self.current = id;
        Ok(())
    }

    fn leave_block(&mut self) {
        if let Some(parent) = self.parent(self.current) {
            self.current = parent;
        }
    }

    fn declare(
        &mut self,
        name: &'a str,
        span: Span,
        kind: HandlerDeclarationKind,
    ) -> Result<(), ResolutionErrorKind> {
        let target = if kind == HandlerDeclarationKind::Var {
            HandlerScopeId(0)
        } else {
            self.current
        };
        let existing = self.named(target, name);
        if kind == HandlerDeclarationKind::Var {
            let mut scope = self.current;
            loop {
                if self.named(scope, name).is_some_and(|binding| {
                    matches!(
                        binding.kind,
                        HandlerDeclarationKind::Let | HandlerDeclarationKind::Const
                    )
                }) {
                    return Err(ResolutionErrorKind::UnsupportedSyntax);
                }
                let Some(parent) = self.parent(scope) else {
                    break;
                };
                scope = parent;
            }
        } else if kind == HandlerDeclarationKind::EventParameter
            || existing.is_some()
            || self.declarations.iter().any(|declaration| {
                declaration.name == name
                    && declaration.kind == HandlerDeclarationKind::Var
                    && self.within(declaration.scope, self.current)
            })
        {
            return Err(ResolutionErrorKind::UnsupportedSyntax);
        }
        let id = if let Some(existing) = existing {
            existing.id
        } else {
            let id = HandlerLocalId(
                u32::try_from(self.bindings.len())
                    .map_err(|_| ResolutionErrorKind::TraversalLimit)?,
            );
            self.bindings.push(HandlerBinding {
                id,
                scope: target,
                name,
                kind,
            });
            id
        };
        self.declarations.push(HandlerDeclaration {
            binding: id,
            scope: self.current,
            name,
            kind,
            span,
        });
        Ok(())
    }
}
