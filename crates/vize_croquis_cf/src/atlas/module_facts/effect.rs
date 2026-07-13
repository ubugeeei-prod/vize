use vize_carton::{CompactString, FxHashMap, FxHashSet, cstr};
use vize_croquis::{EffectGraph, EffectGraphSummary};
use vize_module::{
    ModuleDocument, ModuleExpression, ModuleExpressionKind, ModuleOperationKind, ModuleSyntax,
};

use super::{CallView, ModuleAliases, expression_path, first_pattern_name};

pub(in crate::atlas) fn module_effect_summary(document: &ModuleDocument) -> EffectGraphSummary {
    let mut graph = EffectGraph::default();
    let mut inherited = FxHashMap::default();
    for module in &document.modules {
        let prefix = module_prefix(module);
        let aliases = ModuleAliases::new(module);
        let mut reactive = inherited.clone();
        collect_reactive_bindings(module, &aliases, prefix, &mut reactive);
        collect_effects(module, &aliases, prefix, &reactive, &mut graph);
        inherited.extend(reactive);
    }
    graph.summary()
}

fn collect_reactive_bindings(
    module: &ModuleSyntax,
    aliases: &ModuleAliases,
    prefix: &str,
    reactive: &mut FxHashMap<CompactString, CompactString>,
) {
    for operation in &module.operations.operations {
        let ModuleOperationKind::Binding {
            pattern,
            initializer: Some(initializer),
            ..
        } = &operation.kind
        else {
            continue;
        };
        let Some(call) = CallView::from_expression(initializer, aliases) else {
            continue;
        };
        if is_reactive_api(call.callee.as_str())
            && let Some(name) = first_pattern_name(pattern)
        {
            reactive.insert(name.clone(), scoped(prefix, name.as_str()));
        }
    }
}

fn collect_effects(
    module: &ModuleSyntax,
    aliases: &ModuleAliases,
    prefix: &str,
    reactive: &FxHashMap<CompactString, CompactString>,
    graph: &mut EffectGraph,
) {
    let mut seen = FxHashSet::default();
    for operation in &module.operations.operations {
        let (from, call) = match &operation.kind {
            ModuleOperationKind::Binding {
                pattern,
                initializer: Some(initializer),
                ..
            } => {
                let Some(call) = CallView::from_expression(initializer, aliases) else {
                    continue;
                };
                if call.callee != "computed" {
                    continue;
                }
                let Some(name) = first_pattern_name(pattern) else {
                    continue;
                };
                (scoped(prefix, name.as_str()), call)
            }
            ModuleOperationKind::Call(expression) => {
                let Some(call) = CallView::from_expression(expression, aliases) else {
                    continue;
                };
                if !is_effect_api(call.callee.as_str()) {
                    continue;
                }
                (
                    scoped(prefix, &cstr!("{}@{}", call.callee, expression.span.start)),
                    call,
                )
            }
            _ => continue,
        };
        let span = operation.span;
        if !seen.insert((span.start, span.end)) {
            continue;
        }
        let Some(argument) = call.arguments.first() else {
            continue;
        };
        for dependency in argument_dependencies(module, argument) {
            if let Some(target) = reactive.get(dependency.as_str()) {
                graph.add_edge(from.clone(), target.clone());
            }
        }
    }
}

fn argument_dependencies(
    module: &ModuleSyntax,
    argument: &ModuleExpression,
) -> FxHashSet<CompactString> {
    let mut dependencies = FxHashSet::default();
    if let Some(path) = expression_path(argument)
        && let Some(name) = path.first()
    {
        dependencies.insert(name.clone());
    }
    if matches!(argument.kind, ModuleExpressionKind::Function { .. })
        && let Some(function) = module
            .operations
            .functions
            .iter()
            .find(|function| function.span == argument.span)
    {
        dependencies.extend(
            function
                .references
                .iter()
                .filter(|name| {
                    !function
                        .local_bindings
                        .iter()
                        .any(|local| local.as_ref() == name.as_ref())
                })
                .map(|name| CompactString::new(name.as_ref())),
        );
    }
    dependencies
}

fn module_prefix(module: &ModuleSyntax) -> &str {
    if module.name.ends_with("#script-setup") {
        "setup"
    } else if module.name.ends_with("#script") {
        "script"
    } else {
        ""
    }
}

fn scoped(prefix: &str, name: &str) -> CompactString {
    if prefix.is_empty() {
        CompactString::new(name)
    } else {
        cstr!("{prefix}:{name}")
    }
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
