//! Low-level parsing and AST/string helpers shared across the Nuxt detectors.

use std::path::Path;

use oxc_ast::ast::{
    Expression, ModuleExportName, ObjectExpression, ObjectPropertyKind, PropertyKey, Statement,
};
use oxc_span::SourceType;
use vize_carton::{String, ToCompactString};

use super::stubs::tracked_read_to_string;

pub(super) fn source_type_for_script_lang(lang: Option<&str>) -> SourceType {
    match lang {
        Some("tsx") => SourceType::tsx().with_module(true),
        Some("jsx") => SourceType::jsx().with_module(true),
        Some("js") => SourceType::default().with_module(true),
        _ => SourceType::default()
            .with_module(true)
            .with_typescript(true),
    }
}

pub(super) fn source_type_for_path(path: &Path) -> SourceType {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("tsx") => SourceType::tsx().with_module(true),
        Some("jsx") => SourceType::jsx().with_module(true),
        Some("js" | "mjs" | "cjs") => SourceType::default().with_module(true),
        _ => SourceType::default()
            .with_module(true)
            .with_typescript(true),
    }
}

pub(super) fn is_ts_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first == '$' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric())
}

pub(super) fn module_export_name<'a>(name: &'a ModuleExportName<'a>) -> Option<&'a str> {
    match name {
        ModuleExportName::IdentifierName(identifier) => Some(identifier.name.as_str()),
        ModuleExportName::IdentifierReference(identifier) => Some(identifier.name.as_str()),
        ModuleExportName::StringLiteral(literal) => Some(literal.value.as_str()),
    }
}

pub(super) fn parse_module_specifier(from_part: &str) -> Option<&str> {
    let from_part = from_part.trim().trim_end_matches(';').trim();
    let quote = from_part.chars().next()?;
    if quote != '\'' && quote != '"' {
        return None;
    }
    let rest = &from_part[1..];
    let end = rest.find(quote)?;
    Some(&rest[..end])
}

pub(super) fn parse_export_names(export_part: &str) -> (&str, &str) {
    if let Some((local_name, exported_name)) = export_part.split_once(" as ") {
        (local_name.trim(), exported_name.trim())
    } else {
        (export_part, export_part)
    }
}

pub(super) fn normalize_component_binding_name(name: &str) -> Option<String> {
    let name = name.trim().trim_matches('"').trim_matches('\'');
    if name.is_empty() {
        return None;
    }
    if name.chars().enumerate().all(|(index, ch)| {
        ch == '_'
            || ch == '$'
            || (ch.is_ascii_alphanumeric() && (index > 0 || !ch.is_ascii_digit()))
    }) {
        return Some(name.to_compact_string());
    }
    None
}

pub(super) fn extract_call_expression_from_export<'a>(
    expr: &'a oxc_ast::ast::ExportDefaultDeclarationKind<'a>,
) -> Option<&'a oxc_ast::ast::CallExpression<'a>> {
    match expr {
        oxc_ast::ast::ExportDefaultDeclarationKind::CallExpression(call) => Some(call),
        oxc_ast::ast::ExportDefaultDeclarationKind::ParenthesizedExpression(paren) => {
            extract_call_expression(&paren.expression)
        }
        oxc_ast::ast::ExportDefaultDeclarationKind::TSAsExpression(ts_as) => {
            extract_call_expression(&ts_as.expression)
        }
        oxc_ast::ast::ExportDefaultDeclarationKind::TSSatisfiesExpression(ts_satisfies) => {
            extract_call_expression(&ts_satisfies.expression)
        }
        oxc_ast::ast::ExportDefaultDeclarationKind::TSNonNullExpression(ts_non_null) => {
            extract_call_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

pub(super) fn extract_call_expression<'a>(
    expr: &'a Expression<'a>,
) -> Option<&'a oxc_ast::ast::CallExpression<'a>> {
    match expr {
        Expression::CallExpression(call) => Some(call),
        Expression::ParenthesizedExpression(paren) => extract_call_expression(&paren.expression),
        Expression::TSAsExpression(ts_as) => extract_call_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            extract_call_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            extract_call_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

pub(super) fn extract_object_expression<'a>(
    expr: &'a Expression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match expr {
        Expression::ObjectExpression(object) => Some(object),
        Expression::ParenthesizedExpression(paren) => extract_object_expression(&paren.expression),
        Expression::TSAsExpression(ts_as) => extract_object_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            extract_object_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            extract_object_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

pub(super) fn extract_expression<'a>(expr: &'a Expression<'a>) -> Option<&'a Expression<'a>> {
    match expr {
        Expression::ParenthesizedExpression(paren) => extract_expression(&paren.expression),
        Expression::TSAsExpression(ts_as) => extract_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            extract_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => extract_expression(&ts_non_null.expression),
        _ => Some(expr),
    }
}

pub(super) fn static_property_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

pub(super) fn find_object_property<'a>(
    object: &'a ObjectExpression<'a>,
    name: &str,
) -> Option<&'a Expression<'a>> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if static_property_name(&property.key) == Some(name) {
            Some(&property.value)
        } else {
            None
        }
    })
}

/// Finds the config object behind the default export, looking through the
/// `defineNuxtConfig(...)` wrapper as well as parenthesized/`as`/`satisfies`
/// wrappers on either form.
pub(super) fn default_export_config_object<'a>(
    statements: &'a [Statement<'a>],
) -> Option<&'a ObjectExpression<'a>> {
    let export = statements.iter().find_map(|statement| match statement {
        Statement::ExportDefaultDeclaration(export) => Some(export),
        _ => None,
    })?;

    if let Some(call) = extract_call_expression_from_export(&export.declaration) {
        if !matches!(&call.callee, Expression::Identifier(callee) if callee.name == "defineNuxtConfig")
        {
            return None;
        }
        return call
            .arguments
            .first()
            .and_then(|argument| argument.as_expression())
            .and_then(extract_object_expression);
    }

    export
        .declaration
        .as_expression()
        .and_then(extract_object_expression)
}

pub(super) fn nuxt_config_source(cwd: &Path) -> String {
    for file_name in ["nuxt.config.ts", "nuxt.config.js", "nuxt.config.mts"] {
        let path = cwd.join(file_name);
        if let Ok(source) = tracked_read_to_string(path.as_path()) {
            return source.into();
        }
    }
    String::default()
}

pub(super) fn collect_object_keys(object: &ObjectExpression<'_>, keys: &mut Vec<String>) {
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        let Some(name) = static_property_name(&property.key) else {
            continue;
        };
        keys.push(name.to_compact_string());
    }
}
