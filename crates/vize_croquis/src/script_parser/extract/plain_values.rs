mod names;
pub(in crate::script_parser) use names::snapshot_in_reexecuted_scope;
use names::{composable_call_name, is_intentional_discard, root_identifier, spread_writes_back};

mod mutations;
pub use mutations::{
    check_reactive_plain_assignment_mutation, check_reactive_plain_call_mutation,
    check_reactive_plain_update_mutation,
};
mod reassignment;
mod records;
mod sources;

use oxc_ast::ast::{AssignmentTarget, CallExpression, Expression};
use oxc_span::GetSpan;

use vize_carton::{CompactString, FxHashMap};

use super::super::{ReactiveGetterContext, ReactiveValueOrigin, ScriptParseResult};
use crate::scope::ScopeKind;

pub(super) struct ReactivePlainValue {
    pub(super) source_name: CompactString,
    pub(super) argument_name: CompactString,
    pub(super) getter_name: CompactString,
    pub(super) start: u32,
    pub(super) end: u32,
}

/// Plain snapshots are allowed to cross ordinary call boundaries as read-only
/// values. For composables (`useXxx`), passing a plain snapshot usually cuts the
/// reactive graph at the API boundary, so report it and suggest a ref/computed
/// connection instead.
#[inline]
pub fn detect_call_argument_reactivity_loss(
    result: &mut ScriptParseResult,
    call: &CallExpression<'_>,
    source: &str,
) {
    if call.arguments.is_empty()
        || (result.reactivity.count() == 0
            && result.reactive_value_origins.is_empty()
            && result.reactive_getter_contexts.is_empty())
    {
        return;
    }

    let Some(callee_name) = composable_call_name(result, call) else {
        return;
    };

    for arg in call.arguments.iter() {
        match arg {
            oxc_ast::ast::Argument::SpreadElement(spread) => {
                records::record_reactive_plain_values_in_composable_arg(
                    result,
                    &spread.argument,
                    &callee_name,
                    source,
                );
            }
            _ => {
                if let Some(expr) = arg.as_expression() {
                    if records::getter_source_from_function(result, expr, source).is_some() {
                        continue;
                    }
                    records::record_reactive_plain_values_in_composable_arg(
                        result,
                        expr,
                        &callee_name,
                        source,
                    );
                }
            }
        }
    }
}

/// Track call results whose arguments are getters of reactive snapshots.
#[inline]
pub fn record_getter_context_from_call(
    result: &mut ScriptParseResult,
    target_name: &str,
    call: &CallExpression<'_>,
    source: &str,
) {
    if call.arguments.is_empty()
        || (result.reactivity.count() == 0
            && result.reactive_value_origins.is_empty()
            && result.reactive_getter_contexts.is_empty())
    {
        return;
    }

    let mut getters = None;

    for arg in call.arguments.iter() {
        let Some(expr) = arg.as_expression() else {
            continue;
        };
        let Some(value) = records::getter_source_from_function(result, expr, source) else {
            continue;
        };
        getters
            .get_or_insert_with(FxHashMap::default)
            .insert(value.getter_name, value.source_name);
    }

    let Some(getters) = getters else {
        return;
    };

    result.reactive_getter_contexts.insert(
        CompactString::new(target_name),
        ReactiveGetterContext {
            callee_name: super::common::call_label(result, call, source),
            getters,
        },
    );
}

/// Check `const x = ctx.count()` where `ctx` was produced from getter arguments.
#[inline]
pub fn check_getter_call_extraction(
    result: &mut ScriptParseResult,
    id: &oxc_ast::ast::BindingPattern<'_>,
    init: &Expression<'_>,
) {
    if result.reactive_getter_contexts.is_empty() {
        return;
    }

    let target_name = match id {
        oxc_ast::ast::BindingPattern::BindingIdentifier(id) => id.name.as_str(),
        _ => return,
    };

    let Some((context_name, getter_name, source_name, callee_name)) =
        sources::getter_call_source(result, init)
    else {
        return;
    };

    use crate::reactivity::{ReactivityLoss, ReactivityLossKind};
    result.reactivity.add_loss(ReactivityLoss {
        kind: ReactivityLossKind::GetterCallExtract {
            context_name: context_name.clone(),
            getter_name: getter_name.clone(),
            target_name: CompactString::new(target_name),
            callee_name,
            source_name: source_name.clone(),
        },
        start: init.span().start,
        end: init.span().end,
    });
    result.record_reactive_origin(
        CompactString::new(target_name),
        ReactiveValueOrigin::GetterCall {
            context_name,
            getter_name,
            source_name,
        },
    );
}

