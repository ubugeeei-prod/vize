//! script/no-boolean-default
//!
//! Disallow a `default` on a Boolean prop.
//!
//! HTML boolean attributes and Vue Boolean props already default to `false`, so
//! declaring an explicit `default` on a `type: Boolean` prop is redundant and
//! confusing — especially `default: true`, which silently inverts the usual
//! "absent means false" expectation.
//!
//! This is scoped to the Options API `props` *object* form. For each prop whose
//! declaration object has `type: Boolean` (the `Boolean` constructor, exactly)
//! and also carries a `default` property, the `default` property is reported.
//! Props whose `type` is an array union (e.g. `[Boolean, String]`) or any
//! non-`Boolean` constructor are left alone — only an unambiguous Boolean prop is
//! flagged.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! export default {
//!   props: {
//!     // Boolean props already default to false; an explicit default is confusing.
//!     disabled: { type: Boolean, default: true },
//!     checked: { type: Boolean, default: false }
//!   }
//! }
//! ```
//!
//! ### Valid
//! ```ts
//! export default {
//!   props: {
//!     // No explicit default: defaults to false.
//!     disabled: { type: Boolean },
//!     disabled2: Boolean,
//!     // Union type may legitimately need a default.
//!     value: { type: [Boolean, String], default: '' },
//!     // Non-Boolean prop.
//!     count: { type: Number, default: 0 }
//!   }
//! }
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, ExportDefaultDeclarationKind, Expression,
    ObjectExpression, ObjectProperty, ObjectPropertyKind, Program, PropertyKey, Statement,
};
use oxc_span::Span;
use vize_carton::FxHashMap;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-boolean-default",
    description: "Disallow a default on a Boolean prop",
    default_severity: Severity::Warning,
};

/// Disallow a `default` on a Boolean prop.
pub struct NoBooleanDefault;

impl ScriptRule for NoBooleanDefault {
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

/// Walk each prop in the `props` object; report the `default` of any prop whose
/// declaration object is `{ type: Boolean, ..., default: ... }`.
fn check_props_object(props: &ObjectExpression<'_>, offset: usize, result: &mut ScriptLintResult) {
    for property in &props.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        // Only object-form prop declarations can carry both `type` and `default`.
        let Expression::ObjectExpression(declaration) = &property.value else {
            continue;
        };
        if !prop_type_is_boolean(declaration) {
            continue;
        }
        if let Some(default) = find_property(declaration, "default") {
            report(default, offset, result);
        }
    }
}

/// Whether a prop declaration object has `type: Boolean` exactly — the `Boolean`
/// constructor identifier. An array union such as `[Boolean, String]` or any
/// other type expression returns `false`.
fn prop_type_is_boolean(declaration: &ObjectExpression<'_>) -> bool {
    let Some(type_property) = find_property(declaration, "type") else {
        return false;
    };
    matches!(&type_property.value, Expression::Identifier(id) if id.name == "Boolean")
}

/// The first own, non-computed property of `object` whose key is `name`.
fn find_property<'a>(
    object: &'a ObjectExpression<'a>,
    name: &str,
) -> Option<&'a ObjectProperty<'a>> {
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        if matches!(property_key_name(&property.key), Some(key) if key == name) {
            return Some(property);
        }
    }
    None
}

fn report(default: &ObjectProperty<'_>, offset: usize, result: &mut ScriptLintResult) {
    let span: Span = default.span;
    let start = offset as u32 + span.start;
    let end = offset as u32 + span.end;
    let diagnostic = LintDiagnostic::warn(
        META.name,
        "Unexpected default on a Boolean prop.",
        start,
        end,
    )
    .with_label("Boolean props already default to false", start, end)
    .with_help(
        "HTML boolean attributes and Vue Boolean props already default to `false`; \
         remove this `default` (an explicit `default: true` is especially confusing).",
    );
    result.add_diagnostic(diagnostic);
}

/// The `props` option resolved to its object form. Array-form `props` (e.g.
/// `['a', 'b']`) cannot carry a type/default and resolves to `None`.
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

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
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
