//! Normally owned original handler input for the future whole-body resolver.

use alloc::boxed::Box;
use oxc_ast::ast::FunctionBody;
use vize_l0::Span;
use vize_l1::embed::{EmbedSource, SourceError, syntax::RetainedHandlerBody};

use crate::resolution::source::HandlerReferenceSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandlerInputErrorKind {
    IncompleteSyntax,
}

/// A refusal retains the complete original syntax, including holes, comments,
/// diagnostics and source. It cannot supply the no-hole reference capability.
#[derive(Debug)]
pub struct RejectedHandlerInput<'a> {
    syntax: RetainedHandlerBody<'a>,
    pub kind: HandlerInputErrorKind,
    pub span: Span,
}

impl<'a> RejectedHandlerInput<'a> {
    #[must_use]
    pub const fn syntax(&self) -> &RetainedHandlerBody<'a> {
        &self.syntax
    }

    #[must_use]
    pub fn into_syntax(self) -> RetainedHandlerBody<'a> {
        self.syntax
    }
}

/// Whole original handler owner; coordinate projection is not identifier
/// resolution, File association or native event admission.
///
/// Caller-selected AST/source pairs cannot establish this input:
/// ```compile_fail
/// use oxc_ast::ast::FunctionBody;
/// use vize_l1::embed::EmbedSource;
/// use vize_l2::lang::js::HandlerInput;
/// fn substitute<'a>(body: &'a FunctionBody<'a>, source: EmbedSource<'a>) {
///     let _ = HandlerInput::new(body, source);
/// }
/// ```
/// Its normal owner cannot be duplicated:
/// ```compile_fail
/// use vize_l2::lang::js::HandlerInput;
/// fn duplicate(input: HandlerInput<'_>) {
///     let _ = input.clone();
/// }
/// ```
#[derive(Debug)]
pub struct HandlerInput<'a> {
    syntax: RetainedHandlerBody<'a>,
    body: &'a FunctionBody<'a>,
}

impl<'a> HandlerInput<'a> {
    pub fn new(syntax: RetainedHandlerBody<'a>) -> Result<Self, Box<RejectedHandlerInput<'a>>> {
        let Some(body) = HandlerReferenceSource::checked(&syntax).map(|source| source.body())
        else {
            return Err(Box::new(RejectedHandlerInput {
                kind: HandlerInputErrorKind::IncompleteSyntax,
                span: syntax.source().span(),
                syntax,
            }));
        };
        Ok(Self { syntax, body })
    }

    #[must_use]
    pub const fn syntax(&self) -> &RetainedHandlerBody<'a> {
        &self.syntax
    }

    #[must_use]
    pub const fn source(&self) -> EmbedSource<'a> {
        self.syntax.source()
    }

    /// The stock generated container is not an authored source range.
    #[must_use]
    pub fn body(&self) -> &'a FunctionBody<'a> {
        self.body
    }

    /// Checked coordinate projection only. An arbitrary range is not evidence
    /// that a node belongs to this body or has been semantically observed.
    pub fn decoded_span(&self, span: oxc_span::Span) -> Result<Span, SourceError> {
        self.references().decoded_span(span)
    }

    pub fn authored_span(&self, span: oxc_span::Span) -> Result<Span, SourceError> {
        self.references().authored_span(span)
    }

    #[must_use]
    pub fn into_syntax(self) -> RetainedHandlerBody<'a> {
        self.syntax
    }

    pub(crate) fn references(&self) -> HandlerReferenceSource<'_, 'a> {
        HandlerReferenceSource::from_input(self)
    }
}

#[cfg(test)]
mod tests;

mod native;
pub use native::{NativeHandlerInput, RejectedNativeHandlerInput};