/// Check `const alias = count` where `count` is already a plain reactive snapshot.
#[inline]
pub fn check_reactive_plain_alias_extraction(
    result: &mut ScriptParseResult,
    id: &oxc_ast::ast::BindingPattern<'_>,
    init: &Expression<'_>,
) {
    if result.reactive_value_origins.is_empty() {
        return;
    }

    let target_name = match id {
        oxc_ast::ast::BindingPattern::BindingIdentifier(id) => id.name.as_str(),
        _ => return,
    };

    let Some(value) = sources::reactive_plain_identifier_value_from_expr(result, init) else {
        return;
    };
    if value.argument_name.as_str() == target_name {
        return;
    }

    let live = result.keeps_live_ref_origin(value.argument_name.as_str());
    result.reactivity.record_plain_value_alias(
        value.source_name.clone(),
        value.argument_name,
        CompactString::new(target_name),
        value.start,
        value.end,
    );
    result.record_reactive_origin(
        CompactString::new(target_name),
        ReactiveValueOrigin::PlainAlias {
            source_name: value.source_name,
        },
    );
    result.record_live_ref_origin(target_name, live);
}

/// Check `alias = count` where `count` is already a plain reactive snapshot.
#[inline]
pub fn check_reactive_plain_assignment_alias(
    result: &mut ScriptParseResult,
    target_name: &str,
    init: &Expression<'_>,
) {
    if result.reactive_value_origins.is_empty() {
        return;
    }
    if result.reactivity.is_reactive(target_name) {
        return;
    }

    let Some(value) = sources::reactive_plain_identifier_value_from_expr(result, init) else {
        reassignment::clear_replaced_origin(result, target_name);
        return;
    };
    if value.argument_name.as_str() == target_name {
        return;
    }

    let live = result.keeps_live_ref_origin(value.argument_name.as_str());
    result.reactivity.record_plain_value_alias(
        value.source_name.clone(),
        value.argument_name,
        CompactString::new(target_name),
        value.start,
        value.end,
    );
    result.record_reactive_origin(
        CompactString::new(target_name),
        ReactiveValueOrigin::PlainAlias {
            source_name: value.source_name,
        },
    );
    result.record_live_ref_origin(target_name, live);
}

#[inline]
pub fn check_reactive_spread_expression(
    result: &mut ScriptParseResult,
    expr: &Expression<'_>,
    source: &str,
    start: u32,
    end: u32,
) {
    if result.reactivity.count() == 0 && result.reactive_value_origins.is_empty() {
        return;
    }

    let Some(source_name) = sources::reactive_expression_label_for_spread(result, expr, source)
    else {
        return;
    };
    if snapshot_in_reexecuted_scope(result) || spread_writes_back(result, source_name.as_str()) {
        return;
    }
    result
        .reactivity
        .record_spread_expression(source_name, start, end);
}

/// Root of `open.value = ...` / `state[key] = ...`. Identifier assignment is
/// a replacement of the binding, not an immutable update.
pub(in crate::script_parser) fn member_assignment_root(
    target: &AssignmentTarget<'_>,
) -> Option<CompactString> {
    let object = match target {
        AssignmentTarget::StaticMemberExpression(member) => &member.object,
        AssignmentTarget::ComputedMemberExpression(member) => &member.object,
        _ => return None,
    };
    root_identifier(object)
}

