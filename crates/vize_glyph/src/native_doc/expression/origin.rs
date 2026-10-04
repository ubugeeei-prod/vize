//! Private borrowed authority; neither AST references nor source tuples mint it.

use oxc_ast::ast::Expression;
use vize_l0::Span;
use vize_l1::embed::{
    EmbedSource, SourceError,
    syntax::{CommentView, NativeExpressionView, RetainedExpression},
};

use super::ExpressionRefusal;

#[derive(Clone, Copy)]
pub(super) enum Origin<'p, 'a> {
    Retained(&'p RetainedExpression<'a>),
    Borrowed(&'p NativeExpressionView<'p, 'a>),
}

impl<'p, 'a> Origin<'p, 'a> {
    pub fn source(self) -> EmbedSource<'a> {
        match self {
            Self::Retained(original) => original.source(),
            Self::Borrowed(original) => original.source(),
        }
    }

    pub fn expression(self) -> Result<&'p Expression<'a>, ExpressionRefusal> {
        let (root, content) = match self {
            Self::Retained(original) => {
                let admitted =
                    original
                        .admitted_expression()
                        .ok_or(ExpressionRefusal::Unadmitted {
                            hole: original.hole(),
                        })?;
                (admitted.expression(), admitted.content())
            }
            Self::Borrowed(original) => {
                let admitted = original.admitted_expression();
                (admitted.expression(), admitted.content())
            }
        };
        if !core::ptr::eq(content, self.source().text()) {
            return Err(ExpressionRefusal::SourceMismatch {
                span: self.source().span(),
            });
        }
        Ok(root)
    }

    pub fn decoded_span(self, span: oxc_span::Span) -> Result<Span, SourceError> {
        match self {
            Self::Retained(original) => original.decoded_span(span),
            Self::Borrowed(original) => original.decoded_span(span),
        }
    }

    pub fn comments(
        self,
        mut visit: impl FnMut(CommentView<'p, 'a>) -> Result<(), ExpressionRefusal>,
    ) -> Result<(), ExpressionRefusal> {
        match self {
            Self::Retained(original) => {
                for comment in original.comments() {
                    visit(comment)?;
                }
            }
            Self::Borrowed(original) => {
                for comment in original.comments() {
                    visit(comment)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "origin/tests.rs"]
mod tests;
