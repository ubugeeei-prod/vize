//! Original short-lived expression authority and unchanged coordinate views.

use oxc_ast::ast::Expression;
use oxc_parser::{AdmittedBorrowedExpression, ParseOptions};
use oxc_span::SourceType;
use vize_l0::Span;

use super::{CommentView, DiagnosticView, EmbedSource, Grammar, NativeSyntax, Shape, SourceError};

/// A sealed borrow of the original syntax and its authentic parser admission.
/// Neither its complete observation nor its expression root is moved or cloned.
/// The owner and its source/arena must remain alive for the returned view.
///
/// ```
/// use vize_l0::{Allocator, Span};
/// use vize_l1::embed::{Embed, EmbedSource, Grammar, Lang, Shape};
/// use vize_l1::embed::syntax::parse_once;
/// let arena = Allocator::default();
/// let source = "left + right";
/// let owner = parse_once(&arena, Embed {
///     source: EmbedSource::authored(source, Span::new(0, 12)).unwrap(),
///     grammar: Grammar { shape: Shape::Expr, lang: Lang::Js },
/// });
/// let view = owner.borrow_expression().unwrap();
/// assert!(core::ptr::eq(view.original(), &owner));
/// assert!(core::ptr::eq(view.expression(), owner.expression().unwrap()));
/// assert_eq!(view.source().text(), source);
/// ```
///
/// Raw AST references cannot construct parser authority.
///
/// ```compile_fail,E0451
/// use oxc_ast::ast::Expression;
/// use oxc_parser::{AdmittedBorrowedExpression, EmbeddingObservation};
/// fn forge<'o, 'a>(owner: &'o EmbeddingObservation<'a>, root: &'o Expression<'a>)
///     -> AdmittedBorrowedExpression<'o, 'a> {
///     AdmittedBorrowedExpression { owner, root }
/// }
/// ```
///
/// An unrelated syntax owner cannot be paired with an authentic parser proof.
///
/// ```compile_fail,E0451
/// use oxc_parser::AdmittedBorrowedExpression;
/// use vize_l1::embed::syntax::{NativeExpressionView, NativeSyntax};
/// fn forge<'o, 'a>(original: &'o NativeSyntax<'a>, admitted: AdmittedBorrowedExpression<'o, 'a>)
///     -> NativeExpressionView<'o, 'a> {
///     NativeExpressionView { original, admitted }
/// }
/// ```
///
/// The root's short owner borrow cannot become an arena-lifetime reference.
///
/// ```compile_fail
/// use oxc_ast::ast::Expression;
/// use vize_l1::embed::syntax::NativeExpressionView;
/// fn lengthen<'o, 'a>(view: NativeExpressionView<'o, 'a>) -> &'a Expression<'a> {
///     view.expression()
/// }
/// ```
///
/// ```compile_fail,E0515
/// use vize_l1::embed::syntax::{NativeExpressionView, NativeSyntax};
/// fn outlive_owner<'a>(owner: NativeSyntax<'a>) -> NativeExpressionView<'a, 'a> {
///     owner.borrow_expression().unwrap()
/// }
/// ```
///
/// ```compile_fail,E0505
/// use vize_l1::embed::syntax::NativeSyntax;
/// fn drop_while_borrowed(owner: NativeSyntax<'_>) {
///     let view = owner.borrow_expression().unwrap();
///     drop(owner);
///     println!("{}", view.source().text());
/// }
/// ```
///
/// ```compile_fail,E0505
/// use vize_l1::embed::syntax::NativeSyntax;
/// fn move_while_borrowed(owner: NativeSyntax<'_>) {
///     let view = owner.borrow_expression().unwrap();
///     let moved = Box::new(owner);
///     println!("{}", view.source().text());
///     drop(moved);
/// }
/// ```
///
/// Source lifetime remains independent of a longer owner borrow.
///
/// ```compile_fail
/// use vize_l1::embed::syntax::{NativeExpressionView, NativeSyntax};
/// fn replace_source<'o, 'a>(owner: &'o NativeSyntax<'a>) -> NativeExpressionView<'o, 'static> {
///     owner.borrow_expression().unwrap()
/// }
/// ```
///
/// The original arena cannot be dropped even when source bytes are static.
///
/// ```compile_fail,E0597
/// use vize_l0::{Allocator, Span};
/// use vize_l1::embed::{Embed, EmbedSource, Grammar, Lang, Shape};
/// use vize_l1::embed::syntax::parse_once;
/// let owner = {
///     let arena = Allocator::default();
///     parse_once(&arena, Embed {
///         source: EmbedSource::authored("item", Span::new(0, 4)).unwrap(),
///         grammar: Grammar { shape: Shape::Expr, lang: Lang::Js },
///     })
/// };
/// let view = owner.borrow_expression().unwrap();
/// println!("{}", view.source().text());
/// ```
///
/// Existing consuming handoffs cannot discard an outstanding original borrow.
///
/// ```compile_fail,E0505
/// use vize_l1::embed::syntax::NativeSyntax;
/// fn consume_while_borrowed(owner: NativeSyntax<'_>) {
///     let view = owner.borrow_expression().unwrap();
///     let retained = owner.into_expression().unwrap();
///     println!("{}", view.source().text());
///     drop(retained);
/// }
/// ```
pub struct NativeExpressionView<'o, 'a> {
    original: &'o NativeSyntax<'a>,
    admitted: AdmittedBorrowedExpression<'o, 'a>,
}

