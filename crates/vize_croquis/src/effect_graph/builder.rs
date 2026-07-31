//! Parser-driven effect graph construction.

mod context;
mod deps;
mod sfc;

pub use sfc::{EffectGraphScript, build_effect_graph_from_sfc_scripts};

use context::{EffectBuildContext, context_for_program, variable_declaration_from_statement};
use deps::collect_argument_deps;
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPattern, CallExpression, Expression, Program, Statement, VariableDeclaration,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{CompactString, FxHashMap, FxHashSet, cstr};

use super::EffectGraph;

pub fn build_effect_graph_from_script_setup(source: &str) -> EffectGraph {
    build_effect_graph_from_source(source, "script.ts")
}

pub fn build_effect_graph_from_script(source: &str) -> EffectGraph {
    build_effect_graph_from_source(source, "script.ts")
}

fn build_effect_graph_from_source(source: &str, path: &str) -> EffectGraph {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap_or_default();
    let ret = Parser::new(&allocator, source, source_type).parse();
    if ret.panicked || !ret.diagnostics.is_empty() {
        return EffectGraph::default();
    }
    build_effect_graph_from_program(&ret.program)
}

pub(crate) fn build_effect_graph_from_program(program: &Program<'_>) -> EffectGraph {
    let context = context_for_program(program, None, "");
    let mut graph = EffectGraph::default();
    collect_program_effect_edges(program, &context, &mut graph);
    graph
}

fn collect_program_effect_edges(
    program: &Program<'_>,
    context: &EffectBuildContext,
    graph: &mut EffectGraph,
) {
    for statement in &program.body {
        collect_effect_edges(statement, context, graph);
    }
}

fn collect_vue_api_aliases(
    statement: &Statement<'_>,
    api_aliases: &mut FxHashMap<CompactString, CompactString>,
) {
    let Statement::ImportDeclaration(import) = statement else {
        return;
    };
    if import.source.value.as_str() != "vue" || import.import_kind.is_type() {
        return;
    }
    let Some(specifiers) = &import.specifiers else {
        return;
    };

    for specifier in specifiers.iter() {
        if let oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier
            && !specifier.import_kind.is_type()
        {
            let imported = specifier.imported.name().as_str();
            let local = specifier.local.name.as_str();
            if is_reactive_api(imported) || is_effect_api(imported) {
                api_aliases.insert(CompactString::new(local), CompactString::new(imported));
            }
        }
    }
}

fn collect_reactive_sources(
    statement: &Statement<'_>,
    api_aliases: &FxHashMap<CompactString, CompactString>,
    blocked_api_names: &FxHashSet<CompactString>,
    reactive_sources: &mut FxHashSet<CompactString>,
) {
    let Some(declaration) = variable_declaration_from_statement(statement) else {
        return;
    };

    for declarator in declaration.declarations.iter() {
        let BindingPattern::BindingIdentifier(identifier) = &declarator.id else {
            continue;
        };
        let Some(call) = declarator.init.as_ref().and_then(call_from_expression) else {
            continue;
        };
        if call_api_name(call, api_aliases, blocked_api_names)
            .is_some_and(|name| is_reactive_api(name.as_str()))
        {
            reactive_sources.insert(CompactString::new(identifier.name.as_str()));
        }
    }
}

fn collect_effect_edges(
    statement: &Statement<'_>,
    context: &EffectBuildContext,
    graph: &mut EffectGraph,
) {
    if let Some(declaration) = variable_declaration_from_statement(statement) {
        collect_variable_effect_edges(declaration, context, graph);
        return;
    }
    if let Statement::ExpressionStatement(statement) = statement {
        collect_call_effect_edges(&statement.expression, context, graph);
    }
}

