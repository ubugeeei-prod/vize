//! AST extraction for Nuxt plugin provide/inject keys.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, BindingPattern, ExportDefaultDeclarationKind, Expression, Function, ObjectExpression,
    Statement,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::String;

use super::super::parsing::{
    collect_object_keys, extract_call_expression_from_export, extract_expression,
    extract_object_expression, find_object_property,
};

pub(in crate::commands::check::nuxt) fn extract_plugin_provide_keys_from_source(
    source: &str,
) -> Vec<String> {
    let allocator = Allocator::default();
    let source_type = SourceType::default()
        .with_module(true)
        .with_typescript(true);
    let ret = Parser::new(&allocator, source, source_type).parse();
    let mut keys = Vec::new();

    for statement in &ret.program.body {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            continue;
        };
        collect_plugin_keys_from_default_export(&export.declaration, &mut keys);
    }

    keys
}

fn collect_plugin_keys_from_default_export(
    declaration: &ExportDefaultDeclarationKind<'_>,
    keys: &mut Vec<String>,
) {
    if let Some(call) = extract_call_expression_from_export(declaration) {
        let Expression::Identifier(callee) = &call.callee else {
            return;
        };
        if callee.name.as_str() != "defineNuxtPlugin" {
            return;
        }
        if let Some(first_arg) = call.arguments.first() {
            collect_plugin_keys_from_argument(first_arg, keys);
        }
        return;
    }

    match declaration {
        ExportDefaultDeclarationKind::ArrowFunctionExpression(arrow) => {
            collect_plugin_keys_from_arrow_function(arrow, keys);
        }
        ExportDefaultDeclarationKind::FunctionDeclaration(function)
        | ExportDefaultDeclarationKind::FunctionExpression(function) => {
            collect_plugin_keys_from_function(function, keys);
        }
        ExportDefaultDeclarationKind::ParenthesizedExpression(parenthesized) => {
            collect_plugin_keys_from_expression(&parenthesized.expression, keys);
        }
        ExportDefaultDeclarationKind::TSAsExpression(ts_as) => {
            collect_plugin_keys_from_expression(&ts_as.expression, keys);
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(ts_satisfies) => {
            collect_plugin_keys_from_expression(&ts_satisfies.expression, keys);
        }
        ExportDefaultDeclarationKind::TSNonNullExpression(ts_non_null) => {
            collect_plugin_keys_from_expression(&ts_non_null.expression, keys);
        }
        _ => {}
    }
}

fn collect_plugin_keys_from_expression(expression: &Expression<'_>, keys: &mut Vec<String>) {
    match expression {
        Expression::ArrowFunctionExpression(arrow) => {
            collect_plugin_keys_from_arrow_function(arrow, keys);
        }
        Expression::FunctionExpression(function) => {
            collect_plugin_keys_from_function(function, keys)
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            collect_plugin_keys_from_expression(&parenthesized.expression, keys);
        }
        Expression::TSAsExpression(ts_as) => {
            collect_plugin_keys_from_expression(&ts_as.expression, keys);
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            collect_plugin_keys_from_expression(&ts_satisfies.expression, keys);
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            collect_plugin_keys_from_expression(&ts_non_null.expression, keys);
        }
        _ => {}
    }
}

fn collect_plugin_keys_from_argument(arg: &Argument<'_>, keys: &mut Vec<String>) {
    match arg {
        Argument::ObjectExpression(object) => collect_plugin_keys_from_object(object, keys),
        Argument::ArrowFunctionExpression(arrow) => {
            collect_plugin_keys_from_arrow_function(arrow, keys);
        }
        Argument::FunctionExpression(function) => {
            collect_plugin_keys_from_function(function, keys);
        }
        _ => {}
    }
}

fn collect_plugin_keys_from_arrow_function(
    arrow: &oxc_ast::ast::ArrowFunctionExpression<'_>,
    keys: &mut Vec<String>,
) {
    if let Some(inject_name) = nuxt2_inject_param_name(&arrow.params) {
        collect_plugin_keys_from_nuxt2_inject_calls(&arrow.body.statements, inject_name, keys);
    }
    collect_plugin_keys_from_function_body(&arrow.body.statements, keys);
}

fn collect_plugin_keys_from_function(function: &Function<'_>, keys: &mut Vec<String>) {
    let Some(body) = &function.body else {
        return;
    };
    if let Some(inject_name) = nuxt2_inject_param_name(&function.params) {
        collect_plugin_keys_from_nuxt2_inject_calls(&body.statements, inject_name, keys);
    }
    collect_plugin_keys_from_function_body(&body.statements, keys);
}

fn nuxt2_inject_param_name<'a>(params: &'a oxc_ast::ast::FormalParameters<'a>) -> Option<&'a str> {
    let param = params.items.get(1)?;
    binding_identifier_name(&param.pattern)
}

fn binding_identifier_name<'a>(pattern: &'a BindingPattern<'a>) -> Option<&'a str> {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}

fn collect_plugin_keys_from_nuxt2_inject_calls<'a>(
    statements: &'a oxc_allocator::Vec<'a, Statement<'a>>,
    inject_name: &str,
    keys: &mut Vec<String>,
) {
    for statement in statements {
        collect_plugin_keys_from_nuxt2_inject_statement(statement, inject_name, keys);
    }
}

