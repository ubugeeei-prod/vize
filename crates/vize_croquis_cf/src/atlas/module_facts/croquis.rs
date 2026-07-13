use vize_carton::{BindingType, CompactString, FxHashSet};
use vize_croquis::{
    Croquis,
    provide::{InjectPattern, ProvideKey},
    reactivity::ReactiveKind,
    setup_context::SetupContextViolationKind,
};
use vize_module::{
    ModuleDocument, ModuleExpression, ModuleExpressionKind, ModuleOperationKind, ModulePattern,
};

use super::{
    CallView, ModuleAliases, expression_path, expression_text, first_pattern_name, pattern_names,
};

/// Project the script facts needed by cross-file rules from an owned module.
///
/// Raw modules do not have a separate template frontend. Their production
/// Croquis is therefore derived here from neutral operations instead of
/// invoking the legacy OXC analyzer on `ModuleSyntax::source`.
pub(in crate::atlas) fn project_raw_croquis(document: &ModuleDocument) -> Croquis {
    let mut analysis = Croquis::new();
    let mut seen_calls = FxHashSet::default();
    for module in &document.modules {
        let aliases = ModuleAliases::new(module);
        add_import_bindings(&mut analysis, module);
        for operation in &module.operations.operations {
            match &operation.kind {
                ModuleOperationKind::Binding {
                    pattern,
                    initializer,
                    ..
                } => project_binding(
                    &mut analysis,
                    pattern,
                    initializer.as_ref(),
                    operation.span.start,
                    operation.span.end,
                    &aliases,
                    &mut seen_calls,
                ),
                ModuleOperationKind::Assignment { target, .. } => {
                    if let Some(name) = first_pattern_name(target) {
                        analysis.reactivity.record_reassign(
                            name,
                            operation.span.start,
                            operation.span.end,
                        );
                    }
                }
                ModuleOperationKind::Call(call) => project_call(
                    &mut analysis,
                    call,
                    operation.top_level,
                    &aliases,
                    &mut seen_calls,
                ),
                ModuleOperationKind::Return(_) | ModuleOperationKind::Await(_) => {}
            }
        }
    }
    analysis
}

