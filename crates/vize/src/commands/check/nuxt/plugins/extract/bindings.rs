//! Conservative resolution of classic Nuxt 2 plugin bindings.

use oxc_ast::ast::{
    BindingPattern, ExportDefaultDeclarationKind, Expression, Function, Statement,
    VariableDeclarationKind,
};

pub(super) enum StaticPlugin<'a> {
    Arrow(&'a oxc_ast::ast::ArrowFunctionExpression<'a>),
    Function(&'a Function<'a>),
}

pub(super) fn resolve_static_plugin_from_export<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
    program_body: &'a [Statement<'a>],
) -> Option<StaticPlugin<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ArrowFunctionExpression(arrow) => {
            Some(StaticPlugin::Arrow(arrow))
        }
        ExportDefaultDeclarationKind::FunctionDeclaration(function)
        | ExportDefaultDeclarationKind::FunctionExpression(function) => {
            Some(StaticPlugin::Function(function))
        }
        ExportDefaultDeclarationKind::Identifier(identifier) => {
            resolve_static_plugin(program_body, identifier.name.as_str())
        }
        ExportDefaultDeclarationKind::ParenthesizedExpression(parenthesized) => {
            resolve_static_plugin_from_export_expression(&parenthesized.expression, program_body)
        }
        ExportDefaultDeclarationKind::TSAsExpression(ts_as) => {
            resolve_static_plugin_from_export_expression(&ts_as.expression, program_body)
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(ts_satisfies) => {
            resolve_static_plugin_from_export_expression(&ts_satisfies.expression, program_body)
        }
        ExportDefaultDeclarationKind::TSNonNullExpression(ts_non_null) => {
            resolve_static_plugin_from_export_expression(&ts_non_null.expression, program_body)
        }
        _ => None,
    }
}

fn resolve_static_plugin_from_export_expression<'a>(
    expression: &'a Expression<'a>,
    program_body: &'a [Statement<'a>],
) -> Option<StaticPlugin<'a>> {
    match expression {
        Expression::Identifier(identifier) => {
            resolve_static_plugin(program_body, identifier.name.as_str())
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            resolve_static_plugin_from_export_expression(&parenthesized.expression, program_body)
        }
        Expression::TSAsExpression(ts_as) => {
            resolve_static_plugin_from_export_expression(&ts_as.expression, program_body)
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            resolve_static_plugin_from_export_expression(&ts_satisfies.expression, program_body)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            resolve_static_plugin_from_export_expression(&ts_non_null.expression, program_body)
        }
        _ => static_plugin_from_expression(expression),
    }
}

/// Resolve direct local declarations only. Mutable/imported bindings, aliases,
/// and dynamic initializers stay unknown so they cannot fabricate inject keys.
fn resolve_static_plugin<'a>(
    program_body: &'a [Statement<'a>],
    name: &str,
) -> Option<StaticPlugin<'a>> {
    for statement in program_body {
        match statement {
            Statement::VariableDeclaration(declaration)
                if declaration.kind == VariableDeclarationKind::Const =>
            {
                for declarator in &declaration.declarations {
                    let BindingPattern::BindingIdentifier(identifier) = &declarator.id else {
                        continue;
                    };
                    if identifier.name.as_str() != name {
                        continue;
                    }
                    return declarator
                        .init
                        .as_ref()
                        .and_then(static_plugin_from_expression);
                }
            }
            Statement::FunctionDeclaration(function)
                if function
                    .id
                    .as_ref()
                    .is_some_and(|identifier| identifier.name.as_str() == name) =>
            {
                return Some(StaticPlugin::Function(function));
            }
            _ => {}
        }
    }
    None
}

fn static_plugin_from_expression<'a>(expression: &'a Expression<'a>) -> Option<StaticPlugin<'a>> {
    match expression {
        Expression::ArrowFunctionExpression(arrow) => Some(StaticPlugin::Arrow(arrow)),
        Expression::FunctionExpression(function) => Some(StaticPlugin::Function(function)),
        Expression::ParenthesizedExpression(parenthesized) => {
            static_plugin_from_expression(&parenthesized.expression)
        }
        Expression::TSAsExpression(ts_as) => static_plugin_from_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            static_plugin_from_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            static_plugin_from_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}
