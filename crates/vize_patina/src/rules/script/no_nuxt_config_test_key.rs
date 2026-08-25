//! nuxt/no-nuxt-config-test-key
//!
//! Nuxt detects its test environment automatically. This reproduces
//! `@nuxt/eslint-plugin` 1.16.0's deliberately narrow syntax contract: inspect
//! only a default-exported object (or the first object argument of any
//! default-exported call), and reject direct identifier keys named `test` when
//! their value is a boolean literal.

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    Argument, CallExpression, ExportDefaultDeclarationKind, Expression, ObjectExpression,
    ObjectPropertyKind, Program, PropertyKey, Statement,
};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "nuxt/no-nuxt-config-test-key",
    description: "Disallow setting `test` key in Nuxt config",
    default_severity: Severity::Error,
};

const MESSAGE: &str =
    "Do not set `test` key in Nuxt config. The test environment is automatically detected.";

/// Disallow a boolean `test` key in a directly exported Nuxt config object.
pub struct NoNuxtConfigTestKey;

impl ScriptRule for NoNuxtConfigTestKey {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        for statement in &program.body {
            let Statement::ExportDefaultDeclaration(export) = statement else {
                continue;
            };
            let Some(object) = exported_object(&export.declaration) else {
                continue;
            };
            report_boolean_test_properties(object, offset, result);
        }
    }
}

fn exported_object<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object),
        ExportDefaultDeclarationKind::CallExpression(call) => first_object_argument(call),
        ExportDefaultDeclarationKind::ParenthesizedExpression(parenthesized) => {
            object_from_expression(&parenthesized.expression)
        }
        _ => None,
    }
}

fn object_from_expression<'a>(expression: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object),
        Expression::CallExpression(call) => first_object_argument(call),
        Expression::ParenthesizedExpression(parenthesized) => {
            object_from_expression(&parenthesized.expression)
        }
        _ => None,
    }
}

fn first_object_argument<'a>(call: &'a CallExpression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match call.arguments.first()? {
        Argument::ObjectExpression(object) => Some(object),
        Argument::ParenthesizedExpression(parenthesized) => {
            object_from_expression(&parenthesized.expression)
        }
        _ => None,
    }
}

fn report_boolean_test_properties(
    object: &ObjectExpression<'_>,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    for entry in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = entry else {
            continue;
        };
        if identifier_key_name(&property.key) != Some("test") || !is_boolean(&property.value) {
            continue;
        }
        result.add_diagnostic(LintDiagnostic::error(
            META.name,
            MESSAGE,
            offset as u32 + property.span.start,
            offset as u32 + property.span.end,
        ));
    }
}

fn identifier_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(key) => Some(key.name.as_str()),
        PropertyKey::Identifier(key) => Some(key.name.as_str()),
        PropertyKey::ParenthesizedExpression(parenthesized) => {
            identifier_expression_name(&parenthesized.expression)
        }
        _ => None,
    }
}

fn identifier_expression_name<'a>(expression: &'a Expression<'a>) -> Option<&'a str> {
    match expression {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        Expression::ParenthesizedExpression(parenthesized) => {
            identifier_expression_name(&parenthesized.expression)
        }
        _ => None,
    }
}

fn is_boolean(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::BooleanLiteral(_) => true,
        Expression::ParenthesizedExpression(parenthesized) => is_boolean(&parenthesized.expression),
        _ => false,
    }
}

#[cfg(test)]
mod tests;
