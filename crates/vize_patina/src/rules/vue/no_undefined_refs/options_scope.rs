//! Names an Options API component puts on the instance its template reads.
//!
//! Lint draws a normal `<script>` without Options API bindings, so template
//! reads of `props`, `inject`, `computed`, `methods`, `data`, `setup()` and
//! Nuxt's `asyncData()` members arrive unresolved. The literal members are
//! accepted here. Members that come from somewhere a single file cannot see,
//! such as `mixins`, `extends`, `...mapState(...)` or a wrapper whose options
//! are not a literal object, make the whole component unresolved: the rule
//! then reports nothing for it.

use crate::context::LintContext;
use oxc_ast::ast::{
    Argument, ArrayExpression, ArrayExpressionElement, Expression, ObjectExpression,
    ObjectPropertyKind, Program, PropertyKey, Statement,
};
use vize_croquis::croquis::{OptionsDescriptor, normalize_options_template_binding_name};
use vize_croquis::script_parser::collect_options_object;
use vize_l0::String;

/// The instance names the component defines, or `None` when its members
/// cannot be resolved statically.
pub(super) fn instance_names(ctx: &LintContext<'_>) -> Option<Vec<String>> {
    if let Some(analysis) = ctx.art_script_analysis {
        return Some(descriptor_names(analysis.options_descriptor.as_ref()));
    }
    let analysis_names = || {
        descriptor_names(
            ctx.analysis()
                .and_then(|analysis| analysis.options_descriptor.as_ref()),
        )
    };
    let Some(parsed) = ctx.sfc_scripts().and_then(|scripts| scripts.script()) else {
        return Some(analysis_names());
    };
    let program = &parsed.program;
    // Croquis's descriptor already lists the members of the options objects
    // it resolves; `defineNuxtComponent` options list their own here.
    let (object, mut names, members) = if let Some(object) = collect_options_object(program) {
        (object, analysis_names(), Members::AsyncData)
    } else {
        match exported_wrapper_options(program) {
            Exported::None => return Some(analysis_names()),
            Exported::Unresolved => return None,
            Exported::Nuxt(object) => (object, Vec::new(), Members::All),
        }
    };
    resolve_members(object, &mut names, members).then_some(names)
}

/// Which literal members [`resolve_members`] adds to the names.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Members {
    /// Only the `asyncData()` return keys, which croquis does not track.
    AsyncData,
    /// Every member group.
    All,
}

fn descriptor_names(descriptor: Option<&OptionsDescriptor>) -> Vec<String> {
    descriptor
        .map(|options| {
            options
                .members
                .iter()
                .filter_map(|member| normalize_options_template_binding_name(member.name.as_str()))
                .collect()
        })
        .unwrap_or_default()
}

enum Exported<'a> {
    /// No `export default` that could carry component options.
    None,
    /// An exported component whose options are not a literal object.
    Unresolved,
    /// The literal options of `export default defineNuxtComponent({ ... })`.
    Nuxt(&'a ObjectExpression<'a>),
}

/// The default export croquis did not resolve to an options object: Nuxt's
/// `defineNuxtComponent({ ... })`, or a call or binding whose options live
/// elsewhere. Classes, functions and other declarations are left as before.
fn exported_wrapper_options<'a>(program: &'a Program<'a>) -> Exported<'a> {
    for statement in &program.body {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            continue;
        };
        let Some(expression) = export.declaration.as_expression() else {
            return Exported::None;
        };
        return match unwrap_expression(expression) {
            Expression::CallExpression(call) if is_define_nuxt_component(&call.callee) => call
                .arguments
                .first()
                .and_then(Argument::as_expression)
                .and_then(|options| match unwrap_expression(options) {
                    Expression::ObjectExpression(object) => Some(Exported::Nuxt(object)),
                    _ => None,
                })
                .unwrap_or(Exported::Unresolved),
            Expression::CallExpression(_) | Expression::Identifier(_) => Exported::Unresolved,
            _ => Exported::None,
        };
    }
    Exported::None
}

fn is_define_nuxt_component(callee: &Expression<'_>) -> bool {
    match callee {
        Expression::Identifier(identifier) => identifier.name == "defineNuxtComponent",
        Expression::StaticMemberExpression(member) => member.property.name == "defineNuxtComponent",
        _ => false,
    }
}

