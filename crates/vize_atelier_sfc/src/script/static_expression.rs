//! Vue-compatible static expression classification for TypeScript enums.

use oxc_ast::ast::{Expression, TSEnumDeclaration};

use crate::types::{BindingMetadata, BindingType};

pub(crate) fn is_static_enum(declaration: &TSEnumDeclaration<'_>) -> bool {
    declaration
        .body
        .members
        .iter()
        .all(|member| member.initializer.as_ref().is_none_or(is_static_expression))
}

pub(crate) fn register_enum(bindings: &mut BindingMetadata, declaration: &TSEnumDeclaration<'_>) {
    let binding_type = if is_static_enum(declaration) {
        BindingType::LiteralConst
    } else {
        BindingType::SetupConst
    };
    bindings
        .bindings
        .insert(declaration.id.name.as_str().into(), binding_type);
}

fn is_static_expression(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::StringLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::BigIntLiteral(_) => true,
        Expression::UnaryExpression(expression) => is_static_expression(&expression.argument),
        Expression::LogicalExpression(expression) => {
            is_static_expression(&expression.left) && is_static_expression(&expression.right)
        }
        Expression::BinaryExpression(expression) => {
            is_static_expression(&expression.left) && is_static_expression(&expression.right)
        }
        Expression::ConditionalExpression(expression) => {
            is_static_expression(&expression.test)
                && is_static_expression(&expression.consequent)
                && is_static_expression(&expression.alternate)
        }
        Expression::SequenceExpression(expression) => {
            expression.expressions.iter().all(is_static_expression)
        }
        Expression::TemplateLiteral(expression) => {
            expression.expressions.iter().all(is_static_expression)
        }
        Expression::ParenthesizedExpression(expression) => {
            is_static_expression(&expression.expression)
        }
        Expression::TSAsExpression(expression) => is_static_expression(&expression.expression),
        Expression::TSSatisfiesExpression(expression) => {
            is_static_expression(&expression.expression)
        }
        Expression::TSTypeAssertion(expression) => is_static_expression(&expression.expression),
        Expression::TSNonNullExpression(expression) => is_static_expression(&expression.expression),
        Expression::TSInstantiationExpression(expression) => {
            is_static_expression(&expression.expression)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;
    use oxc_ast::ast::Statement;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    use super::is_static_enum;

    fn classify(source: &str) -> bool {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
        assert!(!parsed.panicked, "enum should parse: {source}");
        let Statement::TSEnumDeclaration(declaration) = &parsed.program.body[0] else {
            panic!("expected enum declaration: {source}");
        };
        is_static_enum(declaration)
    }

    #[test]
    fn accepts_every_vue_static_enum_initializer_shape() {
        assert!(classify(
            "enum E { Implicit, String = 'x', Number = 1, Boolean = true, Null = null, BigInt = 1n, Unary = -1, Logical = true && false, Binary = 1 + 2, Conditional = true ? 1 : 2, Sequence = (1, 2), Template = `x${1}`, Parenthesized = (1), As = 1 as number, Satisfies = 1 satisfies number, Assert = <number>1, NonNull = 1! }"
        ));
    }

    #[test]
    fn rejects_every_runtime_dependent_neighbor() {
        for source in [
            "enum E { Value = local }",
            "enum E { Value = makeValue() }",
            "enum E { Value = local.value }",
            "enum E { Value = /runtime/ }",
            "enum E { Value = `x${local}` }",
            "enum E { Value = true ? 1 : local }",
        ] {
            assert!(
                !classify(source),
                "runtime enum must stay in setup: {source}"
            );
        }
    }
}
