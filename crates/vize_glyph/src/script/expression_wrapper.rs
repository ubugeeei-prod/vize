//! Remove only the formatter's unary wrapper, using the existing parsed AST.
use oxc_ast::ast::{Expression, Program, Statement};

#[inline]
pub(super) fn unwrap_argument<'a>(printed: &'a str, program: &Program<'_>) -> Option<&'a str> {
    let [Statement::ExpressionStatement(statement)] = program.body.as_slice() else {
        return None;
    };
    let Expression::UnaryExpression(unary) = &statement.expression else {
        return None;
    };
    // These roots need no unary wrapper. A leading '(' and final ')' therefore
    // belong to a member receiver and call, or to a semantically required
    // sequence. Removing them is unsafe even when both characters are present.
    if matches!(
        &unary.argument,
        Expression::CallExpression(_)
            | Expression::StaticMemberExpression(_)
            | Expression::ComputedMemberExpression(_)
            | Expression::PrivateFieldExpression(_)
            | Expression::ChainExpression(_)
            | Expression::TSNonNullExpression(_)
            | Expression::TSInstantiationExpression(_)
            | Expression::SequenceExpression(_)
    ) {
        return Some(printed);
    }
    Some(
        printed
            .strip_prefix('(')
            .and_then(|rest| rest.strip_suffix(')'))
            .unwrap_or(printed),
    )
}
