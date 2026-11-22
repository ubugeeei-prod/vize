//! Conservative resolution of classic Nuxt 2 plugin bindings.

use oxc_ast::ast::{
    BindingPattern, ExportDefaultDeclarationKind, Expression, Function, Statement,
    VariableDeclarationKind,
};

use crate::commands::project_nuxt::unwrap_expression;

pub(super) enum StaticPlugin<'a> {
    Arrow(&'a oxc_ast::ast::ArrowFunctionExpression<'a>),
    Function(&'a Function<'a>),
}

pub(super) fn resolve_static_plugin_from_export<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
    program_body: &'a [Statement<'a>],
) -> Option<StaticPlugin<'a>> {
    if let ExportDefaultDeclarationKind::FunctionDeclaration(function) = declaration {
        return Some(StaticPlugin::Function(function));
    }
    match unwrap_expression(declaration.as_expression()?) {
        Expression::Identifier(identifier) => {
            resolve_static_plugin(program_body, identifier.name.as_str())
        }
        expression => static_plugin_from_expression(expression),
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
                        .map(unwrap_expression)
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
        _ => None,
    }
}
