mod records;
mod sinks;
mod sources;

use oxc_ast::ast::{Argument, CallExpression, Expression};
use oxc_span::GetSpan;

use vize_carton::{CompactString, FxHashMap};

use super::super::{ReactiveGetterContext, ReactiveValueOrigin, ScriptParseResult};

pub(super) struct ReactivePlainValue {
    pub(super) source_name: CompactString,
    pub(super) argument_name: CompactString,
    pub(super) getter_name: CompactString,
    pub(super) start: u32,
    pub(super) end: u32,
}

/// Record reactivity loss when a plain reactive snapshot crosses a call boundary.
#[inline]
pub fn detect_call_argument_reactivity_loss(
    result: &mut ScriptParseResult,
    call: &CallExpression<'_>,
    source: &str,
) {
    if sinks::is_reactivity_loss_value_sink_call(result, call) {
        return;
    }

    let callee_name = super::common::call_label(result, call, source);

    for arg in call.arguments.iter() {
        match arg {
            Argument::SpreadElement(spread) => {
                records::record_reactive_plain_values_in_call_arg(
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
                    records::record_reactive_plain_values_in_call_arg(
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
    let mut getters = FxHashMap::default();

    for arg in call.arguments.iter() {
        let Some(expr) = arg.as_expression() else {
            continue;
        };
        let Some(value) = records::getter_source_from_function(result, expr, source) else {
            continue;
        };
        getters.insert(value.getter_name, value.source_name);
    }

    if getters.is_empty() {
        return;
    }

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
    result.reactive_value_origins.insert(
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

    result.reactivity.record_plain_value_alias(
        value.source_name.clone(),
        value.argument_name,
        CompactString::new(target_name),
        value.start,
        value.end,
    );
    result.reactive_value_origins.insert(
        CompactString::new(target_name),
        ReactiveValueOrigin::PlainAlias {
            source_name: value.source_name,
        },
    );
}

/// Check `alias = count` where `count` is already a plain reactive snapshot.
#[inline]
pub fn check_reactive_plain_assignment_alias(
    result: &mut ScriptParseResult,
    target_name: &str,
    init: &Expression<'_>,
) {
    if result.reactivity.is_reactive(target_name) {
        return;
    }

    let Some(value) = sources::reactive_plain_identifier_value_from_expr(result, init) else {
        return;
    };
    if value.argument_name.as_str() == target_name {
        return;
    }

    result.reactivity.record_plain_value_alias(
        value.source_name.clone(),
        value.argument_name,
        CompactString::new(target_name),
        value.start,
        value.end,
    );
    result.reactive_value_origins.insert(
        CompactString::new(target_name),
        ReactiveValueOrigin::PlainAlias {
            source_name: value.source_name,
        },
    );
}

pub fn check_ref_value_extraction(
    result: &mut ScriptParseResult,
    id: &oxc_ast::ast::BindingPattern<'_>,
    init: &Expression<'_>,
) {
    // Only check simple identifier bindings
    let target_name = match id {
        oxc_ast::ast::BindingPattern::BindingIdentifier(id) => id.name.as_str(),
        _ => return,
    };

    // Check for ref.value pattern: someRef.value
    if let Expression::StaticMemberExpression(member) = init
        && member.property.name.as_str() == "value"
        && let Expression::Identifier(obj_id) = &member.object
    {
        let ref_name = CompactString::new(obj_id.name.as_str());
        if result.reactivity.needs_value_access(ref_name.as_str()) {
            use crate::reactivity::{ReactivityLoss, ReactivityLossKind};
            result.reactivity.add_loss(ReactivityLoss {
                kind: ReactivityLossKind::RefValueExtract {
                    source_name: ref_name.clone(),
                    target_name: CompactString::new(target_name),
                },
                start: member.span.start,
                end: member.span.end,
            });
            result.reactive_value_origins.insert(
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
    init: &Expression<'_>,
) {
    let target_name = match id {
        oxc_ast::ast::BindingPattern::BindingIdentifier(id) => id.name.as_str(),
        _ => return,
    };

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

    result.reactivity.record_property_extract(
        source_name.clone(),
        prop_name.clone(),
        CompactString::new(target_name),
        init.span().start,
        init.span().end,
    );
    result.reactive_value_origins.insert(
        CompactString::new(target_name),
        ReactiveValueOrigin::ReactiveProperty {
            source_name,
            prop_name,
        },
    );
}
