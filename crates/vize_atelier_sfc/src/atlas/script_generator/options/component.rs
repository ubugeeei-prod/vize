//! Component options object discovery and shared expression helpers.

use oxc_ast::ast::{
    Argument, ArrayExpressionElement, CallExpression, ExportDefaultDeclarationKind, Expression,
    ObjectExpression, ObjectPropertyKind, Program, PropertyKey, Statement,
};
use oxc_span::Span;

pub(crate) fn component_options_from_program<'a>(
    program: &'a Program<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    program.body.iter().find_map(|statement| {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            return None;
        };
        component_options_from_declaration(&export.declaration)
    })
}

fn component_options_from_declaration<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object),
        ExportDefaultDeclarationKind::CallExpression(call) => component_options_from_call(call),
        ExportDefaultDeclarationKind::TSAsExpression(value) => {
            component_options_from_expression(&value.expression)
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(value) => {
            component_options_from_expression(&value.expression)
        }
        ExportDefaultDeclarationKind::TSNonNullExpression(value) => {
            component_options_from_expression(&value.expression)
        }
        _ => None,
    }
}

fn component_options_from_call<'a>(
    call: &'a CallExpression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    if !is_define_component_callee(&call.callee) {
        return None;
    }
    match call.arguments.first()? {
        Argument::ObjectExpression(object) => Some(object),
        Argument::CallExpression(call) => component_options_from_call(call),
        argument => argument
            .as_expression()
            .and_then(component_options_from_expression),
    }
}

fn component_options_from_expression<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object),
        Expression::CallExpression(call) => component_options_from_call(call),
        Expression::ParenthesizedExpression(value) => {
            component_options_from_expression(&value.expression)
        }
        Expression::TSAsExpression(value) => component_options_from_expression(&value.expression),
        Expression::TSSatisfiesExpression(value) => {
            component_options_from_expression(&value.expression)
        }
        Expression::TSNonNullExpression(value) => {
            component_options_from_expression(&value.expression)
        }
        _ => None,
    }
}

pub(crate) fn option_expression_property<'a>(
    object: &'a ObjectExpression<'a>,
    name: &str,
) -> Option<&'a Expression<'a>> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        (!property.computed && property_key_name(&property.key) == Some(name))
            .then_some(&property.value)
    })
}

pub(crate) fn option_object_property<'a>(
    object: &'a ObjectExpression<'a>,
    name: &str,
) -> Option<&'a ObjectExpression<'a>> {
    option_expression_property(object, name).and_then(object_expression_from_expression)
}

pub(super) fn object_expression_from_expression<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object),
        Expression::ParenthesizedExpression(value) => {
            object_expression_from_expression(&value.expression)
        }
        Expression::TSAsExpression(value) => object_expression_from_expression(&value.expression),
        Expression::TSSatisfiesExpression(value) => {
            object_expression_from_expression(&value.expression)
        }
        Expression::TSNonNullExpression(value) => {
            object_expression_from_expression(&value.expression)
        }
        _ => None,
    }
}

pub(super) fn object_props_must_stay_in_value_scope(object: &ObjectExpression<'_>) -> bool {
    object.properties.iter().any(|property| match property {
        ObjectPropertyKind::SpreadProperty(_) => true,
        ObjectPropertyKind::ObjectProperty(property) => {
            property.method || expression_must_stay_in_value_scope(&property.value)
        }
    })
}

fn expression_must_stay_in_value_scope(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::ArrowFunctionExpression(_)
        | Expression::CallExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::NewExpression(_)
        | Expression::TSAsExpression(_)
        | Expression::TSInstantiationExpression(_)
        | Expression::TSNonNullExpression(_)
        | Expression::TSSatisfiesExpression(_)
        | Expression::TSTypeAssertion(_) => true,
        Expression::ArrayExpression(array) => array.elements.iter().any(|element| {
            matches!(element, ArrayExpressionElement::SpreadElement(_))
                || element
                    .as_expression()
                    .is_some_and(expression_must_stay_in_value_scope)
        }),
        Expression::ObjectExpression(object) => object_props_must_stay_in_value_scope(object),
        Expression::ParenthesizedExpression(value) => {
            expression_must_stay_in_value_scope(&value.expression)
        }
        _ => false,
    }
}

fn is_define_component_callee(callee: &Expression<'_>) -> bool {
    match callee {
        Expression::Identifier(id) => {
            matches!(id.name.as_str(), "defineComponent" | "_defineComponent")
        }
        Expression::StaticMemberExpression(member) => matches!(
            member.property.name.as_str(),
            "defineComponent" | "_defineComponent"
        ),
        _ => false,
    }
}

pub(crate) fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
        PropertyKey::StringLiteral(value) => Some(value.value.as_str()),
        _ => None,
    }
}

pub(crate) fn source_slice(source: &str, span: Span) -> Option<&str> {
    source.get(span.start as usize..span.end as usize)
}

pub(super) fn is_safe_value_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_' || first == '$')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$')
}
