//! script/component-options-name-casing
//!
//! Enforce PascalCase for the component `name` option.
//!
//! Vue's style guide recommends PascalCase for component names. The `name`
//! option (and its `<script setup>` equivalent, `defineOptions({ name })`) is
//! used for `<KeepAlive>` `include`/`exclude`, recursive self-reference, and
//! devtools display, so a kebab-case, camelCase, or snake_case value here is
//! inconsistent with the recommended casing.
//!
//! This rule resolves the component options object — `export default {...}`,
//! `defineComponent({...})`, or an identifier bound to one (unwrapping TS
//! `as`/`satisfies`/non-null/parenthesized wrappers, mirroring `no_dupe_keys`
//! and `no_arrow_functions_in_watch`) — finds the `name` property whose value is
//! a string literal, and reports it when the string is not PascalCase (does not
//! match `^[A-Z][a-zA-Z0-9]*$`). The `<script setup>` `defineOptions({ name })`
//! macro call is checked the same way when it is cleanly reachable at the top
//! level. Non-string-literal `name` values are skipped.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! export default {
//!   name: 'my-component' // kebab-case
//! }
//! ```
//!
//! ### Valid
//! ```ts
//! export default {
//!   name: 'MyComponent'
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
    name: "script/component-options-name-casing",
    description: "Enforce PascalCase for the component `name` option",
    default_severity: Severity::Error,
};

/// Enforce PascalCase for the component `name` option.
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
        if let Some(options) = find_component_options(program)
            && let Some(name) = name_string_literal(options)
        {
            check_name(name, offset, result);
        }

        // `<script setup>`: `defineOptions({ name: "..." })`.
        if let Some(name) = define_options_name(program) {
            check_name(name, offset, result);
        }
    }
}

/// Report the `name` literal when its value is not PascalCase.
fn check_name(name: &StringLiteral<'_>, offset: usize, result: &mut ScriptLintResult) {
    let value = name.value.as_str();
    if is_pascal_case(value) {
        return;
    }

    let start = offset as u32 + name.span.start;
    let end = offset as u32 + name.span.end;

    let mut message = CompactString::with_capacity(value.len() + 48);
    message.push_str("Component name '");
    message.push_str(value);
    message.push_str("' is not PascalCase.");

    let diagnostic = LintDiagnostic::error(META.name, message, start, end)
        .with_label("expected PascalCase", start, end)
        .with_help(
            "Vue's style guide recommends PascalCase for component names; rename this \
             to PascalCase (e.g. `MyComponent`).",
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

/// The string-literal value of the `name` property of an options object.
fn name_string_literal<'a>(options: &'a ObjectExpression<'a>) -> Option<&'a StringLiteral<'a>> {
    for property in &options.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        if !matches!(property_key_name(&property.key), Some("name")) {
            continue;
        }
        if let Expression::StringLiteral(literal) = &property.value {
            return Some(literal);
        }
    }
    None
}

/// The `name` string literal from a top-level `defineOptions({ name: "..." })`
/// call, when one is cleanly reachable.
fn define_options_name<'a>(program: &'a Program<'a>) -> Option<&'a StringLiteral<'a>> {
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
        if !matches!(callee.name.as_str(), "defineOptions") {
            continue;
        }
        if let Some(Argument::ObjectExpression(object)) = call.arguments.first() {
            return name_string_literal(object);
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
