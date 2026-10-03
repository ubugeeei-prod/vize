//! Native documents over the original compiler-profile expression tree.

use oxc_span::GetSpan;
use vize_l0::{Allocator, SourceBlock, Span, Vec};
use vize_l1::embed::{
    SourceError,
    syntax::{EmbedHole, RetainedExpression},
};

use super::Doc;

#[path = "expression/ast.rs"]
mod ast;
#[path = "expression/source.rs"]
mod source;
use source::{Context, Gap};

/// A bounded syntax consumer; larger L1 safety admissions remain independent.
pub const MAX_EXPRESSION_DOCUMENT_DEPTH: usize = 16;

/// Refusal retains the caller's original parse, diagnostics and comments.
/// SourceMismatch spans are authored-file coordinates; syntax/gap/depth spans
/// are decoded-relative coordinates in that retained expression's source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpressionRefusal {
    Unadmitted { hole: Option<EmbedHole> },
    SourceMismatch { span: Span },
    Projection { offset: u32, error: SourceError },
    UnsupportedNode { span: Span },
    InvalidGap { span: Span },
    InvalidFraming { span: Span },
    DepthLimit { span: Span },
}

/// The document borrows the same admitted expression and authored source.
#[derive(Debug)]
pub struct ExpressionDocument<'p, 'a> {
    original: &'p RetainedExpression<'a>,
    document: Doc<'a>,
}

impl<'p, 'a> ExpressionDocument<'p, 'a> {
    pub fn original(&self) -> &'p RetainedExpression<'a> {
        self.original
    }
    pub fn document(&self) -> &Doc<'a> {
        &self.document
    }
    /// Transfer both the same retained borrow and its document to a consumer.
    pub fn into_parts(self) -> (&'p RetainedExpression<'a>, Doc<'a>) {
        (self.original, self.document)
    }
}

#[cfg(test)]
#[path = "expression/projection_tests.rs"]
mod projection_tests;

/// Build directly from the original retained compiler-profile AST, once.
///
/// Identifier, numeric/string/boolean/null atoms, authored parentheses and
/// binary/logical nodes are supported. Original literal/operator spellings,
/// parentheses, complete entities and typed comments remain source slices.
/// Only proven plain ASCII whitespace gaps change. Encoded whitespace and
/// gaps containing comments stay verbatim. This does not select a language,
/// parse, clone/normalize an AST or call an OXC/legacy formatter.
///
/// The checked caller block must cover the selected embed inside the same
/// physical authored root. It establishes byte custody, not document/version
/// identity or whole-product admission. Unsupported descendants refuse the
/// complete document while leaving original observations available.
pub fn expression_document<'p, 'a>(
    original: &'p RetainedExpression<'a>,
    block: SourceBlock<'a>,
    allocator: &'a Allocator,
) -> Result<ExpressionDocument<'p, 'a>, ExpressionRefusal> {
    let admitted = original
        .admitted_expression()
        .ok_or(ExpressionRefusal::Unadmitted {
            hole: original.hole(),
        })?;
    if admitted.content().len() != original.source().text().len()
        || admitted.content().as_ptr() != original.source().text().as_ptr()
    {
        return Err(ExpressionRefusal::SourceMismatch {
            span: original.source().span(),
        });
    }
    let mut context = Context::new(original, block, allocator)?;
    let entire = Span::new(0, original.source().text().len() as u32);
    let root = context.decoded_span(admitted.expression().span())?;
    let mut parts = Vec::new_in(&allocator);
    parts.push(context.gap(Span::new(0, root.start), Gap::Empty)?);
    parts.push(context.node(admitted.expression(), entire, 0)?);
    parts.push(context.gap(Span::new(root.end, entire.end), Gap::Empty)?);
    context.finish()?;
    Ok(ExpressionDocument {
        original,
        document: Doc::concat(parts),
    })
}