impl core::fmt::Debug for NativeExpressionView<'_, '_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("NativeExpressionView")
            .field(self.original)
            .finish()
    }
}

impl<'a> NativeSyntax<'a> {
    /// Borrow the original Expr observation without consuming syntax custody.
    /// The real stock-parser witness and exact original content identity are
    /// mandatory; callers cannot supply an AST, admission status or source map.
    #[must_use]
    pub fn borrow_expression(&self) -> Option<NativeExpressionView<'_, 'a>> {
        if self.hole.is_some() || self.grammar.shape != Shape::Expr {
            return None;
        }
        let admitted = self.embedding.as_ref()?.admitted_expression()?;
        if !core::ptr::eq(admitted.content(), self.source().text())
            || admitted.source_type() != self.source_type
            || admitted.parser_content_span().start != self.coordinates.prefix
        {
            return None;
        }
        Some(NativeExpressionView {
            original: self,
            admitted,
        })
    }
}

impl<'o, 'a> NativeExpressionView<'o, 'a> {
    #[must_use]
    pub const fn original(&self) -> &'o NativeSyntax<'a> {
        self.original
    }
    #[must_use]
    pub const fn admitted_expression(&self) -> &AdmittedBorrowedExpression<'o, 'a> {
        &self.admitted
    }
    #[must_use]
    pub const fn expression(&self) -> &'o Expression<'a> {
        self.admitted.expression()
    }
    #[must_use]
    pub const fn grammar(&self) -> Grammar {
        self.original.grammar()
    }
    #[must_use]
    pub const fn source_type(&self) -> SourceType {
        self.original.source_type()
    }
    #[must_use]
    pub const fn source(&self) -> EmbedSource<'a> {
        self.original.source()
    }
    #[must_use]
    pub const fn parser_prefix(&self) -> u32 {
        self.original.coordinates.prefix
    }
    #[must_use]
    pub const fn options(&self) -> ParseOptions {
        self.admitted.options()
    }
    #[must_use]
    pub const fn has_legacy_literals(&self) -> bool {
        self.admitted.has_legacy_literals()
    }
    pub fn comments(&self) -> impl Iterator<Item = CommentView<'o, 'a>> {
        let original: &'o NativeSyntax<'a> = self.original;
        original.comments()
    }
    pub fn diagnostics(&self) -> impl Iterator<Item = DiagnosticView<'o, 'a>> {
        let original: &'o NativeSyntax<'a> = self.original;
        original.diagnostics()
    }
    pub fn decoded_span(&self, span: oxc_span::Span) -> Result<Span, SourceError> {
        self.original.decoded_span(span)
    }
    pub fn authored_span(&self, span: oxc_span::Span) -> Result<Span, SourceError> {
        self.original.authored_span(span)
    }
}

#[cfg(test)]
mod tests;
