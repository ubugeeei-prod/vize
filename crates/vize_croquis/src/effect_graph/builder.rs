//! Parser-driven effect graph construction.

mod deps;

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
    if ret.panicked {
        return EffectGraph::default();
    }
    build_effect_graph_from_program(&ret.program)
}

pub(crate) fn build_effect_graph_from_program(program: &Program<'_>) -> EffectGraph {
    let mut api_aliases = FxHashMap::default();
    let mut reactive_sources = FxHashSet::default();

    for statement in program.body.iter() {
        collect_vue_api_aliases(statement, &mut api_aliases);
    }
    for statement in program.body.iter() {
        collect_reactive_sources(statement, &api_aliases, &mut reactive_sources);
    }

    let mut graph = EffectGraph::default();
    for statement in program.body.iter() {
        collect_effect_edges(statement, &api_aliases, &reactive_sources, &mut graph);
    }
    graph
}

fn collect_vue_api_aliases(
    statement: &Statement<'_>,
    api_aliases: &mut FxHashMap<CompactString, CompactString>,
) {
    let Statement::ImportDeclaration(import) = statement else {
        return;
    };
    if import.source.value.as_str() != "vue" {
        return;
    }
    let Some(specifiers) = &import.specifiers else {
        return;
    };

    for specifier in specifiers.iter() {
        if let oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier {
            let imported = specifier.imported.name().as_str();
            let local = specifier.local.name.as_str();
            if (is_reactive_api(imported) || is_effect_api(imported)) && imported != local {
                api_aliases.insert(CompactString::new(local), CompactString::new(imported));
            }
        }
    }
}

fn collect_reactive_sources(
    statement: &Statement<'_>,
    api_aliases: &FxHashMap<CompactString, CompactString>,
    reactive_sources: &mut FxHashSet<CompactString>,
) {
    let Statement::VariableDeclaration(declaration) = statement else {
        return;
    };

    for declarator in declaration.declarations.iter() {
        let BindingPattern::BindingIdentifier(identifier) = &declarator.id else {
            continue;
        };
        let Some(call) = declarator.init.as_ref().and_then(call_from_expression) else {
            continue;
        };
        if call_api_name(call, api_aliases).is_some_and(|name| is_reactive_api(name.as_str())) {
            reactive_sources.insert(CompactString::new(identifier.name.as_str()));
        }
    }
}

fn collect_effect_edges(
    statement: &Statement<'_>,
    api_aliases: &FxHashMap<CompactString, CompactString>,
    reactive_sources: &FxHashSet<CompactString>,
    graph: &mut EffectGraph,
) {
    match statement {
        Statement::VariableDeclaration(declaration) => {
            collect_variable_effect_edges(declaration, api_aliases, reactive_sources, graph);
        }
        Statement::ExpressionStatement(statement) => {
            collect_call_effect_edges(&statement.expression, api_aliases, reactive_sources, graph);
        }
        _ => {}
    }
}

fn collect_variable_effect_edges(
    declaration: &VariableDeclaration<'_>,
    api_aliases: &FxHashMap<CompactString, CompactString>,
    reactive_sources: &FxHashSet<CompactString>,
    graph: &mut EffectGraph,
) {
    for declarator in declaration.declarations.iter() {
        let Some(init) = &declarator.init else {
            continue;
        };
        let Some(call) = call_from_expression(init) else {
            collect_call_effect_edges(init, api_aliases, reactive_sources, graph);
            continue;
        };

        let Some(api_name) = call_api_name(call, api_aliases) else {
            collect_call_effect_edges(init, api_aliases, reactive_sources, graph);
            continue;
        };

        if api_name == "computed" {
            if let BindingPattern::BindingIdentifier(identifier) = &declarator.id {
                let from = CompactString::new(identifier.name.as_str());
                add_edges_from_first_arg(&from, call, reactive_sources, graph);
            }
        } else if is_effect_api(api_name.as_str()) {
            let from = effect_node_name(api_name.as_str(), call.span.start);
            add_edges_from_first_arg(&from, call, reactive_sources, graph);
        }
    }
}

fn collect_call_effect_edges(
    expression: &Expression<'_>,
    api_aliases: &FxHashMap<CompactString, CompactString>,
    reactive_sources: &FxHashSet<CompactString>,
    graph: &mut EffectGraph,
) {
    let Some(call) = call_from_expression(expression) else {
        return;
    };
    let Some(api_name) = call_api_name(call, api_aliases) else {
        return;
    };
    if is_effect_api(api_name.as_str()) {
        let from = effect_node_name(api_name.as_str(), call.span.start);
        add_edges_from_first_arg(&from, call, reactive_sources, graph);
    }
}

fn add_edges_from_first_arg(
    from: &CompactString,
    call: &CallExpression<'_>,
    reactive_sources: &FxHashSet<CompactString>,
    graph: &mut EffectGraph,
) {
    let Some(arg) = call.arguments.first() else {
        return;
    };

    let mut deps = std::collections::BTreeSet::new();
    let excluded = FxHashSet::default();
    collect_argument_deps(arg, reactive_sources, &excluded, &mut deps);
    for dep in deps {
        if dep != *from {
            graph.add_edge(from.clone(), dep);
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
) -> Option<CompactString> {
    let Expression::Identifier(identifier) = &call.callee else {
        return None;
    };
    let raw = identifier.name.as_str();
    Some(
        api_aliases
            .get(raw)
            .cloned()
            .unwrap_or_else(|| CompactString::new(raw)),
    )
}

fn effect_node_name(api_name: &str, start: u32) -> CompactString {
    cstr!("{api_name}@{start}")
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