fn unwrap_expression<'e, 'a>(expression: &'e Expression<'a>) -> &'e Expression<'a> {
    match expression {
        Expression::ParenthesizedExpression(inner) => unwrap_expression(&inner.expression),
        Expression::TSAsExpression(inner) => unwrap_expression(&inner.expression),
        Expression::TSSatisfiesExpression(inner) => unwrap_expression(&inner.expression),
        Expression::TSNonNullExpression(inner) => unwrap_expression(&inner.expression),
        _ => expression,
    }
}

/// Whether every member the options object puts on the instance is literal,
/// adding the template spellings of the `members` it lists to `names`.
fn resolve_members(
    object: &ObjectExpression<'_>,
    names: &mut Vec<String>,
    members: Members,
) -> bool {
    object.properties.iter().all(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            // `...base` can carry any option.
            return false;
        };
        if property.computed {
            return false;
        }
        let Some(key) = key_name(&property.key) else {
            return false;
        };
        let listed = (members == Members::All).then_some(&mut *names);
        let value = unwrap_expression(&property.value);
        match key {
            "mixins" | "extends" => {
                matches!(value, Expression::ArrayExpression(array) if array.elements.is_empty())
            }
            "props" | "inject" => match value {
                Expression::ArrayExpression(array) => literal_strings(array, listed),
                Expression::ObjectExpression(object) => literal_keys(object, listed),
                _ => false,
            },
            "computed" | "methods" => {
                matches!(value, Expression::ObjectExpression(object) if literal_keys(object, listed))
            }
            "data" | "setup" => returned_members(value, listed),
            "asyncData" => returned_members(value, Some(&mut *names)),
            _ => true,
        }
    })
}

/// Whether an array-form `props`/`inject` lists only string literals,
/// pushing their template spellings into `names` when given.
fn literal_strings(array: &ArrayExpression<'_>, mut names: Option<&mut Vec<String>>) -> bool {
    array.elements.iter().all(|element| {
        let ArrayExpressionElement::StringLiteral(string) = element else {
            return false;
        };
        if let Some(names) = names.as_deref_mut()
            && let Some(name) = normalize_options_template_binding_name(string.value.as_str())
        {
            names.push(name);
        }
        true
    })
}

/// Whether a `data`/`setup`/`asyncData` value defines only literal keys:
/// a direct object, or a function whose top-level returns are all objects.
/// A function without a top-level return defines nothing.
fn returned_members(value: &Expression<'_>, mut names: Option<&mut Vec<String>>) -> bool {
    let statements = match value {
        Expression::ObjectExpression(object) => return literal_keys(object, names),
        Expression::FunctionExpression(function) => match function.body.as_ref() {
            Some(body) => &body.statements,
            None => return true,
        },
        Expression::ArrowFunctionExpression(arrow) if arrow.expression => {
            return match arrow.body.statements.first() {
                Some(Statement::ExpressionStatement(statement)) => {
                    match unwrap_expression(&statement.expression) {
                        Expression::ObjectExpression(object) => literal_keys(object, names),
                        _ => false,
                    }
                }
                _ => false,
            };
        }
        Expression::ArrowFunctionExpression(arrow) => &arrow.body.statements,
        _ => return false,
    };
    statements.iter().all(|statement| {
        let Statement::ReturnStatement(ret) = statement else {
            return true;
        };
        match ret.argument.as_ref().map(unwrap_expression) {
            None => true,
            Some(Expression::ObjectExpression(object)) => {
                literal_keys(object, names.as_deref_mut())
            }
            Some(_) => false,
        }
    })
}

/// Whether `object` has only literal keys, pushing their template spellings
/// into `names` when given.
fn literal_keys(object: &ObjectExpression<'_>, mut names: Option<&mut Vec<String>>) -> bool {
    object.properties.iter().all(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            // `...mapState([...])` and other spreads.
            return false;
        };
        let Some(key) = key_name(&property.key).filter(|_| !property.computed) else {
            return false;
        };
        if let Some(names) = names.as_deref_mut()
            && let Some(name) = normalize_options_template_binding_name(key)
        {
            names.push(name);
        }
        true
    })
}

fn key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