fn collect_variable_effect_edges(
    declaration: &VariableDeclaration<'_>,
    context: &EffectBuildContext,
    graph: &mut EffectGraph,
) {
    for declarator in declaration.declarations.iter() {
        let Some(init) = &declarator.init else {
            continue;
        };
        let Some(call) = call_from_expression(init) else {
            collect_call_effect_edges(init, context, graph);
            continue;
        };

        let Some(api_name) = call_api_name(call, &context.api_aliases, &context.blocked_api_names)
        else {
            collect_call_effect_edges(init, context, graph);
            continue;
        };

        if api_name == "computed" {
            if let BindingPattern::BindingIdentifier(identifier) = &declarator.id {
                let source_name = CompactString::new(identifier.name.as_str());
                let from = context
                    .reactive_node_ids
                    .get(&source_name)
                    .cloned()
                    .unwrap_or_else(|| scoped_node_name(&context.prefix, &source_name));
                add_edges_from_first_arg(&from, call, context, graph);
            }
        } else if is_effect_api(api_name.as_str()) {
            let from = effect_node_name(&context.prefix, api_name.as_str(), call.span.start);
            add_edges_from_first_arg(&from, call, context, graph);
        }
    }
}

fn collect_call_effect_edges(
    expression: &Expression<'_>,
    context: &EffectBuildContext,
    graph: &mut EffectGraph,
) {
    let Some(call) = call_from_expression(expression) else {
        return;
    };
    let Some(api_name) = call_api_name(call, &context.api_aliases, &context.blocked_api_names)
    else {
        return;
    };
    if is_effect_api(api_name.as_str()) {
        let from = effect_node_name(&context.prefix, api_name.as_str(), call.span.start);
        add_edges_from_first_arg(&from, call, context, graph);
    }
}

fn add_edges_from_first_arg(
    from: &CompactString,
    call: &CallExpression<'_>,
    context: &EffectBuildContext,
    graph: &mut EffectGraph,
) {
    let Some(arg) = call.arguments.first() else {
        return;
    };

    let mut deps = std::collections::BTreeSet::new();
    let excluded = FxHashSet::default();
    collect_argument_deps(arg, &context.reactive_sources, &excluded, &mut deps);
    for dep in deps {
        if let Some(target) = context.reactive_node_ids.get(&dep) {
            graph.add_edge(from.clone(), target.clone());
        }
    }
}

fn call_from_expression<'a>(expression: &'a Expression<'a>) -> Option<&'a CallExpression<'a>> {
    match expression {
        Expression::CallExpression(call) => Some(call),
        Expression::ParenthesizedExpression(expression) => {
            call_from_expression(&expression.expression)
        }
        Expression::TSAsExpression(expression) => call_from_expression(&expression.expression),
        Expression::TSSatisfiesExpression(expression) => {
            call_from_expression(&expression.expression)
        }
        Expression::TSNonNullExpression(expression) => call_from_expression(&expression.expression),
        _ => None,
    }
}

fn call_api_name(
    call: &CallExpression<'_>,
    api_aliases: &FxHashMap<CompactString, CompactString>,
    blocked_api_names: &FxHashSet<CompactString>,
) -> Option<CompactString> {
    let Expression::Identifier(identifier) = &call.callee else {
        return None;
    };
    let raw = identifier.name.as_str();
    if blocked_api_names.contains(raw) {
        return None;
    }
    Some(
        api_aliases
            .get(raw)
            .cloned()
            .unwrap_or_else(|| CompactString::new(raw)),
    )
}

fn scoped_node_name(prefix: &str, name: &str) -> CompactString {
    if prefix.is_empty() {
        CompactString::new(name)
    } else {
        cstr!("{prefix}:{name}")
    }
}

fn effect_node_name(prefix: &str, api_name: &str, start: u32) -> CompactString {
    scoped_node_name(prefix, &cstr!("{api_name}@{start}"))
}

fn is_reactive_api(name: &str) -> bool {
    matches!(
        name,
        "ref"
            | "shallowRef"
            | "reactive"
            | "shallowReactive"
            | "computed"
            | "readonly"
            | "shallowReadonly"
            | "toRef"
            | "toRefs"
            | "customRef"
            | "useTemplateRef"
    )
}

fn is_effect_api(name: &str) -> bool {
    matches!(
        name,
        "watch" | "watchEffect" | "watchPostEffect" | "watchSyncEffect"
    )
}
