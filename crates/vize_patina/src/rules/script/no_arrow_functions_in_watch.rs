//! script/no-arrow-functions-in-watch
//!
//! Disallow arrow functions as Options API `watch` handlers.
//!
//! A `watch` handler declared in the Options API is invoked with `this` bound to
//! the component instance, so handlers routinely read `this.*`. An arrow
//! function captures `this` lexically instead of receiving the component
//! instance, so `this` inside it points at the surrounding (module) scope and
//! `this.*` is `undefined` — almost always a bug.
//!
//! This is scoped to the Options API `watch` option, covering both the shorthand
//! handler form (`key: () => {}`) and the object form's `handler` property
//! (`key: { handler: () => {} }`). The Composition API `watch(src, () => {})`
//! call form is intentionally **not** flagged: its callback has no `this`
//! expectation, so an arrow function there is correct.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! export default {
//!   watch: {
//!     // `this` is not the component instance inside an arrow function.
//!     value: () => {
//!       this.doSomething()
//!     },
//!     other: {
//!       handler: () => {}
//!     }
//!   }
//! }
//! ```
//!
//! ### Valid
//! ```ts
//! export default {
//!   watch: {
//!     value(newValue, oldValue) {
//!       this.doSomething()
//!     },
//!     other: {
//!       handler(newValue) {},
//!       deep: true
//!     }
//!   }
//! }
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    Argument, ArrowFunctionExpression, BindingPattern, CallExpression,
    ExportDefaultDeclarationKind, Expression, ObjectExpression, ObjectPropertyKind, Program,
    PropertyKey, Statement,
};
use oxc_span::Span;
use vize_carton::FxHashMap;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-arrow-functions-in-watch",
    description: "Disallow arrow functions as Options API watch handlers",
    default_severity: Severity::Error,
};

/// Disallow arrow functions as Options API `watch` handlers.
pub struct NoArrowFunctionsInWatch;

impl ScriptRule for NoArrowFunctionsInWatch {
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
        let Some(watch) = find_watch_object(options) else {
            return;
        };
        check_watch_object(watch, offset, result);
    }
}

fn check_watch_object(watch: &ObjectExpression<'_>, offset: usize, result: &mut ScriptLintResult) {
    for property in &watch.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        match &property.value {
            // Shorthand handler: `key: () => {}`.
            Expression::ArrowFunctionExpression(arrow) => {
                report(arrow, offset, result);
            }
            // Object form: `key: { handler: () => {}, deep: true }`.
            Expression::ObjectExpression(object) => {
                if let Some(arrow) = object_form_arrow_handler(object) {
                    report(arrow, offset, result);
                }
            }
            // Array form: `key: [handlerA, handlerB]` — flag any arrow entry.
            Expression::ArrayExpression(array) => {
                for element in &array.elements {
                    if let Some(Expression::ArrowFunctionExpression(arrow)) =
                        element.as_expression()
                    {
                        report(arrow, offset, result);
                    }
                }
            }
            _ => {}
        }
    }
}

/// The arrow function bound to the `handler` property of an object-form watch
/// entry (`{ handler: () => {} }`), if present.
fn object_form_arrow_handler<'a>(
    object: &'a ObjectExpression<'a>,
) -> Option<&'a ArrowFunctionExpression<'a>> {
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        if !matches!(property_key_name(&property.key), Some("handler")) {
            continue;
        }
        if let Expression::ArrowFunctionExpression(arrow) = &property.value {
            return Some(arrow);
        }
    }
    None
}

fn report(arrow: &ArrowFunctionExpression<'_>, offset: usize, result: &mut ScriptLintResult) {
    let span: Span = arrow.span;
    let start = offset as u32 + span.start;
    let end = offset as u32 + span.end;
    let diagnostic = LintDiagnostic::error(
        META.name,
        "Unexpected arrow function as a watch handler.",
        start,
        end,
    )
    .with_label("arrow function does not bind `this`", start, end)
    .with_help(
        "An Options API watch handler is called with `this` bound to the component \
         instance, but an arrow function captures `this` lexically. Use a regular \
         function (`handler(newValue, oldValue) {}`) instead.",
    );
    result.add_diagnostic(diagnostic);
}

fn find_watch_object<'a>(options: &'a ObjectExpression<'a>) -> Option<&'a ObjectExpression<'a>> {
    for property in &options.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        if matches!(property_key_name(&property.key), Some("watch"))
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
// Mirrors the resolution in `no_dupe_keys` / `no_side_effects_in_computed`: a
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
