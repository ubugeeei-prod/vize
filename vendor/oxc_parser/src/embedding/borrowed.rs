//! Short immutable admission over the original complete parser observation.

use oxc_ast::ast::{Comment, Expression};
use oxc_diagnostics::Diagnostics;
use oxc_span::{GetSpan, SourceType, Span};

use super::EmbeddingObservation;
use crate::ParseOptions;

/// A root authenticated by its original embedding owner, without moving it.
/// The root remains borrowed for the owner's lifetime, not the arena lifetime.
/// No caller AST, source window or replacement admission status can mint this.
pub struct AdmittedBorrowedExpression<'o, 'a> {
    owner: &'o EmbeddingObservation<'a>,
    root: &'o Expression<'a>,
}

impl<'a> EmbeddingObservation<'a> {
    /// Borrow only the original successfully admitted Expr root. Existing local
    /// holes and other goals retain their observations without exposing a proof.
    /// Selection checks the stored fixed wrapper in constant size; this never
    /// consumes the owner, allocates, parses or walks expression descendants.
    #[must_use]
    pub fn admitted_expression(&self) -> Option<AdmittedBorrowedExpression<'_, 'a>> {
        let root = self.expression()?;
        Some(AdmittedBorrowedExpression { owner: self, root })
    }
}

impl<'o, 'a> AdmittedBorrowedExpression<'o, 'a> {
    #[must_use]
    pub const fn original(&self) -> &'o EmbeddingObservation<'a> {
        self.owner
    }
    #[must_use]
    pub const fn expression(&self) -> &'o Expression<'a> {
        self.root
    }
    #[must_use]
    pub const fn content(&self) -> &'a str {
        self.owner.content
    }
    #[must_use]
    pub const fn source_type(&self) -> SourceType {
        self.owner.source_type
    }
    #[must_use]
    pub const fn options(&self) -> ParseOptions {
        self.owner.options
    }
    #[must_use]
    pub const fn parser_content_span(&self) -> Span {
        self.owner.content_span
    }
    #[must_use]
    pub fn parser_container_span(&self) -> Span {
        self.root.span()
    }
    #[must_use]
    pub fn comments(&self) -> &'o [Comment] {
        self.owner.comments()
    }
    #[must_use]
    pub fn diagnostics(&self) -> &'o Diagnostics {
        self.owner.diagnostics()
    }
    /// Borrow the stored stock-lexer fact without another source or AST scan.
    /// False does not certify semantic or classic-runtime early-error validity.
    #[must_use]
    pub const fn has_legacy_literals(&self) -> bool {
        self.owner.parsed.has_legacy_literals
    }
}
