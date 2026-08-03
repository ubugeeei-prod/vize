//! nuxt/nuxt-config-keys-order
//!
//! Reproduce `@nuxt/eslint-plugin` 1.16.0's config ordering and fixes, while
//! reporting the first authored inversion instead of the whole config object.
//! This includes official-module positions, the `$environment` group,
//! unknown-key collation, comment-preserving edits, and nested environments.

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{Fix, LintDiagnostic, Severity, TextEdit};
use oxc_ast::ast::{
    Argument, ExportDefaultDeclarationKind, Expression, ObjectExpression, ObjectPropertyKind,
    Program, PropertyKey, Statement,
};
use oxc_span::GetSpan;
use vize_carton::{String, cstr};

#[path = "nuxt_config_keys_order_support.rs"]
mod support;
use support::{
    first_order_inversion, property_display_name, property_name, property_text_ranges,
    sort_named_segments,
};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "nuxt/nuxt-config-keys-order",
    description: "Prefer recommended order of Nuxt config properties",
    default_severity: Severity::Error,
};

/// Sort direct properties in a default-exported Nuxt config object.
pub struct NuxtConfigKeysOrder;

impl ScriptRule for NuxtConfigKeysOrder {
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
        source: &str,
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
            report_sort(object, source, offset, result);
            report_environment_sorts(object, source, offset, result);
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

fn first_object_argument<'a>(
    call: &'a oxc_ast::ast::CallExpression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match call.arguments.first()? {
        Argument::ObjectExpression(object) => Some(object),
        Argument::ParenthesizedExpression(parenthesized) => {
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

fn report_environment_sorts(
    object: &ObjectExpression<'_>,
    source: &str,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    for entry in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = entry else {
            continue;
        };
        let Some(name) = identifier_key_name(&property.key) else {
            continue;
        };
        if name.starts_with('$')
            && let Some(environment) = object_from_expression(&property.value)
        {
            report_sort(environment, source, offset, result);
        }
    }
}

fn report_sort(
    object: &ObjectExpression<'_>,
    source: &str,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    if object.properties.len() < 2 {
        return;
    }

    let names = object
        .properties
        .iter()
        .map(|property| property_name(property, source))
        .collect::<Vec<_>>();
    let Some((misplaced_index, expected_after_index)) = first_order_inversion(&names) else {
        return;
    };
    let mut reordered = (0..object.properties.len()).collect::<Vec<_>>();
    sort_named_segments(&mut reordered, &names);

    let (range_start, range_end, pieces) = property_text_ranges(object, source);
    let mut replacement = String::new("");
    for index in &reordered {
        replacement.push_str(&pieces[*index]);
    }
    let misplaced = property_display_name(&object.properties[misplaced_index], source)
        .or_else(|| names[misplaced_index].clone())
        .unwrap_or_else(|| "unknown".into());
    let expected_after = property_display_name(&object.properties[expected_after_index], source)
        .or_else(|| names[expected_after_index].clone())
        .unwrap_or_else(|| "unknown".into());
    let misplaced_span = object.properties[misplaced_index].span();
    let start = offset as u32 + misplaced_span.start;
    let end = offset as u32 + misplaced_span.end;
    result.add_diagnostic(
        LintDiagnostic::error(
            META.name,
            cstr!("Expected config key \"{misplaced}\" to come after \"{expected_after}\""),
            start,
            end,
        )
        .with_fix(Fix::new(
            "Sort Nuxt config keys",
            TextEdit::replace(
                offset as u32 + range_start as u32,
                offset as u32 + range_end as u32,
                replacement,
            ),
        )),
    );
}

fn identifier_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::Identifier(identifier) => Some(identifier.name.as_str()),
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

#[cfg(test)]
#[path = "nuxt_config_keys_order_tests.rs"]
mod tests;
