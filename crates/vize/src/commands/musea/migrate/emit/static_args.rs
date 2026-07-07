use oxc_ast::ast::{
    ArrayExpressionElement, Expression, ObjectExpression, ObjectPropertyKind, PropertyKey,
};

use super::super::csf::unwrap_expression;

pub(super) type ModuleBindings<'a> = [(&'a str, &'a Expression<'a>)];

pub(super) fn args_contain_unmigrated_bindings(
    meta_args: Option<&ObjectExpression<'_>>,
    story_args: Option<&ObjectExpression<'_>>,
    module_bindings: &ModuleBindings<'_>,
) -> bool {
    if let Some(args) = meta_args
        && args_object_contains_unmigrated_bindings(args, story_args, module_bindings)
    {
        return true;
    }
    story_args
        .is_some_and(|args| args_object_contains_unmigrated_bindings(args, None, module_bindings))
}

fn args_object_contains_unmigrated_bindings(
    args: &ObjectExpression<'_>,
    overrides: Option<&ObjectExpression<'_>>,
    module_bindings: &ModuleBindings<'_>,
) -> bool {
    args.properties.iter().any(|property| {
        let ObjectPropertyKind::ObjectProperty(prop) = property else {
            return true;
        };
        if prop.computed {
            return true;
        }
        let Some(name) = property_key_name(&prop.key) else {
            return true;
        };
        if overrides.is_some_and(|object| args_has_static_property(object, name)) {
            return false;
        }
        if static_binding_value(&prop.value, module_bindings).is_some() {
            return false;
        }
        expression_needs_module_binding(&prop.value)
    })
}

fn expression_needs_module_binding(expression: &Expression<'_>) -> bool {
    match unwrap_expression(expression) {
        Expression::StringLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_) => false,
        Expression::TemplateLiteral(template) => !template.expressions.is_empty(),
        Expression::UnaryExpression(unary) => expression_needs_module_binding(&unary.argument),
        Expression::ArrayExpression(array) => array.elements.iter().any(|element| match element {
            ArrayExpressionElement::Elision(_) => false,
            ArrayExpressionElement::SpreadElement(_) => true,
            _ => element
                .as_expression()
                .is_none_or(expression_needs_module_binding),
        }),
        Expression::ObjectExpression(object) => {
            args_object_contains_unmigrated_bindings(object, None, &[])
        }
        Expression::Identifier(_)
        | Expression::StaticMemberExpression(_)
        | Expression::ComputedMemberExpression(_)
        | Expression::CallExpression(_) => true,
        _ => true,
    }
}

pub(super) fn static_binding_value<'a>(
    expression: &'a Expression<'a>,
    module_bindings: &ModuleBindings<'a>,
) -> Option<&'a Expression<'a>> {
    let Expression::Identifier(ident) = unwrap_expression(expression) else {
        return None;
    };
    let value = module_bindings
        .iter()
        .rev()
        .find_map(|(name, value)| (*name == ident.name.as_str()).then_some(*value))?;
    (!expression_needs_module_binding(value)).then_some(value)
}

pub(super) fn args_has_static_property(args: &ObjectExpression<'_>, name: &str) -> bool {
    args.properties.iter().any(|property| {
        let ObjectPropertyKind::ObjectProperty(prop) = property else {
            return false;
        };
        !prop.computed && property_key_name(&prop.key) == Some(name)
    })
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(ident) => Some(ident.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}
