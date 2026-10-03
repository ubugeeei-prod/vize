//! Original handler source coordinates, borrowed only from the full L1 owner.

use oxc_ast::ast::FunctionBody;
use vize_l0::Span;
use vize_l1::embed::SourceError;
use vize_l1::embed::syntax::RetainedHandlerBody;

use super::ReferenceSource;

/// A short original no-hole owner capability. This is not a public AST/source
/// pair and never converts a FunctionBody into an expression or Program.
pub(crate) struct HandlerReferenceSource<'h, 'a> {
    owner: &'h RetainedHandlerBody<'a>,
    body: &'a FunctionBody<'a>,
}

impl<'h, 'a> HandlerReferenceSource<'h, 'a> {
    pub(crate) fn checked(owner: &'h RetainedHandlerBody<'a>) -> Option<Self> {
        let admitted = owner.admitted_body()?;
        Some(Self {
            owner,
            body: admitted.body(),
        })
    }

    pub(crate) fn from_input(input: &'h crate::lang::js::HandlerInput<'a>) -> Self {
        Self {
            owner: input.syntax(),
            body: input.body(),
        }
    }

    pub(crate) fn from_native(input: &'h crate::lang::js::NativeHandlerInput<'a>) -> Self {
        Self {
            owner: input.operand().syntax(),
            body: input.body(),
        }
    }

    pub(crate) const fn body(&self) -> &'a FunctionBody<'a> {
        self.body
    }

    /// Only the original parser's generated prefix is removed. UTF-8 boundaries
    /// and the complete original decoded window are checked before projection.
    pub(crate) fn decoded_span(&self, span: oxc_span::Span) -> Result<Span, SourceError> {
        ReferenceSource::Handler {
            text: self.owner.source().text(),
            prefix: self.owner.parser_prefix(),
        }
        .span(span)
        .ok_or(SourceError::InvalidDecodedSpan)
    }

    pub(crate) fn authored_span(&self, span: oxc_span::Span) -> Result<Span, SourceError> {
        self.owner.source().authored_span(self.decoded_span(span)?)
    }
}
