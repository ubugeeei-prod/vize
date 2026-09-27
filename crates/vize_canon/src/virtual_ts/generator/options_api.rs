//! Options API template-binding emission for the virtual TypeScript generator.

use oxc_ast::ast::{
    Argument, CallExpression, ExportDefaultDeclarationKind, Expression, ObjectExpression,
    ObjectPropertyKind, Program, PropertyKey, Statement,
};
use vize_croquis::Croquis;
use vize_croquis::facts::used_component_name_list;

use super::options_api_support::is_safe_value_identifier;
use vize_carton::{CompactString, FxHashSet, String};

mod computed;
mod default_export;
mod variables;

#[cfg(test)]
pub(super) use default_export::find_default_export_targets;
pub(super) use default_export::{OptionsApiScriptFacts, analyze_options_api_script};
pub(super) use variables::generate_options_api_variables;

fn unresolved_extends_template_names(
    summary: &Croquis,
    configured_globals: &FxHashSet<&str>,
    has_unresolved_extends: bool,
) -> Vec<String> {
    if !has_unresolved_extends {
        return Vec::new();
    }

    let type_export_names: FxHashSet<&str> = summary
        .type_exports
        .iter()
        .map(|export| export.name.as_str())
        .collect();
    let used_components: FxHashSet<CompactString> =
        used_component_name_list(summary).into_iter().collect();
    let mut names = crate::virtual_ts::script_facts::undefined_refs(summary)
        .iter()
        .filter_map(|reference| {
            let name = reference.name.as_str();
            if crate::virtual_ts::script_facts::contains_binding(summary, name)
                || configured_globals.contains(name)
                || type_export_names.contains(name)
                || used_components.contains(name)
                || !is_safe_value_identifier(name)
            {
                return None;
            }
            Some(String::from(name))
        })
        .collect::<Vec<_>>();
    for expression in &summary.template_expressions {
        collect_unresolved_extends_expression_names(
            &mut names,
            expression.content.as_str(),
            summary,
            configured_globals,
            &type_export_names,
            &used_components,
        );
        if let Some(guard) = expression.vif_guard.as_ref() {
            collect_unresolved_extends_expression_names(
                &mut names,
                guard.as_str(),
                summary,
                configured_globals,
                &type_export_names,
                &used_components,
            );
        }
    }
    names.sort();
    names.dedup();
    names
}

fn has_unresolved_extends<'a>(
    script: &str,
    program: &'a Program<'a>,
    options: &'a ObjectExpression<'a>,
) -> bool {
    if !script.contains("extends") || !script.contains("export default") {
        return false;
    }

    let Some(extends) = option_expression_property(options, "extends") else {
        return false;
    };

    let object_bindings = collect_object_expression_bindings(program);
    !is_resolved_options_target(extends, &object_bindings)
}

fn collect_object_expression_bindings<'a>(program: &'a Program<'a>) -> FxHashSet<&'a str> {
    let mut bindings = FxHashSet::default();
    for statement in program.body.iter() {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        for declarator in declaration.declarations.iter() {
            let oxc_ast::ast::BindingPattern::BindingIdentifier(id) = &declarator.id else {
                continue;
            };
            let Some(init) = declarator.init.as_ref() else {
                continue;
            };
            if object_expression_from_expression(init).is_some() {
                bindings.insert(id.name.as_str());
            }
        }
    }
    bindings
}

fn is_resolved_options_target<'a>(
    expression: &'a Expression<'a>,
    object_bindings: &FxHashSet<&'a str>,
) -> bool {
    match expression {
        Expression::ObjectExpression(_) => true,
        Expression::Identifier(identifier) => object_bindings.contains(identifier.name.as_str()),
        Expression::ParenthesizedExpression(parenthesized) => {
            is_resolved_options_target(&parenthesized.expression, object_bindings)
        }
        Expression::TSAsExpression(ts_as) => {
            is_resolved_options_target(&ts_as.expression, object_bindings)
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            is_resolved_options_target(&ts_satisfies.expression, object_bindings)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            is_resolved_options_target(&ts_non_null.expression, object_bindings)
        }
        _ => false,
    }
}

