use oxc_ast::ast::Expression;
use oxc_syntax::operator::BinaryOperator;
use vize_l0::String;

/// Only the exact `typeof name [!== / ===] "undefined"` witness establishes
/// browser presence. Other conditions do not change execution assumptions.
pub(super) fn defined_name(expression: &Expression<'_>, truthy: bool) -> Option<String> {
    let Expression::BinaryExpression(binary) = expression.get_inner_expression() else {
        return None;
    };
    let unequal = match binary.operator {
        BinaryOperator::Equality | BinaryOperator::StrictEquality => false,
        BinaryOperator::Inequality | BinaryOperator::StrictInequality => true,
        _ => return None,
    };
    if unequal != truthy {
        return None;
    }
    for (probe, value) in [(&binary.left, &binary.right), (&binary.right, &binary.left)] {
        if let Expression::StringLiteral(value) = value.get_inner_expression()
            && value.value == "undefined"
            && let Expression::UnaryExpression(probe) = probe.get_inner_expression()
            && probe.operator == oxc_syntax::operator::UnaryOperator::Typeof
            && let Expression::Identifier(id) = probe.argument.get_inner_expression()
        {
            return Some(String::from(id.name.as_str()));
        }
    }
    None
}
