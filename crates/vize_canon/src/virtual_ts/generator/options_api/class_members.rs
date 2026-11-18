//! Class-component members a template may assign.
//!
//! vue-class-component turns a `get`/`set` accessor pair into a `{ get, set }`
//! computed, and vue-property-decorator's `@PropSync`, `@ModelSync` and
//! `@VModel` turn their field into a computed whose setter emits the update
//! event. Both bind like a `computed` member (`BindingType::Options`, #13335),
//! so, like a writable Options API computed, they accept a template assignment
//! such as `@click="open = !open"`. A getter-only accessor stays read-only.

use oxc_ast::ast::{
    Class, ClassElement, Decorator, ExportDefaultDeclarationKind, Expression, MethodDefinitionKind,
    Program, Statement,
};
use vize_carton::{FxHashSet, String};

use super::property_key_name;

/// Add the writable members of a class default export to `writable`.
pub(super) fn extend_writable_class_members(
    program: &Program<'_>,
    writable: &mut FxHashSet<String>,
) {
    let Some(class) = program.body.iter().find_map(|statement| match statement {
        Statement::ExportDefaultDeclaration(export) => class_from_export(&export.declaration),
        _ => None,
    }) else {
        return;
    };
    for element in &class.body.body {
        let key = match element {
            ClassElement::MethodDefinition(method)
                if method.kind == MethodDefinitionKind::Set
                    && !method.r#static
                    && !method.computed =>
            {
                &method.key
            }
            ClassElement::PropertyDefinition(property)
                if !property.r#static
                    && !property.computed
                    && !property.declare
                    && has_computed_wrapper_decorator(&property.decorators) =>
            {
                &property.key
            }
            ClassElement::AccessorProperty(accessor)
                if !accessor.r#static
                    && !accessor.computed
                    && has_computed_wrapper_decorator(&accessor.decorators) =>
            {
                &accessor.key
            }
            _ => continue,
        };
        if let Some(name) = property_key_name(key) {
            writable.insert(String::from(name));
        }
    }
}

fn class_from_export<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
) -> Option<&'a Class<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ClassDeclaration(class) => Some(class),
        ExportDefaultDeclarationKind::ParenthesizedExpression(parenthesized) => {
            class_from_expression(&parenthesized.expression)
        }
        ExportDefaultDeclarationKind::TSAsExpression(ts_as) => {
            class_from_expression(&ts_as.expression)
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(ts_satisfies) => {
            class_from_expression(&ts_satisfies.expression)
        }
        ExportDefaultDeclarationKind::TSNonNullExpression(ts_non_null) => {
            class_from_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

fn class_from_expression<'a>(expression: &'a Expression<'a>) -> Option<&'a Class<'a>> {
    match expression.without_parentheses() {
        Expression::ClassExpression(class) => Some(class),
        Expression::TSAsExpression(ts_as) => class_from_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            class_from_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            class_from_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

/// `@PropSync`, `@ModelSync` and `@VModel`, bare or called.
fn has_computed_wrapper_decorator(decorators: &[Decorator<'_>]) -> bool {
    decorators.iter().any(|decorator| {
        let callee = match &decorator.expression {
            Expression::CallExpression(call) => &call.callee,
            expression => expression,
        };
        matches!(
            callee,
            Expression::Identifier(identifier)
                if matches!(identifier.name.as_str(), "PropSync" | "ModelSync" | "VModel")
        )
    })
}
