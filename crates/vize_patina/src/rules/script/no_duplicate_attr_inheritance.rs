//! script/no-duplicate-attr-inheritance
//!
//! Flag an explicit `inheritAttrs: true` component option as redundant.
//!
//! Ports the redundant-by-default core of
//! [`vue/no-duplicate-attr-inheritance`](https://eslint.vuejs.org/rules/no-duplicate-attr-inheritance.html),
//! which warns about *double attribute application*: a component that keeps the
//! default attribute inheritance (`inheritAttrs: true`) **and** also forwards
//! `$attrs` manually (typically `v-bind="$attrs"` in the template) ends up
//! applying the fallthrough attributes twice.
//!
//! ## Scope (conservative subset)
//!
//! eslint's rule pairs a template signal (`v-bind="$attrs"`) with a script
//! signal (the effective `inheritAttrs` value). In the patina architecture
//! markup-rules and script-rules are separate passes: a script rule cannot
//! observe the `v-bind="$attrs"` in the `<template>`. Rather than approximate
//! that cross-block relationship unsoundly (and risk false positives on the many
//! components that *don't* forward `$attrs`), this rule implements the sound,
//! template-independent subset: an **explicit** `inheritAttrs: true` is always
//! redundant, because `true` is the framework default, so removing it changes
//! nothing. That is exactly the location eslint also reports when both signals
//! are present, and because `true` is unconditionally the default the rule fires
//! with **zero false positives**. Components that opt out with
//! `inheritAttrs: false`, or that omit the option, are never touched; the
//! template-side check (`v-bind="$attrs"` with an *implicit* default) is out of
//! scope.
//!
//! The option is resolved like `script/component-options-name-casing`: the
//! `<script setup>` `defineOptions({ inheritAttrs: true })` macro, plus the
//! Options API object reached through `export default {...}`,
//! `defineComponent({...})`, or an identifier bound to one (unwrapping TS
//! `as`/`satisfies`/non-null/parenthesized wrappers).
//!
//! ## Examples
//!
//! ### Invalid (redundant — `true` is the default)
//! ```ts
//! defineOptions({ inheritAttrs: true })
//! export default { inheritAttrs: true }
//! ```
//!
//! ### Valid
//! ```ts
//! defineOptions({ inheritAttrs: false }) // intentional opt-out
//! export default {}                      // default inheritance, unstated
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    Argument, BindingPattern, BooleanLiteral, CallExpression, ExportDefaultDeclarationKind,
    Expression, ObjectExpression, ObjectPropertyKind, Program, PropertyKey, Statement,
};
use vize_carton::FxHashMap;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-duplicate-attr-inheritance",
    description: "Flag an explicit `inheritAttrs: true` option as redundant (true is the default)",
    default_severity: Severity::Warning,
};

const MESSAGE: &str = "`inheritAttrs: true` is redundant because it is the default.";
const LABEL: &str = "redundant default";
const HELP: &str = "Remove `inheritAttrs: true`: attribute inheritance is already on by default. \
     Keep this option only to opt out with `inheritAttrs: false` (for example when forwarding \
     `$attrs` manually with `v-bind=\"$attrs\"`).";

/// Flag an explicit `inheritAttrs: true` component option as redundant.
pub struct NoDuplicateAttrInheritance;

impl ScriptRule for NoDuplicateAttrInheritance {
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
            && let Some(literal) = inherit_attrs_true_literal(options)
        {
            report(literal, offset, result);
        }

        // `<script setup>`: `defineOptions({ inheritAttrs: true })`.
        if let Some(literal) = define_options_inherit_attrs_true(program) {
            report(literal, offset, result);
        }
    }
}

/// Report the redundant `inheritAttrs: true` boolean literal.
fn report(literal: &BooleanLiteral, offset: usize, result: &mut ScriptLintResult) {
    let start = offset as u32 + literal.span.start;
    let end = offset as u32 + literal.span.end;
    result.add_diagnostic(
        LintDiagnostic::warn(META.name, MESSAGE, start, end)
            .with_label(LABEL, start, end)
            .with_help(HELP),
    );
}

/// The `true` boolean literal of an `inheritAttrs: true` property, if present.
///
/// Only a literal `true` is flagged; `inheritAttrs: false`, a computed key, a
/// shorthand, or any non-boolean-literal value (an identifier, a call, ...) is
/// left alone so the rule never guesses at a value it cannot see.
fn inherit_attrs_true_literal<'a>(options: &'a ObjectExpression<'a>) -> Option<&'a BooleanLiteral> {
    for property in &options.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        if !matches!(property_key_name(&property.key), Some("inheritAttrs")) {
            continue;
        }
        if let Expression::BooleanLiteral(literal) = &property.value
            && literal.value
        {
            return Some(literal);
        }
    }
    None
}

/// The `inheritAttrs: true` literal from a top-level
/// `defineOptions({ inheritAttrs: true })` call, when one is cleanly reachable.
fn define_options_inherit_attrs_true<'a>(program: &'a Program<'a>) -> Option<&'a BooleanLiteral> {
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
            return inherit_attrs_true_literal(object);
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
// Mirrors the resolution in `component_options_name_casing` / `no_dupe_keys`: a
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
#[path = "no_duplicate_attr_inheritance_tests.rs"]
mod tests;
