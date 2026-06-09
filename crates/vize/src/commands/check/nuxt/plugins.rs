//! Detection of `defineNuxtPlugin` provide keys for injected helpers.

use std::path::Path;

use ignore::WalkBuilder;
use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, Expression, ObjectExpression, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{FxHashSet, String, cstr};

use super::parsing::{
    collect_object_keys, extract_call_expression_from_export, extract_expression,
    extract_object_expression, find_object_property,
};
use super::stubs::{push_stub, tracked_read_to_string};

pub(super) fn collect_plugin_injection_stubs(
    cwd: &Path,
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
) {
    let plugin_dirs = [cwd.join("app/plugins"), cwd.join("plugins")];
    let mut plugin_keys = Vec::new();

    for dir in plugin_dirs {
        if !dir.exists() {
            continue;
        }

        let walker = WalkBuilder::new(dir)
            .hidden(false)
            .standard_filters(false)
            .build();

        for entry in walker.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else {
                continue;
            };
            if ext != "ts" && ext != "js" && ext != "mts" && ext != "cts" {
                continue;
            }

            if let Ok(source) = tracked_read_to_string(path) {
                plugin_keys.extend(extract_plugin_provide_keys_from_source(&source));
            }
        }
    }

    plugin_keys.sort();
    plugin_keys.dedup();

    if plugin_keys.is_empty() {
        return;
    }

    stubs.push(
        "type __VizeNuxtInjection<K extends PropertyKey> = import('#app').NuxtApp extends Record<K, infer T> ? T : any;"
            .into(),
    );

    for key in plugin_keys {
        let injected_name = if key.starts_with('$') {
            key
        } else {
            cstr!("${key}")
        };
        push_stub(
            stubs,
            seen_names,
            cstr!("declare const {injected_name}: __VizeNuxtInjection<'{injected_name}'>;"),
        );
    }
}

pub(super) fn extract_plugin_provide_keys_from_source(source: &str) -> Vec<String> {
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
        let Some(call) = extract_call_expression_from_export(&export.declaration) else {
            continue;
        };
        let Expression::Identifier(callee) = &call.callee else {
            continue;
        };
        if callee.name.as_str() != "defineNuxtPlugin" {
            continue;
        }
        let Some(first_arg) = call.arguments.first() else {
            continue;
        };
        collect_plugin_keys_from_argument(first_arg, &mut keys);
    }

    keys
}

fn collect_plugin_keys_from_argument(arg: &Argument<'_>, keys: &mut Vec<String>) {
    match arg {
        Argument::ObjectExpression(object) => collect_plugin_keys_from_object(object, keys),
        Argument::ArrowFunctionExpression(arrow) => {
            collect_plugin_keys_from_function_body(&arrow.body.statements, keys)
        }
        Argument::FunctionExpression(function) => {
            if let Some(body) = &function.body {
                collect_plugin_keys_from_function_body(&body.statements, keys);
            }
        }
        _ => {}
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