fn add_import_bindings(analysis: &mut Croquis, module: &vize_module::ModuleSyntax) {
    for import in &module.imports {
        for local in &import.locals {
            analysis
                .bindings
                .add(local.as_ref(), BindingType::SetupConst);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn project_binding(
    analysis: &mut Croquis,
    pattern: &ModulePattern,
    initializer: Option<&ModuleExpression>,
    start: u32,
    end: u32,
    aliases: &ModuleAliases,
    seen_calls: &mut FxHashSet<(u32, u32)>,
) {
    let call = initializer.and_then(|value| CallView::from_expression(value, aliases));
    let reactive_kind = call
        .as_ref()
        .and_then(|call| ReactiveKind::from_name(call.callee.as_str()));
    for name in pattern_names(pattern) {
        let binding_type = reactive_kind.map_or(BindingType::SetupConst, |kind| {
            if kind.needs_value_access() {
                BindingType::SetupRef
            } else {
                BindingType::SetupReactiveConst
            }
        });
        analysis.bindings.add(name.as_str(), binding_type);
        analysis.binding_spans.insert(name, (start, end));
    }
    if let (Some(kind), Some(name)) = (reactive_kind, first_pattern_name(pattern)) {
        analysis.reactivity.register(name, kind, start);
    }
    if let Some(initializer) = initializer {
        project_extraction(analysis, pattern, initializer, start, end);
        if CallView::from_expression(initializer, aliases).is_some() {
            project_bound_call(analysis, pattern, initializer, aliases);
            seen_calls.insert((initializer.span.start, initializer.span.end));
        }
    }
}

fn project_extraction(
    analysis: &mut Croquis,
    pattern: &ModulePattern,
    initializer: &ModuleExpression,
    start: u32,
    end: u32,
) {
    let Some(path) = expression_path(initializer) else {
        return;
    };
    let Some(source) = path.first().cloned() else {
        return;
    };
    match pattern {
        ModulePattern::Object(_) | ModulePattern::Array(_) => analysis
            .reactivity
            .record_destructure(source, pattern_names(pattern), start, end),
        ModulePattern::Identifier(target) if path.len() > 1 && path[1] == "value" => analysis
            .reactivity
            .record_ref_value_extract(source, CompactString::new(target.as_ref()), start, end),
        ModulePattern::Identifier(target) if path.len() > 1 => {
            analysis.reactivity.record_property_extract(
                source,
                path[1].clone(),
                CompactString::new(target.as_ref()),
                start,
                end,
            )
        }
        _ => {}
    }
}

fn project_bound_call(
    analysis: &mut Croquis,
    pattern: &ModulePattern,
    expression: &ModuleExpression,
    aliases: &ModuleAliases,
) {
    let Some(call) = CallView::from_expression(expression, aliases) else {
        return;
    };
    if call.callee == "provide" {
        let key = call.arguments.first().map_or_else(
            || ProvideKey::String(CompactString::new("<unknown>")),
            provide_key,
        );
        analysis.provide_inject.add_provide(
            key,
            call.arguments
                .get(1)
                .map_or_else(|| CompactString::new("undefined"), expression_text),
            None,
            None,
            expression.span.start,
            expression.span.end,
        );
    }
    if call.callee == "inject" {
        let key = call.arguments.first().map_or_else(
            || ProvideKey::String(CompactString::new("<unknown>")),
            provide_key,
        );
        analysis.provide_inject.add_inject(
            key,
            first_pattern_name(pattern).unwrap_or_else(|| CompactString::new("<destructure>")),
            call.arguments.get(1).map(expression_text),
            None,
            inject_pattern(pattern),
            None,
            expression.span.start,
            expression.span.end,
        );
    }
    if call.raw_callee.starts_with("use") {
        let source = aliases
            .source_for_local(call.raw_callee.as_str())
            .map_or_else(|| CompactString::new(""), CompactString::new);
        analysis.provide_inject.add_composable(
            call.raw_callee,
            source,
            matches!(pattern, ModulePattern::Identifier(_))
                .then(|| first_pattern_name(pattern))
                .flatten(),
            false,
            false,
            false,
            expression.span.start,
            expression.span.end,
        );
    }
}

fn project_call(
    analysis: &mut Croquis,
    expression: &ModuleExpression,
    top_level: bool,
    aliases: &ModuleAliases,
    seen_calls: &mut FxHashSet<(u32, u32)>,
) {
    let Some(call) = CallView::from_expression(expression, aliases) else {
        return;
    };
    let first_visit = seen_calls.insert((expression.span.start, expression.span.end));
    if first_visit && call.callee == "provide" {
        let key = call.arguments.first().map_or_else(
            || ProvideKey::String(CompactString::new("<unknown>")),
            provide_key,
        );
        analysis.provide_inject.add_provide(
            key,
            call.arguments
                .get(1)
                .map_or_else(|| CompactString::new("undefined"), expression_text),
            None,
            None,
            expression.span.start,
            expression.span.end,
        );
    }
    if first_visit && call.callee == "inject" {
        let key = call.arguments.first().map_or_else(
            || ProvideKey::String(CompactString::new("<unknown>")),
            provide_key,
        );
        analysis.provide_inject.add_inject(
            key,
            CompactString::new("<inject>"),
            call.arguments.get(1).map(expression_text),
            None,
            InjectPattern::Simple,
            None,
            expression.span.start,
            expression.span.end,
        );
    }
    if top_level && let Some(kind) = SetupContextViolationKind::from_api_name(call.callee.as_str())
    {
        analysis.setup_context.record_violation(
            kind,
            call.callee,
            expression.span.start,
            expression.span.end,
        );
    }
}

fn provide_key(expression: &ModuleExpression) -> ProvideKey {
    match &expression.kind {
        ModuleExpressionKind::Literal {
            kind: vize_module::ModuleLiteralKind::String,
            value: Some(value),
            ..
        } => ProvideKey::String(CompactString::new(value.as_ref())),
        _ => ProvideKey::Symbol(expression_text(expression)),
    }
}

fn inject_pattern(pattern: &ModulePattern) -> InjectPattern {
    match pattern {
        ModulePattern::Object(_) => InjectPattern::ObjectDestructure(pattern_names(pattern)),
        ModulePattern::Array(_) => InjectPattern::ArrayDestructure(pattern_names(pattern)),
        _ => InjectPattern::Simple,
    }
}