#[inline]
pub(in crate::script_parser) fn reactive_destructure_source(
    result: &ScriptParseResult,
    init: &Expression<'_>,
    source: &str,
) -> Option<(CompactString, bool, u32, u32)> {
    if result.reactivity.count() == 0 && result.reactive_value_origins.is_empty() {
        return None;
    }

    if let Some((source_name, is_ref_value)) =
        sources::ref_value_destructure_source(result, init, source)
    {
        return Some((
            source_name,
            is_ref_value,
            init.span().start,
            init.span().end,
        ));
    }
    sources::reactive_member_destructure_source(result, init, source)
        .map(|source_name| (source_name, false, init.span().start, init.span().end))
}

pub fn check_ref_value_extraction(
    result: &mut ScriptParseResult,
    id: &oxc_ast::ast::BindingPattern<'_>,
    type_annotation: Option<&oxc_ast::ast::TSTypeAnnotation<'_>>,
    init: &Expression<'_>,
) {
    if result.reactivity.count() == 0 {
        return;
    }

    // Only check simple identifier bindings
    let target_name = match id {
        oxc_ast::ast::BindingPattern::BindingIdentifier(id) => id.name.as_str(),
        _ => return,
    };
    if is_intentional_discard(id, type_annotation) {
        return;
    }

    // Check for ref.value pattern: someRef.value
    if let Expression::StaticMemberExpression(member) = init
        && member.property.name.as_str() == "value"
        && let Expression::Identifier(obj_id) = &member.object
    {
        let ref_name = CompactString::new(obj_id.name.as_str());
        if result.reactivity.needs_value_access(ref_name.as_str()) {
            if !snapshot_in_reexecuted_scope(result) {
                use crate::reactivity::{ReactivityLoss, ReactivityLossKind};
                result.reactivity.add_loss(ReactivityLoss {
                    kind: ReactivityLossKind::RefValueExtract {
                        source_name: ref_name.clone(),
                        target_name: CompactString::new(target_name),
                    },
                    start: member.span.start,
                    end: member.span.end,
                });
            }
            result.record_reactive_origin(
                CompactString::new(target_name),
                ReactiveValueOrigin::RefValue {
                    source_name: ref_name,
                },
            );
        }
    }
}

/// Check for reactive property extraction to a plain variable.
/// e.g., `const x = state.x` or `const x = props.x`
#[inline]
pub fn check_reactive_property_extraction(
    result: &mut ScriptParseResult,
    id: &oxc_ast::ast::BindingPattern<'_>,
    type_annotation: Option<&oxc_ast::ast::TSTypeAnnotation<'_>>,
    init: &Expression<'_>,
) {
    let target_name = match id {
        oxc_ast::ast::BindingPattern::BindingIdentifier(id) => id.name.as_str(),
        _ => return,
    };
    if is_intentional_discard(id, type_annotation) {
        return;
    }

    if let Some((source_name, prop_name)) = sources::ref_value_property_extract(result, init) {
        if !snapshot_in_reexecuted_scope(result) {
            use crate::reactivity::{ReactivityLoss, ReactivityLossKind};
            result.reactivity.add_loss(ReactivityLoss {
                kind: ReactivityLossKind::ReactivePropertyExtract {
                    source_name: source_name.clone(),
                    prop_name: prop_name.clone(),
                    target_name: CompactString::new(target_name),
                },
                start: init.span().start,
                end: init.span().end,
            });
        }
        result.record_reactive_origin(
            CompactString::new(target_name),
            ReactiveValueOrigin::ReactiveProperty {
                source_name,
                prop_name,
            },
        );
        return;
    }

    let Some((source_name, prop_name)) = super::common::extract_member_chain_root(init) else {
        return;
    };

    let is_reactive_property = result
        .reactivity
        .lookup(source_name.as_str())
        .is_some_and(|source| !source.kind.needs_value_access());
    if !is_reactive_property {
        return;
    }

    if !snapshot_in_reexecuted_scope(result) {
        result.reactivity.record_property_extract(
            source_name.clone(),
            prop_name.clone(),
            CompactString::new(target_name),
            init.span().start,
            init.span().end,
        );
    }
    result.record_reactive_origin(
        CompactString::new(target_name),
        ReactiveValueOrigin::ReactiveProperty {
            source_name,
            prop_name,
        },
    );
}
