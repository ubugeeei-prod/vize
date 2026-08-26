//! `inheritAttrs` as the script declares it.
//!
//! The option is resolved like `script/component-options-name-casing`: the
//! `<script setup>` `defineOptions({ ... })` macro, plus the Options API object
//! reached through `export default {...}`, `defineComponent({...})`, or an
//! identifier bound to one (unwrapping TS `as`/`satisfies`/non-null and
//! parenthesized wrappers).

use oxc_ast::ast::{
    Argument, BindingPattern, BooleanLiteral, CallExpression, ExportDefaultDeclarationKind,
    Expression, ObjectExpression, ObjectPropertyKind, Program, PropertyKey, Statement,
};
use vize_s0::FxHashMap;

/// An `inheritAttrs` property the script states.
pub(super) enum InheritAttrs<'a> {
    /// `inheritAttrs: true` / `inheritAttrs: false`.
    Literal(&'a BooleanLiteral),
    /// Present, but not as a boolean literal — an identifier, a call, a
    /// computed key. The effective value is not knowable from the syntax, so
    /// the rule must not guess at it in either direction.
    Opaque,
}

/// Every `inheritAttrs` the block states, from the Options API object and from
/// `defineOptions({ ... })`.
pub(super) fn declared_inherit_attrs<'a>(program: &'a Program<'a>) -> Vec<InheritAttrs<'a>> {
    let mut declared = Vec::new();
    if let Some(options) = find_component_options(program) {
        declared.extend(inherit_attrs_property(options));
    }
    if let Some(options) = define_options_object(program) {
        declared.extend(inherit_attrs_property(options));
    }
    declared
}

/// The `inheritAttrs` property of an options object, if it has one.
fn inherit_attrs_property<'a>(options: &'a ObjectExpression<'a>) -> Option<InheritAttrs<'a>> {
    for property in &options.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed || !matches!(property_key_name(&property.key), Some("inheritAttrs")) {
            continue;
        }
        return Some(match &property.value {
            Expression::BooleanLiteral(literal) => InheritAttrs::Literal(literal),
            _ => InheritAttrs::Opaque,
        });
    }
    None
}

/// The object argument of a top-level `defineOptions({ ... })` call.
fn define_options_object<'a>(program: &'a Program<'a>) -> Option<&'a ObjectExpression<'a>> {
    for statement in program.body.iter() {
        let Statement::ExpressionStatement(expression) = statement else {
            continue;
        };
        let Expression::CallExpression(call) = &expression.expression else {
            continue;
        };
        let Expression::Identifier(callee) = &call.callee else {
            continue;
        };
        if callee.name.as_str() != "defineOptions" {
            continue;
        }
        if let Some(Argument::ObjectExpression(object)) = call.arguments.first() {
            return Some(object);
        }
    }
    None
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}

fn find_component_options<'a>(program: &'a Program<'a>) -> Option<&'a ObjectExpression<'a>> {
    let mut bindings: FxHashMap<&'a str, &'a ObjectExpression<'a>> = FxHashMap::default();

    for statement in program.body.iter() {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        for declarator in &declaration.declarations {
            let Some(init) = declarator.init.as_ref() else {
                continue;
            };
            if let BindingPattern::BindingIdentifier(id) = &declarator.id
                && let Some(object) = options_from_expression(init, &bindings)
            {
                bindings.insert(id.name.as_str(), object);
            }
        }
    }

    for statement in program.body.iter() {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            continue;
        };
        if let Some(object) = options_from_export(&export.declaration, &bindings) {
            return Some(object);
        }
    }

    None
}

fn options_from_export<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object),
        ExportDefaultDeclarationKind::CallExpression(call) => options_from_call(call, bindings),
        ExportDefaultDeclarationKind::Identifier(identifier) => {
            bindings.get(identifier.name.as_str()).copied()
        }
        ExportDefaultDeclarationKind::ParenthesizedExpression(paren) => {
            options_from_expression(&paren.expression, bindings)
        }
        ExportDefaultDeclarationKind::TSAsExpression(ts_as) => {
            options_from_expression(&ts_as.expression, bindings)
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(ts_satisfies) => {
            options_from_expression(&ts_satisfies.expression, bindings)
        }
        ExportDefaultDeclarationKind::TSNonNullExpression(ts_non_null) => {
            options_from_expression(&ts_non_null.expression, bindings)
        }
        _ => None,
    }
}

fn options_from_expression<'a>(
    expression: &'a Expression<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object),
        Expression::CallExpression(call) => options_from_call(call, bindings),
        Expression::Identifier(identifier) => bindings.get(identifier.name.as_str()).copied(),
        Expression::ParenthesizedExpression(paren) => {
            options_from_expression(&paren.expression, bindings)
        }
        Expression::TSAsExpression(ts_as) => options_from_expression(&ts_as.expression, bindings),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            options_from_expression(&ts_satisfies.expression, bindings)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            options_from_expression(&ts_non_null.expression, bindings)
        }
        _ => None,
    }
}

fn options_from_call<'a>(
    call: &'a CallExpression<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    let Expression::Identifier(callee) = &call.callee else {
        return None;
    };
    if !matches!(callee.name.as_str(), "defineComponent" | "_defineComponent") {
        return None;
    }
    match call.arguments.first()? {
        Argument::ObjectExpression(object) => Some(object),
        Argument::Identifier(identifier) => bindings.get(identifier.name.as_str()).copied(),
        argument => argument
            .as_expression()
            .and_then(|expression| options_from_expression(expression, bindings)),
    }
}
