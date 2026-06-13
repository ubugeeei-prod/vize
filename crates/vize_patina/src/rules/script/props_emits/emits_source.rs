//! Shared resolution of the Options API `emits` declaration object for the
//! `script/*` emits rules.
//!
//! Resolves the object form of the `emits` option
//! (`export default { emits: { ... } }` / `defineComponent({ emits: { ... } })`
//! / a same-file identifier bound to such an object). The array form
//! (`emits: ['submit']`) and type-based declarations carry no validator
//! functions, so only the object form is returned.

use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, ExportDefaultDeclarationKind, Expression,
    ObjectExpression, ObjectPropertyKind, Program, PropertyKey, Statement,
};

use vize_carton::FxHashMap;

/// The Options API `emits` option as an object literal, if declared in object
/// form. Resolves `export default` / `defineComponent(...)` / identifier-bound
/// options objects, and an identifier-bound `emits` value.
pub(super) fn resolve_emits_object<'a>(
    program: &'a Program<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    let bindings = collect_object_bindings(program);
    let options = resolve_options_object(program, &bindings)?;
    option_emits_object(options, &bindings)
}

/// The `emits` option's value as an object literal, resolving an
/// identifier-bound object when the value is a bare identifier.
fn option_emits_object<'a>(
    options: &'a ObjectExpression<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    options.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property.computed || property_key_name(&property.key) != Some("emits") {
            return None;
        }
        resolve_object_or_binding(&property.value, bindings)
    })
}

// ---------------------------------------------------------------------------
// Options-object resolution (export default / defineComponent / identifier).
// A self-contained subset mirroring `no_required_prop_with_default`.
// ---------------------------------------------------------------------------

fn collect_object_bindings<'a>(
    program: &'a Program<'a>,
) -> FxHashMap<&'a str, &'a ObjectExpression<'a>> {
    let mut bindings = FxHashMap::default();
    for statement in program.body.iter() {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        for declarator in declaration.declarations.iter() {
            let BindingPattern::BindingIdentifier(id) = &declarator.id else {
                continue;
            };
            if let Some(object) = declarator.init.as_ref().and_then(unwrap_object_expression) {
                bindings.entry(id.name.as_str()).or_insert(object);
            }
        }
    }
    bindings
}

fn resolve_options_object<'a>(
    program: &'a Program<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    program.body.iter().find_map(|statement| {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            return None;
        };
        match &export.declaration {
            ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object.as_ref()),
            ExportDefaultDeclarationKind::CallExpression(call) => {
                component_options_from_call(call, bindings)
            }
            ExportDefaultDeclarationKind::Identifier(identifier) => {
                bindings.get(identifier.name.as_str()).copied()
            }
            declaration => declaration
                .as_expression()
                .and_then(|expression| resolve_object_or_binding(expression, bindings)),
        }
    })
}

fn component_options_from_call<'a>(
    call: &'a CallExpression<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    if !is_define_component_callee(&call.callee) {
        return None;
    }
    match call.arguments.first()? {
        Argument::ObjectExpression(object) => Some(object.as_ref()),
        Argument::Identifier(identifier) => bindings.get(identifier.name.as_str()).copied(),
        argument => argument
            .as_expression()
            .and_then(|expression| resolve_object_or_binding(expression, bindings)),
    }
}

fn is_define_component_callee(callee: &Expression<'_>) -> bool {
    match callee {
        Expression::Identifier(identifier) => matches!(
            identifier.name.as_str(),
            "defineComponent" | "_defineComponent"
        ),
        Expression::StaticMemberExpression(member) => matches!(
            member.property.name.as_str(),
            "defineComponent" | "_defineComponent"
        ),
        _ => false,
    }
}

fn resolve_object_or_binding<'a>(
    expression: &'a Expression<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    if let Expression::Identifier(identifier) = expression {
        return bindings.get(identifier.name.as_str()).copied();
    }
    unwrap_object_expression(expression)
}

fn unwrap_object_expression<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object.as_ref()),
        Expression::ParenthesizedExpression(paren) => unwrap_object_expression(&paren.expression),
        Expression::TSAsExpression(ts) => unwrap_object_expression(&ts.expression),
        Expression::TSSatisfiesExpression(ts) => unwrap_object_expression(&ts.expression),
        Expression::TSNonNullExpression(ts) => unwrap_object_expression(&ts.expression),
        _ => None,
    }
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}
