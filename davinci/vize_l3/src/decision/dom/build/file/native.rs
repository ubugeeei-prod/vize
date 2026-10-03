//! Original selected primitive eligibility within the existing sole L3 visit.

use crate::decision::dom::{DomRejection, DomUnsupported};
use oxc_ast::ast::Expression;
use vize_l0::id::NodeId;
use vize_l2::{expr::JsExpr, file::FileArtifact};

pub(super) fn refusal<'a>(
    file: &FileArtifact<'a>,
    node: NodeId,
    expression: &JsExpr<'a>,
) -> Option<DomRejection> {
    // Only the normal private receiver's actual node/input association grants
    // selected origin. Other File routes retain their existing policy.
    let record = file.native_interpolation(node)?;
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
    (!primitive || original.has_legacy_literals()).then(|| rejected(DomUnsupported::Expression))
}
