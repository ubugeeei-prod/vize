//! script/component-options-name-casing
//!
//! Enforce PascalCase for registered component names.
//!
//! Vue's style guide recommends PascalCase for names in the Options API
//! `components` registration object. A camelCase, kebab-case, or snake_case
//! key there makes the registered template name inconsistent with that style.
//!
//! This rule resolves the component options object — `export default {...}`,
//! `defineComponent({...})`, or an identifier bound to one (unwrapping TS
//! `as`/`satisfies`/non-null/parenthesized wrappers, mirroring `no_dupe_keys`
//! and `no_arrow_functions_in_watch`) — and checks the static keys of an inline
//! `components: { ... }` object. Computed keys, spreads, and component maps held
//! in a separate variable are skipped, matching `eslint-plugin-vue`.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! export default {
//!   components: { myComponent }
//! }
//! ```
//!
//! ### Valid
//! ```ts
//! export default {
//!   components: { MyComponent }
//! }
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, ExportDefaultDeclarationKind, Expression,
    ObjectExpression, ObjectPropertyKind, Program, PropertyKey, Statement,
};
use vize_l0::{CompactString, FxHashMap};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/component-options-name-casing",
    description: "Enforce PascalCase for registered component names",
    default_severity: Severity::Error,
};

/// Enforce PascalCase for registered component names.
pub struct ComponentOptionsNameCasing;

impl ScriptRule for ComponentOptionsNameCasing {
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
        // Options API: `export default {...}` / `defineComponent({...})` / an
        // identifier bound to an options object.
        if let Some(options) = find_component_options(program) {
            check_registered_components(options, offset, result);
        }
    }
}

fn check_registered_components(
    options: &ObjectExpression<'_>,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    for property in &options.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed || property_key_name(&property.key) != Some("components") {
            continue;
        }
        let Expression::ObjectExpression(components) = &property.value else {
            continue;
        };
        for registration in &components.properties {
            let ObjectPropertyKind::ObjectProperty(registration) = registration else {
                continue;
            };
            if registration.computed {
                continue;
            }
            let Some((name, start, end)) = static_property_key(&registration.key) else {
                continue;
            };
            check_name(name, start, end, offset, result);
        }
    }
}

/// Report a component registration key when it is not PascalCase.
fn check_name(
    name: &str,
    local_start: u32,
    local_end: u32,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    if is_pascal_case(name) {
        return;
    }

    let start = offset as u32 + local_start;
    let end = offset as u32 + local_end;

    let mut message = CompactString::with_capacity(name.len() + 48);
    message.push_str("Component name '");
    message.push_str(name);
    message.push_str("' is not PascalCase.");

    let diagnostic = LintDiagnostic::error(META.name, message, start, end)
        .with_label("expected PascalCase", start, end)
        .with_help(
            "Rename this registered component key to PascalCase (for example, \
             `MyComponent`).",
        );
    result.add_diagnostic(diagnostic);
}

/// Whether `value` matches `^[A-Z][a-zA-Z0-9]*$` (PascalCase).
fn is_pascal_case(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) if first.is_ascii_uppercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric())
}

fn static_property_key<'a>(key: &'a PropertyKey<'a>) -> Option<(&'a str, u32, u32)> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some((
            identifier.name.as_str(),
            identifier.span.start,
            identifier.span.end,
        )),
        PropertyKey::StringLiteral(literal) => {
            Some((literal.value.as_str(), literal.span.start, literal.span.end))
        }
        _ => None,
    }
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
// Mirrors the resolution in `no_dupe_keys` / `no_arrow_functions_in_watch`: a
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
