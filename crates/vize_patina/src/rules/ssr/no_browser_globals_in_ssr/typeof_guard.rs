use super::script_symbols;
use oxc_ast::ast::Expression;
use oxc_semantic::Semantic;
use oxc_syntax::operator::BinaryOperator;
use vize_l0::String;

/// Only an actual import-meta flag proves that this arm cannot execute in SSR.
/// Ordinary objects, aliases and computed properties do not witness the realm.
pub(super) fn client_only(expression: &Expression<'_>, truthy: bool) -> bool {
    match expression.get_inner_expression() {
        Expression::StaticMemberExpression(member)
            if !member.optional
                && matches!(
                    member.object.get_inner_expression(),
                    Expression::ImportMeta(_)
                ) =>
        {
            match member.property.name.as_str() {
                "client" => truthy,
                "server" => !truthy,
                _ => false,
            }
        }
        Expression::UnaryExpression(unary)
            if unary.operator == oxc_syntax::operator::UnaryOperator::LogicalNot =>
        {
            client_only(&unary.argument, !truthy)
        }
        _ => false,
    }
}

/// Only the exact `typeof name [!== / ===] "undefined"` witness establishes
/// browser presence. Other conditions do not change execution assumptions.
pub(super) fn defined_name(
    expression: &Expression<'_>,
    truthy: bool,
    semantic: &Semantic<'_>,
) -> Option<String> {
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
            && let Some(reference) = id.reference_id.get()
            && !semantic
                .scoping()
                .get_reference(reference)
                .symbol_id()
                .is_some_and(|symbol| script_symbols::runtime_shadow(semantic, symbol))
        {
            return Some(String::from(id.name.as_str()));
        }
    }
    None
}
