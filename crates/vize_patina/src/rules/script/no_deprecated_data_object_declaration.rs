//! script/no-deprecated-data-object-declaration
//!
//! Disallow an object literal as the component `data` option.
//!
//! In Vue 2 a component's `data` option could be declared either as a function
//! returning the initial state or, for a root instance, as a plain object
//! literal. Vue 3 dropped the object-literal form: component `data` must be a
//! function so each instance receives a fresh state object and instances do not
//! share (and mutate) the same reactive object. Declaring `data` as an object
//! literal is therefore a Vue 2 -> 3 migration hazard.
//!
//! This is a port of the object-literal half of
//! [`vue/no-deprecated-data-object-declaration`](https://eslint.vuejs.org/rules/no-deprecated-data-object-declaration.html)
//! from eslint-plugin-vue: it flags `data` declared as an `ObjectExpression`
//! and leaves the function forms (`data() {}`, `data: function () {}`,
//! `data: () => ({})`) alone.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! export default {
//!   // `data` must be a function in Vue 3, not an object literal.
//!   data: {
//!     count: 0
//!   }
//! }
//! ```
//!
//! ### Valid
//! ```ts
//! export default {
//!   data() {
//!     return { count: 0 }
//!   }
//! }
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, ExportDefaultDeclarationKind, Expression,
    ObjectExpression, ObjectPropertyKind, Program, PropertyKey, Statement,
};
use oxc_span::Span;
use vize_carton::FxHashMap;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-deprecated-data-object-declaration",
    description: "Disallow an object literal as the component data option (Vue 3 requires a function)",
    default_severity: Severity::Error,
};

/// Disallow an object literal as the component `data` option.
pub struct NoDeprecatedDataObjectDeclaration;

impl ScriptRule for NoDeprecatedDataObjectDeclaration {
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
        let Some(data) = find_data_object_literal(options) else {
            return;
        };
        report(data, offset, result);
    }
}

/// The object-literal value of the `data` option, if `data` is declared as a
/// plain object (`data: { ... }`) rather than a function.
fn find_data_object_literal<'a>(
    options: &'a ObjectExpression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    for property in &options.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        // A method shorthand (`data() {}`) parses as an ObjectProperty whose
        // value is a FunctionExpression, so it is naturally excluded by the
        // `ObjectExpression` match below.
        if matches!(property_key_name(&property.key), Some("data"))
            && let Expression::ObjectExpression(object) = &property.value
        {
            return Some(object);
        }
    }
    None
}

fn report(data: &ObjectExpression<'_>, offset: usize, result: &mut ScriptLintResult) {
    let span: Span = data.span;
    let start = offset as u32 + span.start;
    let end = offset as u32 + span.end;
    let diagnostic = LintDiagnostic::error(
        META.name,
        "Object declaration on `data` is deprecated. Use a function that returns the object instead.",
        start,
        end,
    )
    .with_label("`data` is declared as an object literal", start, end)
    .with_help(
        "Vue 3 requires the component `data` option to be a function so each \
         instance gets a fresh state object. Wrap the object in a function, e.g. \
         `data() { return { count: 0 } }`.",
    );
    result.add_diagnostic(diagnostic);
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