fn collect_unresolved_extends_expression_names(
    names: &mut Vec<String>,
    expression: &str,
    summary: &Croquis,
    configured_globals: &FxHashSet<&str>,
    type_export_names: &FxHashSet<&str>,
    used_components: &FxHashSet<CompactString>,
) {
    for identifier in vize_croquis::drawer::extract_identifiers_oxc(expression) {
        let name = identifier.as_str();
        if crate::virtual_ts::script_facts::contains_binding(summary, name)
            || configured_globals.contains(name)
            || type_export_names.contains(name)
            || used_components.contains(name)
            || !is_safe_value_identifier(name)
        {
            continue;
        }
        names.push(String::from(name));
    }
}

pub(super) fn option_expression_property<'a>(
    object: &'a ObjectExpression<'a>,
    key_name: &str,
) -> Option<&'a Expression<'a>> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property.computed || property_key_name(&property.key) != Some(key_name) {
            return None;
        }
        Some(&property.value)
    })
}

pub(super) fn component_options_from_program<'a>(
    program: &'a Program<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    program.body.iter().find_map(|statement| {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            return None;
        };
        component_options_from_export(&export.declaration)
    })
}

fn component_options_from_export<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object.as_ref()),
        ExportDefaultDeclarationKind::CallExpression(call) => component_options_from_call(call),
        ExportDefaultDeclarationKind::ParenthesizedExpression(parenthesized) => {
            component_options_from_expression(&parenthesized.expression)
        }
        ExportDefaultDeclarationKind::TSAsExpression(ts_as) => {
            component_options_from_expression(&ts_as.expression)
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(ts_satisfies) => {
            component_options_from_expression(&ts_satisfies.expression)
        }
        ExportDefaultDeclarationKind::TSNonNullExpression(ts_non_null) => {
            component_options_from_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

fn component_options_from_expression<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object.as_ref()),
        Expression::CallExpression(call) => component_options_from_call(call),
        Expression::ParenthesizedExpression(parenthesized) => {
            component_options_from_expression(&parenthesized.expression)
        }
        Expression::TSAsExpression(ts_as) => component_options_from_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            component_options_from_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            component_options_from_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

fn component_options_from_call<'a>(
    call: &'a CallExpression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    if !is_define_component_callee(&call.callee) {
        return None;
    }
    let first = call.arguments.first()?;
    match first {
        Argument::ObjectExpression(object) => Some(object.as_ref()),
        Argument::CallExpression(call) => component_options_from_call(call),
        Argument::ParenthesizedExpression(parenthesized) => {
            component_options_from_expression(&parenthesized.expression)
        }
        Argument::TSAsExpression(ts_as) => component_options_from_expression(&ts_as.expression),
        Argument::TSSatisfiesExpression(ts_satisfies) => {
            component_options_from_expression(&ts_satisfies.expression)
        }
        Argument::TSNonNullExpression(ts_non_null) => {
            component_options_from_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

fn is_define_component_callee(callee: &Expression<'_>) -> bool {
    match callee {
        Expression::Identifier(callee) => {
            matches!(callee.name.as_str(), "defineComponent" | "_defineComponent")
        }
        Expression::StaticMemberExpression(member) => {
            matches!(
                member.property.name.as_str(),
                "defineComponent" | "_defineComponent"
            )
        }
        _ => false,
    }
}

pub(super) fn option_object_property<'a>(
    object: &'a ObjectExpression<'a>,
    key_name: &str,
) -> Option<&'a ObjectExpression<'a>> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property.computed || property_key_name(&property.key) != Some(key_name) {
            return None;
        }
        object_expression_from_expression(&property.value)
    })
}

fn object_expression_from_expression<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object.as_ref()),
        Expression::ParenthesizedExpression(parenthesized) => {
            object_expression_from_expression(&parenthesized.expression)
        }
        Expression::TSAsExpression(ts_as) => object_expression_from_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            object_expression_from_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            object_expression_from_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

pub(super) fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}

pub(super) fn source_slice(script: &str, span: oxc_span::Span) -> Option<&str> {
    script.get(span.start as usize..span.end as usize)
}

pub(super) fn safe_identifier(name: &str) -> String {
    let mut result = String::default();
    for (index, ch) in name.chars().enumerate() {
        if (index == 0 && (ch.is_ascii_alphabetic() || ch == '_' || ch == '$'))
            || (index > 0 && (ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'))
        {
            result.push(ch);
        } else {
            result.push('_');
        }
    }
    if result.is_empty() {
        result.push('_');
    }
    result
}
