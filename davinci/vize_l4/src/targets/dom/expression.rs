//! Statically selected expression writers; the caller cannot supply a file policy.

use vize_l0::id::NodeId;
use vize_l2::expr::ExprRef;
use vize_l3::decision::NativeAnalysis;

use super::{DomError, DomErrorKind};
use crate::expr::{AccessProvider, ResolvedExpressions};
use crate::write::{LinkSink, Writer};

pub(super) trait ExpressionWriter {
    fn write<L: LinkSink>(
        &self,
        writer: &mut Writer<L>,
        node: NodeId,
        expression: ExprRef<'_>,
    ) -> Result<(), DomError>;
}

// The previous bare-artifact contract remains distinct from the genuine file
// entry. It never supplies runtime access to a file-owned analysis.
pub(super) struct BareExpressions<'s, 'owner, 'arena, A: AccessProvider> {
    pub analysis: &'s NativeAnalysis<'owner, 'arena>,
    pub expressions: ResolvedExpressions<'s, 'arena>,
    pub access: &'s A,
}

impl<A: AccessProvider> ExpressionWriter for BareExpressions<'_, '_, '_, A> {
    fn write<L: LinkSink>(
        &self,
        writer: &mut Writer<L>,
        node: NodeId,
        expression: ExprRef<'_>,
    ) -> Result<(), DomError> {
        let ExprRef::Js(retained) = expression else {
            return Err(DomError {
                node: Some(node),
                span: expression.span(),
                kind: DomErrorKind::UnsupportedExpression,
            });
        };
        self.expressions
            .write_retained(
                writer,
                self.analysis.artifact().source(),
                retained,
                self.access,
            )
            .map_err(|error| DomError {
                node: Some(node),
                span: error.span,
                kind: DomErrorKind::Expression(error),
            })
    }
}
