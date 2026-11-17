//! Runtime prop option values read as TypeScript types them.

use oxc_ast::ast::{Expression, TSLiteral, TSType, TSTypeName};

/// The boolean literal type TypeScript gives `expr`, if any.
///
/// Vue reads `required` and similar options from the option's type, so a
/// value keeps counting as `true` through parentheses, `satisfies`, `!`,
/// `as const`/`<const>` and `as true`. `true as boolean` has no literal type.
pub fn literal_boolean_type(expr: &Expression<'_>) -> Option<bool> {
    match expr {
        Expression::BooleanLiteral(literal) => Some(literal.value),
        Expression::ParenthesizedExpression(wrapper) => literal_boolean_type(&wrapper.expression),
        Expression::TSSatisfiesExpression(wrapper) => literal_boolean_type(&wrapper.expression),
        Expression::TSNonNullExpression(wrapper) => literal_boolean_type(&wrapper.expression),
        Expression::TSAsExpression(wrapper) => {
            asserted_boolean_type(&wrapper.type_annotation, &wrapper.expression)
        }
        Expression::TSTypeAssertion(wrapper) => {
            asserted_boolean_type(&wrapper.type_annotation, &wrapper.expression)
        }
        _ => None,
    }
}

fn asserted_boolean_type(annotation: &TSType<'_>, expr: &Expression<'_>) -> Option<bool> {
    match annotation {
        TSType::TSLiteralType(literal) => match &literal.literal {
            TSLiteral::BooleanLiteral(value) => Some(value.value),
            _ => None,
        },
        _ if is_const_type_assertion(annotation) => literal_boolean_type(expr),
        _ => None,
    }
}

/// `as const`/`<const>`: the assertion keeps the expression's literal types.
pub(crate) fn is_const_type_assertion(annotation: &TSType<'_>) -> bool {
    matches!(annotation, TSType::TSTypeReference(reference)
        if reference.type_arguments.is_none()
            && matches!(&reference.type_name, TSTypeName::IdentifierReference(identifier)
                if identifier.name == "const"))
}

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    use super::literal_boolean_type;

    #[test]
    fn keeps_only_wrappers_that_keep_the_literal_type() {
        for (source, expected) in [
            ("true", Some(true)),
            ("false", Some(false)),
            ("(true)", Some(true)),
            ("true as const", Some(true)),
            ("<const>true", Some(true)),
            ("true satisfies boolean", Some(true)),
            ("(true as const) satisfies boolean", Some(true)),
            ("true!", Some(true)),
            ("true as true", Some(true)),
            ("false as const", Some(false)),
            ("true as boolean", None),
            ("<boolean>true", None),
            ("!false", None),
            ("flag", None),
        ] {
            let allocator = Allocator::default();
            let expression = Parser::new(&allocator, source, SourceType::ts())
                .parse_expression()
                .unwrap();
            assert_eq!(literal_boolean_type(&expression), expected, "{source}");
        }
    }
}
