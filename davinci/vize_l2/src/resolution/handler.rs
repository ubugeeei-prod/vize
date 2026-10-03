//! Runtime reference identities from the entire original event handler body.

use super::{BindingLookup, ResolutionError, walk};
use crate::lang::js::NativeHandlerInput;
use alloc::boxed::Box;
use vize_l0::Span;

/// Private facts are minted from a live original input borrow. The owning
/// File keeps that input parked across the fallible walk and short rejoin.
pub(crate) struct ResolvedHandlerFacts<'a> {
    body: &'a oxc_ast::ast::FunctionBody<'a>,
    source: vize_l1::embed::EmbedSource<'a>,
    tables: Tables<'a>,
}

impl<'a> ResolvedHandlerFacts<'a> {
    pub(crate) fn join(
        self,
        input: &mut Option<NativeHandlerInput<'a>>,
    ) -> Option<HandlerResolution<'a>> {
        let original = input.as_ref()?;
        let source = original.operand().syntax().source();
        if !core::ptr::eq(self.body, original.body())
            || !core::ptr::eq(self.source.authored_root(), source.authored_root())
            || !core::ptr::eq(self.source.text(), source.text())
            || self.source.span() != source.span()
        {
            return None;
        }
        Some(HandlerResolution {
            input: input.take()?,
            tables: self.tables,
        })
    }
}

pub(crate) fn resolve_handler_facts<'a>(
    input: &NativeHandlerInput<'a>,
    bindings: &impl BindingLookup,
) -> Result<ResolvedHandlerFacts<'a>, ResolutionError> {
    let source = input.references();
    let mut pending = Pending::new();
    walk::handler_body(&source, &mut pending)?;
    let tables = pending.finish(bindings)?;
    Ok(ResolvedHandlerFacts {
        body: input.body(),
        source: input.operand().syntax().source(),
        tables,
    })
}

mod facts;
pub use facts::{
    HandlerBinding, HandlerBindingRef, HandlerDeclaration, HandlerDeclarationKind, HandlerLocalId,
    HandlerReference, HandlerScope, HandlerScopeId,
};
pub(super) mod sink;
use sink::{Pending, Tables};

/// Complete bounded handler facts over the original body. Caller-supplied outer
/// identities do not establish native File association or template visibility.
///
/// ```compile_fail
/// use vize_l2::resolution::HandlerResolution;
/// fn substitute<'a>(result: &mut HandlerResolution<'a>) {
///     result.tables.references.clear();
/// }
/// ```
#[derive(Debug)]
pub struct HandlerResolution<'a> {
    input: NativeHandlerInput<'a>,
    tables: Tables<'a>,
}

impl<'a> HandlerResolution<'a> {
    #[must_use]
    pub const fn input(&self) -> &NativeHandlerInput<'a> {
        &self.input
    }
    #[must_use]
    pub fn scopes(&self) -> &[HandlerScope] {
        &self.tables.scopes
    }
    #[must_use]
    pub fn bindings(&self) -> &[HandlerBinding<'a>] {
        &self.tables.bindings
    }
    #[must_use]
    pub fn declarations(&self) -> &[HandlerDeclaration<'a>] {
        &self.tables.declarations
    }
    #[must_use]
    pub fn references(&self) -> &[HandlerReference<'a>] {
        &self.tables.references
    }
    #[must_use]
    pub fn into_input(self) -> NativeHandlerInput<'a> {
        self.input
    }
    pub fn authored_span(&self, span: Span) -> Result<Span, vize_l1::embed::SourceError> {
        self.input.operand().syntax().source().authored_span(span)
    }
}

/// No partial table escapes failure; the entire original event/body owner does.
#[derive(Debug)]
pub struct RejectedHandlerResolution<'a> {
    input: NativeHandlerInput<'a>,
    pub error: ResolutionError,
}
impl<'a> RejectedHandlerResolution<'a> {
    #[must_use]
    pub const fn input(&self) -> &NativeHandlerInput<'a> {
        &self.input
    }
    #[must_use]
    pub fn into_input(self) -> NativeHandlerInput<'a> {
        self.input
    }
}

/// One original AST walk collects declarations and references. A flat fact
/// resolution afterwards accounts for lexical shadowing and function-scoped
/// var hoisting, without walking, decoding or parsing the body again.
///
/// Root $event derives only from this genuine event operand. Local identities
/// are distinct from the caller's outer BindingIds. Functions/classes, loops,
/// destructuring, TS declarations/annotations and other unsupported forms
/// return precise refusals, retaining the complete original owner. These facts
/// do not legalize ui.on, establish File admission, or emit runtime code.
pub fn resolve_handler<'a>(
    input: NativeHandlerInput<'a>,
    bindings: &impl BindingLookup,
) -> Result<HandlerResolution<'a>, Box<RejectedHandlerResolution<'a>>> {
    let result = {
        let source = input.references();
        let mut pending = Pending::new();
        walk::handler_body(&source, &mut pending).and_then(|()| pending.finish(bindings))
    };
    match result {
        Ok(tables) => Ok(HandlerResolution { input, tables }),
        Err(error) => Err(Box::new(RejectedHandlerResolution { input, error })),
    }
}

#[cfg(test)]
mod tests;
