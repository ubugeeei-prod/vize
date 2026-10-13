//! Receivers rooted at the component instance (`this`).

use oxc_ast::ast::{AssignmentTarget, Expression, SimpleAssignmentTarget};

/// Whether an assignment target is a member access rooted at `this`
/// (e.g. `this.foo`, `this.foo.bar`, `this['foo']`).
pub(super) fn assignment_target_is_this_member(target: &AssignmentTarget<'_>) -> bool {
    match target {
        AssignmentTarget::StaticMemberExpression(member) => expression_root_is_this(&member.object),
        AssignmentTarget::ComputedMemberExpression(member) => {
            expression_root_is_this(&member.object)
        }
        _ => false,
    }
}

pub(super) fn simple_target_is_this_member(target: &SimpleAssignmentTarget<'_>) -> bool {
    match target {
        SimpleAssignmentTarget::StaticMemberExpression(member) => {
            expression_root_is_this(&member.object)
        }
        SimpleAssignmentTarget::ComputedMemberExpression(member) => {
            expression_root_is_this(&member.object)
        }
        _ => false,
    }
}

/// Whether a member-access chain bottoms out at `this`
/// (`this`, `this.a`, `this.a.b`, `this.a[i]`, `this!.a`, `(this).a`, ...).
pub(super) fn expression_root_is_this(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::ThisExpression(_) => true,
        Expression::StaticMemberExpression(member) => expression_root_is_this(&member.object),
        Expression::ComputedMemberExpression(member) => expression_root_is_this(&member.object),
        Expression::PrivateFieldExpression(member) => expression_root_is_this(&member.object),
        Expression::ParenthesizedExpression(paren) => expression_root_is_this(&paren.expression),
        Expression::TSNonNullExpression(non_null) => expression_root_is_this(&non_null.expression),
        Expression::TSAsExpression(ts_as) => expression_root_is_this(&ts_as.expression),
        Expression::TSSatisfiesExpression(satisfies) => {
            expression_root_is_this(&satisfies.expression)
        }
        _ => false,
    }
}
