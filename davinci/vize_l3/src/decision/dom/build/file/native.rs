//! Original selected primitive eligibility within the existing sole L3 visit.

use crate::decision::dom::{DomRejection, DomUnsupported};
use oxc_ast::ast::Expression;
use vize_l0::id::NodeId;
use vize_l2::{expr::JsExpr, file::FileArtifact};

pub(super) fn refusal<'a>(
    file: &FileArtifact<'a>,
    node: NodeId,
    expression: &JsExpr<'a>,
    setup: bool,
) -> Option<DomRejection> {
    // Only the normal private receiver's actual node/input association grants
    // selected origin. Other File routes retain their existing policy.
    let Some(record) = file.native_interpolation(node) else {
        return setup.then_some(DomRejection {
            node,
            span: expression.span,
            reason: DomUnsupported::FileExpression,
        });
    };
    let operand = record.input().operand();
    let rejected = |reason| DomRejection {
        node,
        span: operand.full_span(),
        reason,
    };
    let syntax = operand.syntax();
    let Some(original) = syntax.admitted_expression() else {
        return Some(rejected(DomUnsupported::FileExpression));
    };
    let source = syntax.source();
    if !core::ptr::eq(original.expression(), expression.ast)
        || !core::ptr::eq(source.text(), expression.source)
        || !core::ptr::eq(source.authored_root(), file.artifact().source())
        || source.span() != expression.span
        || expression.coordinates.is_none()
    {
        return Some(rejected(DomUnsupported::FileExpression));
    }
    // A clean embedding parse is not a strict-module spelling receipt. Borrow
    // the original lexer scalar, and never admit an unvalidated RegExp pattern.
    let primitive = matches!(
        original.expression(),
        Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::StringLiteral(_)
    );
    let identifier = setup
        && original
            .expression()
            .get_identifier_reference()
            .is_some_and(|id| {
                expression.ast_span_to_source(id.span).is_some_and(|span| {
                    expression
                        .source
                        .get(span.start as usize..span.end as usize)
                        == Some(id.name.as_str())
                })
            });
    (!(primitive || identifier) || original.has_legacy_literals())
        .then(|| rejected(DomUnsupported::Expression))
}
