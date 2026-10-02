//! Borrow already-produced L2 tables; emitters never resolve expressions again.

use vize_l0::Span;
use vize_l2::expr::JsExpr;

use super::{AccessProvider, EmitError, EmitErrorKind, ResolutionTable, write_expression};
use crate::write::{LinkSink, Writer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionSetError {
    UnorderedOrDuplicateSpan,
}

/// A source-ordered view of one compile unit's actual expression resolutions.
#[derive(Debug, Clone, Copy)]
pub struct ResolvedExpressions<'r, 'a> {
    tables: &'r [ResolutionTable<'a>],
}

impl<'r, 'a> ResolvedExpressions<'r, 'a> {
    pub fn checked(tables: &'r [ResolutionTable<'a>]) -> Result<Self, ResolutionSetError> {
        if tables.windows(2).any(|pair| {
            matches!(pair, [left, right]
                if key(left.expression().span) >= key(right.expression().span))
        }) {
            return Err(ResolutionSetError::UnorderedOrDuplicateSpan);
        }
        Ok(Self { tables })
    }

    /// Find the exact retained AST, then use its completed table directly.
    /// Merely sharing text or a source span is insufficient to reuse facts.
    pub fn write_retained<L: LinkSink>(
        self,
        writer: &mut Writer<L>,
        authored_file: &str,
        expression: &JsExpr<'_>,
        access: &impl AccessProvider,
    ) -> Result<(), EmitError> {
        let index = self
            .tables
            .binary_search_by_key(&key(expression.span), |table| key(table.expression().span))
            .map_err(|_| EmitError {
                span: expression.span,
                kind: EmitErrorKind::MissingResolution,
            })?;
        let table = self.tables.get(index).ok_or(EmitError {
            span: expression.span,
            kind: EmitErrorKind::MissingResolution,
        })?;
        let retained = table.expression();
        let same_coordinates = match (retained.coordinates, expression.coordinates) {
            (None, None) => true,
            (Some(left), Some(right)) => core::ptr::eq(left, right),
            _ => false,
        };
        if !core::ptr::eq(retained.ast, expression.ast)
            || retained.source != expression.source
            || !same_coordinates
        {
            return Err(EmitError {
                span: expression.span,
                kind: EmitErrorKind::SourceMismatch,
            });
        }
        write_expression(writer, authored_file, table, access)
    }
}

fn key(span: Span) -> (u32, u32) {
    (span.start, span.end)
}
