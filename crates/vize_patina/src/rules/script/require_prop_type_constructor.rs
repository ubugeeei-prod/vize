//! script/require-prop-type-constructor
//!
//! Require Options API prop `type` values to be constructors rather than string
//! literals.
//!
//! A prop's runtime `type` is matched against the constructor of the passed
//! value, so it must be the constructor *identifier* (`String`, `Number`,
//! `Boolean`, `Array`, `Object`, `Function`, `Symbol`, `Date`, ...). Writing the
//! type as a string literal (`type: "String"`) looks plausible but is wrong: Vue
//! compares the prop value's constructor against the string `"String"`, which
//! never matches, so the declared validation silently never fires.
//!
//! This is scoped to the Options API `props` option, in its object form. It
//! flags:
//!
//! * the shorthand where the prop value itself is the type
//!   (`count: "Number"`),
//! * an explicit `type` string literal (`count: { type: "Number" }`),
//! * string-literal entries inside a `type` array
//!   (`count: { type: ["Number", "String"] }`).
//!
//! Constructor identifiers and any other expression (e.g. an imported type, a
//! `() => T` validator) are left alone. Array-form `props: ['a', 'b']` declare
//! only names — no types — so they are never flagged.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! export default {
//!   props: {
//!     // The type should be the `String` constructor, not the string "String".
//!     name: "String",
//!     age: { type: "Number" },
//!     id: { type: ["String", "Number"] }
//!   }
//! }
//! ```
//!
//! ### Valid
//! ```ts
//! export default {
//!   props: {
//!     name: String,
//!     age: { type: Number },
//!     id: { type: [String, Number] }
//!   }
//! }
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, ExportDefaultDeclarationKind, Expression,
    ObjectExpression, ObjectPropertyKind, Program, PropertyKey, Statement, StringLiteral,
};
use vize_carton::{CompactString, FxHashMap};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/require-prop-type-constructor",
    description: "Require prop `type` values to be constructors rather than string literals",
    default_severity: Severity::Error,
};

/// Require Options API prop `type` values to be constructors, not string literals.
pub struct RequirePropTypeConstructor;

impl ScriptRule for RequirePropTypeConstructor {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    #[inline]
    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        let Some(options) = find_component_options(program) else {
            return;
        };
        let Some(props) = find_props_object(options) else {
            return;
        };
        check_props_object(props, offset, result);
    }
}

/// Inspect each entry of the `props` object for a string-literal type.
fn check_props_object(props: &ObjectExpression<'_>, offset: usize, result: &mut ScriptLintResult) {
    for property in &props.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        check_prop_type(&property.value, offset, result);
    }
}

/// Examine a single prop's value (the right-hand side of `name: <value>`).
///
/// In the Options API a prop value is one of:
/// * the type directly (`name: String` / the offending `name: "String"`),
/// * a descriptor object whose `type` carries the type
///   (`name: { type: String, required: true }`),
/// * an array shorthand of constructors (`name: [String, Number]`).
fn check_prop_type(value: &Expression<'_>, offset: usize, result: &mut ScriptLintResult) {
    match value {
        // Shorthand: the prop value *is* the type. A string literal here is the
        // same mistake as an explicit `type: "String"`.
        Expression::StringLiteral(literal) => report(literal, offset, result),
        // Array shorthand: `name: ["String", Number]` — flag string entries.
        Expression::ArrayExpression(array) => report_array_string_types(array, offset, result),
        // Descriptor object: inspect its `type` property, if any.
        Expression::ObjectExpression(object) => {
            if let Some(type_value) = find_type_value(object) {
                report_type_value(type_value, offset, result);
            }
        }
        _ => {}
    }
}

/// The expression bound to the `type` property of a prop descriptor object, if
/// present (`{ type: <value>, ... }`).
fn find_type_value<'a>(object: &'a ObjectExpression<'a>) -> Option<&'a Expression<'a>> {
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        if matches!(property_key_name(&property.key), Some("type")) {
            return Some(&property.value);
        }
    }
    None
}

/// Report a `type` value that is a string literal or an array containing string
/// literals. Other expressions (constructor identifiers, validators) are fine.
fn report_type_value(type_value: &Expression<'_>, offset: usize, result: &mut ScriptLintResult) {
    match type_value {
        Expression::StringLiteral(literal) => report(literal, offset, result),
        Expression::ArrayExpression(array) => report_array_string_types(array, offset, result),
        _ => {}
    }
}

/// Report every string-literal element of a `type` array
/// (`["String", "Number"]`). Non-string elements are left alone.
fn report_array_string_types(
    array: &oxc_ast::ast::ArrayExpression<'_>,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    for element in &array.elements {
        if let Some(Expression::StringLiteral(literal)) = element.as_expression() {
            report(literal, offset, result);
        }
    }
}

/// Emit the diagnostic for a single offending string-literal type.
fn report(literal: &StringLiteral<'_>, offset: usize, result: &mut ScriptLintResult) {
    let start = offset as u32 + literal.span.start;
    let end = offset as u32 + literal.span.end;

    let type_name = literal.value.as_str();
    let mut message = CompactString::with_capacity(type_name.len() + 40);
    message.push_str("Prop type \"");
    message.push_str(type_name);
    message.push_str("\" should be a constructor, not a string.");

    let mut help = CompactString::with_capacity(type_name.len() + 40);
    help.push_str("Use the constructor `");
    help.push_str(type_name);
    help.push_str("` (without quotes) as the prop type.");

    let diagnostic = LintDiagnostic::error(META.name, message, start, end)
        .with_label("string literal used as a prop type", start, end)
        .with_help(help);
    result.add_diagnostic(diagnostic);
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

/// The `props` option object on a resolved component options object, if it is
/// declared in object form (`props: { ... }`). Array-form `props` carry only
/// names and are skipped.
fn find_props_object<'a>(options: &'a ObjectExpression<'a>) -> Option<&'a ObjectExpression<'a>> {
    for property in &options.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        if matches!(property_key_name(&property.key), Some("props"))
            && let Expression::ObjectExpression(object) = &property.value
        {
            return Some(object);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Component options resolution (export default / defineComponent).
//
// Mirrors the resolution in `no_arrow_functions_in_watch` / `no_dupe_keys`: a
// plain object, an identifier bound to one, or a `defineComponent(...)` wrapper,
// optionally through TS expression wrappers.
// ---------------------------------------------------------------------------

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

#[cfg(test)]
mod tests;
