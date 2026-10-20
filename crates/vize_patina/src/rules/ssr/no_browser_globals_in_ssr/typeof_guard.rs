use super::script_symbols;
use oxc_ast::ast::Expression;
use oxc_semantic::Semantic;
use oxc_syntax::operator::BinaryOperator;
use vize_l0::String;

/// A truthy static `import.meta.client` arm does not execute in SSR: Nuxt
/// replaces it with false, and hosts without the flag leave it undefined.
/// The context has no framework witness, so `import.meta.server` cannot prove
/// safety: its false arm (or negation) also executes when that flag is absent.
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
            member.property.name == "client" && truthy
        }
        Expression::UnaryExpression(unary)
            if unary.operator == oxc_syntax::operator::UnaryOperator::LogicalNot =>
        {
            client_only(&unary.argument, !truthy)
        }
        _ => false,
    }
}

/// A branch that excludes `typeof name === "undefined"` establishes browser
/// presence. Equality with any other type string proves the same thing.
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
    for (probe, value) in [(&binary.left, &binary.right), (&binary.right, &binary.left)] {
        if let Expression::StringLiteral(value) = value.get_inner_expression()
            && if value.value == "undefined" {
                unequal == truthy
            } else {
                unequal != truthy
            }
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
