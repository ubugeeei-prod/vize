//! Shallow negative-proof guard only; shared OptionsDescriptor supplies positives.
//! No aliases, control flow, factory names or other option groups are resolved.

use oxc_ast::ast::{
    ArrayExpressionElement, ExportDefaultDeclarationKind, Expression, ObjectExpression,
    ObjectPropertyKind, Program, PropertyKey, Statement,
};

const MAX_ITEMS: usize = 4096;

pub(super) fn normal_setup_is_complete(program: &Program<'_>) -> bool {
    for statement in &program.body {
        // Named/default re-exports are not a direct, authenticated options object.
        if matches!(
            statement,
            Statement::ExportNamedDeclaration(_) | Statement::ExportAllDeclaration(_)
        ) {
            return false;
        }
        let Statement::ExportDefaultDeclaration(export) = statement else {
            continue;
        };
        let ExportDefaultDeclarationKind::ObjectExpression(object) = &export.declaration else {
            return false;
        };
        if object.properties.len() > MAX_ITEMS {
            return false;
        }
        let mut setup_seen = false;
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return false;
            };
            if property.computed {
                return false;
            }
            let name = match &property.key {
                PropertyKey::StaticIdentifier(key) => key.name.as_str(),
                PropertyKey::StringLiteral(key) => key.value.as_str(),
                _ => return false,
            };
            match name {
                "props" | "inject" if !object_or_string_array_keys_complete(&property.value) => {
                    return false;
                }
                "methods" | "computed" if !object_value_keys_complete(&property.value) => {
                    return false;
                }
                "data" if !returned_keys_complete(&property.value) => return false,
                "setup" => {
                    if setup_seen || !returned_keys_complete(&property.value) {
                        return false;
                    }
                    setup_seen = true;
                }
                // Vue's `$listeners` and `$scopedSlots` instance properties
                // remain framework-owned even when a component uses mixins or
                // extends another options object. Those composition surfaces
                // cannot turn the reserved names into setup-owned bindings.
                "mixins" | "extends" => {}
                _ => {}
            }
        }
    }
    true
}

fn returned_keys_complete(value: &Expression<'_>) -> bool {
    let statements = match value {
        Expression::FunctionExpression(function) => {
            if function.generator {
                return false;
            }
            let Some(body) = &function.body else {
                return false;
            };
            &body.statements
        }
        Expression::ArrowFunctionExpression(arrow) if arrow.expression => {
            let [Statement::ExpressionStatement(statement)] = arrow.body.statements.as_slice()
            else {
                return false;
            };
            return static_return_keys(&statement.expression);
        }
        Expression::ArrowFunctionExpression(arrow) => &arrow.body.statements,
        _ => return false,
    };
    if statements.len() > MAX_ITEMS {
        return false;
    }
    for statement in statements {
        match statement {
            Statement::VariableDeclaration(_)
            | Statement::ExpressionStatement(_)
            | Statement::EmptyStatement(_) => {}
            Statement::ReturnStatement(returned) => {
                return returned.argument.as_ref().is_none_or(static_return_keys);
            }
            // Nested returns, branching and unsupported bodies stay Unknown.
            _ => return false,
        }
    }
    true
}

fn object_or_string_array_keys_complete(value: &Expression<'_>) -> bool {
    match value {
        Expression::ObjectExpression(object) => object_keys_complete(object),
        Expression::ArrayExpression(array) => {
            array.elements.len() <= MAX_ITEMS
                && array
                    .elements
                    .iter()
                    .all(|element| matches!(element, ArrayExpressionElement::StringLiteral(_)))
        }
        _ => false,
    }
}

fn object_value_keys_complete(value: &Expression<'_>) -> bool {
    matches!(value, Expression::ObjectExpression(object) if object_keys_complete(object))
}

fn static_return_keys(value: &Expression<'_>) -> bool {
    let Expression::ObjectExpression(object) = value.get_inner_expression() else {
        return false;
    };
    object_keys_complete(object)
}

fn object_keys_complete(object: &ObjectExpression<'_>) -> bool {
    object.properties.len() <= MAX_ITEMS
        && object.properties.iter().all(|property| {
            matches!(property, ObjectPropertyKind::ObjectProperty(property)
            if !property.computed && matches!(&property.key,
                PropertyKey::StaticIdentifier(_) | PropertyKey::StringLiteral(_)))
        })
}