fn collect_plugin_keys_from_nuxt2_inject_statement(
    statement: &Statement<'_>,
    inject_name: &str,
    keys: &mut Vec<String>,
) {
    match statement {
        Statement::ExpressionStatement(expr) => {
            collect_plugin_key_from_nuxt2_inject_expression(&expr.expression, inject_name, keys);
        }
        Statement::BlockStatement(block) => {
            collect_plugin_keys_from_nuxt2_inject_calls(&block.body, inject_name, keys);
        }
        Statement::IfStatement(if_stmt) => {
            collect_plugin_keys_from_nuxt2_inject_statement(&if_stmt.consequent, inject_name, keys);
            if let Some(alternate) = &if_stmt.alternate {
                collect_plugin_keys_from_nuxt2_inject_statement(alternate, inject_name, keys);
            }
        }
        Statement::DoWhileStatement(do_while) => {
            collect_plugin_keys_from_nuxt2_inject_statement(&do_while.body, inject_name, keys);
        }
        Statement::ForInStatement(for_in) => {
            collect_plugin_keys_from_nuxt2_inject_statement(&for_in.body, inject_name, keys);
        }
        Statement::ForOfStatement(for_of) => {
            collect_plugin_keys_from_nuxt2_inject_statement(&for_of.body, inject_name, keys);
        }
        Statement::ForStatement(for_stmt) => {
            collect_plugin_keys_from_nuxt2_inject_statement(&for_stmt.body, inject_name, keys);
        }
        Statement::LabeledStatement(labeled) => {
            collect_plugin_keys_from_nuxt2_inject_statement(&labeled.body, inject_name, keys);
        }
        Statement::SwitchStatement(switch_stmt) => {
            for case in &switch_stmt.cases {
                collect_plugin_keys_from_nuxt2_inject_calls(&case.consequent, inject_name, keys);
            }
        }
        Statement::TryStatement(try_stmt) => {
            collect_plugin_keys_from_nuxt2_inject_calls(&try_stmt.block.body, inject_name, keys);
            if let Some(handler) = &try_stmt.handler {
                collect_plugin_keys_from_nuxt2_inject_calls(&handler.body.body, inject_name, keys);
            }
            if let Some(finalizer) = &try_stmt.finalizer {
                collect_plugin_keys_from_nuxt2_inject_calls(&finalizer.body, inject_name, keys);
            }
        }
        Statement::WhileStatement(while_stmt) => {
            collect_plugin_keys_from_nuxt2_inject_statement(&while_stmt.body, inject_name, keys);
        }
        Statement::WithStatement(with_stmt) => {
            collect_plugin_keys_from_nuxt2_inject_statement(&with_stmt.body, inject_name, keys);
        }
        _ => {}
    }
}

fn collect_plugin_key_from_nuxt2_inject_expression(
    expression: &Expression<'_>,
    inject_name: &str,
    keys: &mut Vec<String>,
) {
    match expression {
        Expression::CallExpression(call) => {
            let Expression::Identifier(callee) = &call.callee else {
                return;
            };
            if callee.name.as_str() != inject_name {
                return;
            }
            if let Some(key) = call.arguments.first().and_then(inject_key_from_argument) {
                keys.push(key);
            }
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            collect_plugin_key_from_nuxt2_inject_expression(
                &parenthesized.expression,
                inject_name,
                keys,
            );
        }
        Expression::TSAsExpression(ts_as) => {
            collect_plugin_key_from_nuxt2_inject_expression(&ts_as.expression, inject_name, keys);
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            collect_plugin_key_from_nuxt2_inject_expression(
                &ts_satisfies.expression,
                inject_name,
                keys,
            );
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            collect_plugin_key_from_nuxt2_inject_expression(
                &ts_non_null.expression,
                inject_name,
                keys,
            );
        }
        _ => {}
    }
}

fn inject_key_from_argument(argument: &Argument<'_>) -> Option<String> {
    match argument {
        Argument::StringLiteral(literal) => Some(literal.value.as_str().into()),
        Argument::TemplateLiteral(template) => {
            template.single_quasi().map(|value| value.as_str().into())
        }
        _ => None,
    }
}

fn collect_plugin_keys_from_function_body<'a>(
    statements: &oxc_allocator::Vec<'a, Statement<'a>>,
    keys: &mut Vec<String>,
) {
    for statement in statements {
        let Statement::ReturnStatement(ret) = statement else {
            continue;
        };
        let Some(argument) = &ret.argument else {
            continue;
        };
        let Some(object) = extract_object_expression(argument) else {
            continue;
        };
        collect_plugin_keys_from_object(object, keys);
    }
}

fn collect_plugin_keys_from_object(object: &ObjectExpression<'_>, keys: &mut Vec<String>) {
    if let Some(provide_object) =
        find_object_property(object, "provide").and_then(extract_object_expression)
    {
        collect_object_keys(provide_object, keys);
    }

    if let Some(setup_expression) = find_object_property(object, "setup") {
        match extract_expression(setup_expression) {
            Some(Expression::ArrowFunctionExpression(arrow)) => {
                collect_plugin_keys_from_function_body(&arrow.body.statements, keys);
            }
            Some(Expression::FunctionExpression(function)) => {
                if let Some(body) = &function.body {
                    collect_plugin_keys_from_function_body(&body.statements, keys);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::extract_plugin_provide_keys_from_source;

    #[test]
    fn extracts_nuxt2_inject_keys_from_callback_plugin() {
        let source = r#"
export default defineNuxtPlugin((_context, register) => {
  register('logger', { info(message) { return message.length } })
  if (true) {
    register(`auth`, {})
  }
})
"#;

        let keys = extract_plugin_provide_keys_from_source(source);
        assert_eq!(keys, vec!["logger", "auth"]);
    }

    #[test]
    fn extracts_nuxt2_inject_keys_from_raw_export_plugin() {
        let source = r#"
export default (_context, inject) => {
  inject('logger', { info(message) { return message.length } })
  if (true) {
    inject(`auth`, {})
  }
}
"#;

        let keys = extract_plugin_provide_keys_from_source(source);
        assert_eq!(keys, vec!["logger", "auth"]);
    }
}
