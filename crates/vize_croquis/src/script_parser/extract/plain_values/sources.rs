use oxc_ast::ast::{AssignmentTarget, CallExpression, Expression, SimpleAssignmentTarget};
use oxc_span::GetSpan;

use vize_carton::{CompactString, cstr};

use super::super::super::{ReactiveValueOrigin, ScriptParseResult};
use super::ReactivePlainValue;

pub(super) fn reactive_plain_value_from_expr(
    result: &ScriptParseResult,
    expr: &Expression<'_>,
    source: &str,
) -> Option<ReactivePlainValue> {
    match expr {
        Expression::Identifier(id) => {
            let binding_name = id.name.as_str();
            let origin = result.reactive_value_origins.get(binding_name)?;
            let (source_name, getter_name) = plain_origin_labels(origin, binding_name);
            Some(ReactivePlainValue {
                source_name,
                argument_name: CompactString::new(binding_name),
                getter_name,
                start: id.span.start,
                end: id.span.end,
            })
        }
        Expression::StaticMemberExpression(member) => {
            if is_ref_value_member_root(result, expr) {
                return Some(ReactivePlainValue {
                    source_name: super::super::common::expression_label(source, member.span),
                    argument_name: super::super::common::expression_label(source, member.span),
                    getter_name: super::super::common::expression_label(source, member.span),
                    start: member.span.start,
                    end: member.span.end,
                });
            }

            if member.property.name.as_str() == "value"
                && let Some(root) =
                    super::super::common::member_chain_root_identifier(&member.object)
                && result.reactivity.needs_value_access(root.as_str())
            {
                return Some(ReactivePlainValue {
                    source_name: super::super::common::expression_label(source, member.span),
                    argument_name: super::super::common::expression_label(source, member.span),
                    getter_name: root,
                    start: member.span.start,
                    end: member.span.end,
                });
            }

            let (root, prop_name) = super::super::common::extract_member_chain_root(expr)?;
            if result
                .reactivity
                .lookup(root.as_str())
                .is_some_and(|source| !source.kind.needs_value_access())
            {
                return Some(ReactivePlainValue {
                    source_name: super::super::common::expression_label(source, member.span),
                    argument_name: super::super::common::expression_label(source, member.span),
                    getter_name: prop_name,
                    start: member.span.start,
                    end: member.span.end,
                });
            }

            let root_origin = result.reactive_value_origins.get(root.as_str())?;
            let (source_name, _) = plain_origin_labels(root_origin, root.as_str());
            Some(ReactivePlainValue {
                source_name,
                argument_name: super::super::common::expression_label(source, member.span),
                getter_name: prop_name,
                start: member.span.start,
                end: member.span.end,
            })
        }
        Expression::ComputedMemberExpression(member) => {
            if is_ref_value_member_root(result, expr) {
                return Some(ReactivePlainValue {
                    source_name: super::super::common::expression_label(source, member.span),
                    argument_name: super::super::common::expression_label(source, member.span),
                    getter_name: super::super::common::expression_label(source, member.span),
                    start: member.span.start,
                    end: member.span.end,
                });
            }

            let root = super::super::common::member_chain_root_identifier(&member.object)?;
            if result
                .reactivity
                .lookup(root.as_str())
                .is_some_and(|source| !source.kind.needs_value_access())
            {
                return Some(ReactivePlainValue {
                    source_name: super::super::common::expression_label(source, member.span),
                    argument_name: super::super::common::expression_label(source, member.span),
                    getter_name: super::super::common::expression_label(source, member.span),
                    start: member.span.start,
                    end: member.span.end,
                });
            }
            None
        }
        Expression::CallExpression(_) => getter_call_plain_value(result, expr, source),
        Expression::ChainExpression(chain) => match &chain.expression {
            oxc_ast::ast::ChainElement::CallExpression(_) => {
                getter_call_plain_value(result, expr, source)
            }
            oxc_ast::ast::ChainElement::StaticMemberExpression(member) => {
                reactive_plain_value_from_expr(result, &member.object, source)
            }
            oxc_ast::ast::ChainElement::ComputedMemberExpression(member) => {
                reactive_plain_value_from_expr(result, &member.object, source)
            }
            oxc_ast::ast::ChainElement::TSNonNullExpression(expr) => {
                reactive_plain_value_from_expr(result, &expr.expression, source)
            }
            oxc_ast::ast::ChainElement::PrivateFieldExpression(field) => {
                reactive_plain_value_from_expr(result, &field.object, source)
            }
        },
        Expression::ParenthesizedExpression(paren) => {
            reactive_plain_value_from_expr(result, &paren.expression, source)
        }
        Expression::TSAsExpression(ts_as) => {
            reactive_plain_value_from_expr(result, &ts_as.expression, source)
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            reactive_plain_value_from_expr(result, &ts_satisfies.expression, source)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            reactive_plain_value_from_expr(result, &ts_non_null.expression, source)
        }
        _ => None,
    }
}

pub(super) fn ref_value_property_extract(
    result: &ScriptParseResult,
    expr: &Expression<'_>,
) -> Option<(CompactString, CompactString)> {
    let (root, prop_name) = static_ref_value_access(result, expr)?;
    let prop_name = prop_name?;
    Some((cstr!("{root}.value"), CompactString::new(prop_name)))
}

pub(super) fn ref_value_destructure_source(
    result: &ScriptParseResult,
    expr: &Expression<'_>,
    source: &str,
) -> Option<(CompactString, bool)> {
    let (root, prop_name) = static_ref_value_access(result, expr)?;
    if prop_name.is_none() {
        Some((CompactString::new(root), true))
    } else {
        Some((
            super::super::common::expression_label(source, expr.span()),
            false,
        ))
    }
}

pub(super) fn reactive_member_destructure_source(
    result: &ScriptParseResult,
    expr: &Expression<'_>,
    source: &str,
) -> Option<CompactString> {
    match expr {
        Expression::StaticMemberExpression(member) => {
            let root = super::super::common::member_chain_root_identifier(&member.object)?;
            if result
                .reactivity
                .lookup(root.as_str())
                .is_some_and(|source| !source.kind.needs_value_access())
                || result.reactive_value_origins.contains_key(root.as_str())
            {
                return Some(super::super::common::expression_label(source, member.span));
            }
            None
        }
        Expression::ComputedMemberExpression(member) => {
            let root = super::super::common::member_chain_root_identifier(&member.object)?;
            if result
                .reactivity
                .lookup(root.as_str())
                .is_some_and(|source| !source.kind.needs_value_access())
                || result.reactive_value_origins.contains_key(root.as_str())
            {
                return Some(super::super::common::expression_label(source, member.span));
            }
            None
        }
        Expression::ParenthesizedExpression(paren) => {
            reactive_member_destructure_source(result, &paren.expression, source)
        }
        Expression::TSAsExpression(ts_as) => {
            reactive_member_destructure_source(result, &ts_as.expression, source)
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            reactive_member_destructure_source(result, &ts_satisfies.expression, source)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            reactive_member_destructure_source(result, &ts_non_null.expression, source)
        }
        _ => None,
    }
}

pub(super) fn reactive_expression_label_for_spread(
    result: &ScriptParseResult,
    expr: &Expression<'_>,
    source: &str,
) -> Option<CompactString> {
    match expr {
        Expression::Identifier(id) => {
            let name = id.name.as_str();
            (result.reactivity.is_reactive(name)
                || result.reactive_value_origins.contains_key(name))
            .then(|| CompactString::new(name))
        }
        Expression::StaticMemberExpression(member) => {
            if is_ref_value_member_root(result, expr) {
                return Some(super::super::common::expression_label(source, member.span));
            }
            let root = super::super::common::member_chain_root_identifier(&member.object)?;
            if result
                .reactivity
                .lookup(root.as_str())
                .is_some_and(|source| !source.kind.needs_value_access())
                || result.reactive_value_origins.contains_key(root.as_str())
            {
                return Some(super::super::common::expression_label(source, member.span));
            }
            None
        }
        Expression::ComputedMemberExpression(member) => {
            if is_ref_value_member_root(result, expr) {
                return Some(super::super::common::expression_label(source, member.span));
            }
            let root = super::super::common::member_chain_root_identifier(&member.object)?;
            if result
                .reactivity
                .lookup(root.as_str())
                .is_some_and(|source| !source.kind.needs_value_access())
                || result.reactive_value_origins.contains_key(root.as_str())
            {
                return Some(super::super::common::expression_label(source, member.span));
            }
            None
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            oxc_ast::ast::ChainElement::StaticMemberExpression(member) => {
                reactive_expression_label_for_spread(result, &member.object, source)
                    .map(|_| super::super::common::expression_label(source, member.span))
            }
            oxc_ast::ast::ChainElement::ComputedMemberExpression(member) => {
                reactive_expression_label_for_spread(result, &member.object, source)
                    .map(|_| super::super::common::expression_label(source, member.span))
            }
            oxc_ast::ast::ChainElement::TSNonNullExpression(expr) => {
                reactive_expression_label_for_spread(result, &expr.expression, source)
            }
            _ => None,
        },
        Expression::ParenthesizedExpression(paren) => {
            reactive_expression_label_for_spread(result, &paren.expression, source)
        }
        Expression::TSAsExpression(ts_as) => {
            reactive_expression_label_for_spread(result, &ts_as.expression, source)
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            reactive_expression_label_for_spread(result, &ts_satisfies.expression, source)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            reactive_expression_label_for_spread(result, &ts_non_null.expression, source)
        }
        _ => None,
    }
}

pub(super) fn reactive_plain_identifier_value_from_expr(
    result: &ScriptParseResult,
    expr: &Expression<'_>,
) -> Option<ReactivePlainValue> {
    match expr {
        Expression::Identifier(id) => {
            let binding_name = id.name.as_str();
            let origin = result.reactive_value_origins.get(binding_name)?;
            let (source_name, _) = plain_origin_labels(origin, binding_name);
            Some(ReactivePlainValue {
                source_name,
                argument_name: CompactString::new(binding_name),
                getter_name: CompactString::new(binding_name),
                start: id.span.start,
                end: id.span.end,
            })
        }
        Expression::ParenthesizedExpression(paren) => {
            reactive_plain_identifier_value_from_expr(result, &paren.expression)
        }
        Expression::TSAsExpression(ts_as) => {
            reactive_plain_identifier_value_from_expr(result, &ts_as.expression)
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            reactive_plain_identifier_value_from_expr(result, &ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            reactive_plain_identifier_value_from_expr(result, &ts_non_null.expression)
        }
        _ => None,
    }
}

fn is_ref_value_member_root(result: &ScriptParseResult, expr: &Expression<'_>) -> bool {
    static_ref_value_access(result, expr).is_some()
}

fn static_ref_value_access<'a>(
    result: &ScriptParseResult,
    expr: &'a Expression<'a>,
) -> Option<(&'a str, Option<&'a str>)> {
    let mut current = expr;
    let mut previous_prop = None;
    let mut prop_after_value = None;

    loop {
        match current {
            Expression::StaticMemberExpression(member) => {
                let prop_name = member.property.name.as_str();
                if prop_name == "value" {
                    prop_after_value = previous_prop;
                }
                previous_prop = Some(prop_name);
                current = &member.object;
            }
            Expression::Identifier(id) => {
                let root = id.name.as_str();
                return prop_after_value
                    .or_else(|| previous_prop.filter(|prop| *prop == "value"))
                    .is_some()
                    .then_some((root, prop_after_value))
                    .filter(|(root, _)| result.reactivity.needs_value_access(root));
            }
            Expression::ParenthesizedExpression(paren) => current = &paren.expression,
            Expression::TSAsExpression(ts_as) => current = &ts_as.expression,
            Expression::TSSatisfiesExpression(ts_satisfies) => current = &ts_satisfies.expression,
            Expression::TSNonNullExpression(ts_non_null) => current = &ts_non_null.expression,
            _ => return None,
        }
    }
}

pub(super) fn reactive_plain_value_from_assignment_target(
    result: &ScriptParseResult,
    target: &AssignmentTarget<'_>,
    source: &str,
) -> Option<ReactivePlainValue> {
    match target {
        AssignmentTarget::AssignmentTargetIdentifier(id) => {
            reactive_plain_mutation_identifier_value(
                result,
                id.name.as_str(),
                id.span.start,
                id.span.end,
            )
        }
        AssignmentTarget::StaticMemberExpression(member) => {
            reactive_plain_value_from_mutated_member(
                result,
                &member.object,
                source,
                member.span.start,
                member.span.end,
            )
        }
        AssignmentTarget::ComputedMemberExpression(member) => {
            reactive_plain_value_from_mutated_member(
                result,
                &member.object,
                source,
                member.span.start,
                member.span.end,
            )
        }
        _ => None,
    }
}

pub(super) fn reactive_plain_value_from_simple_assignment_target(
    result: &ScriptParseResult,
    target: &SimpleAssignmentTarget<'_>,
    source: &str,
) -> Option<ReactivePlainValue> {
    match target {
        SimpleAssignmentTarget::AssignmentTargetIdentifier(id) => {
            reactive_plain_mutation_identifier_value(
                result,
                id.name.as_str(),
                id.span.start,
                id.span.end,
            )
        }
        SimpleAssignmentTarget::StaticMemberExpression(member) => {
            reactive_plain_value_from_mutated_member(
                result,
                &member.object,
                source,
                member.span.start,
                member.span.end,
            )
        }
        SimpleAssignmentTarget::ComputedMemberExpression(member) => {
            reactive_plain_value_from_mutated_member(
                result,
                &member.object,
                source,
                member.span.start,
                member.span.end,
            )
        }
        _ => None,
    }
}

pub(super) fn reactive_plain_value_from_mutated_expression(
    result: &ScriptParseResult,
    expr: &Expression<'_>,
    source: &str,
) -> Option<ReactivePlainValue> {
    match expr {
        Expression::Identifier(id) => reactive_plain_mutation_identifier_value(
            result,
            id.name.as_str(),
            id.span.start,
            id.span.end,
        ),
        Expression::StaticMemberExpression(member) => reactive_plain_value_from_mutated_member(
            result,
            &member.object,
            source,
            member.span.start,
            member.span.end,
        ),
        Expression::ComputedMemberExpression(member) => reactive_plain_value_from_mutated_member(
            result,
            &member.object,
            source,
            member.span.start,
            member.span.end,
        ),
        Expression::ChainExpression(chain) => match &chain.expression {
            oxc_ast::ast::ChainElement::StaticMemberExpression(member) => {
                reactive_plain_value_from_mutated_member(
                    result,
                    &member.object,
                    source,
                    member.span.start,
                    member.span.end,
                )
            }
            oxc_ast::ast::ChainElement::ComputedMemberExpression(member) => {
                reactive_plain_value_from_mutated_member(
                    result,
                    &member.object,
                    source,
                    member.span.start,
                    member.span.end,
                )
            }
            oxc_ast::ast::ChainElement::TSNonNullExpression(expr) => {
                reactive_plain_value_from_mutated_expression(result, &expr.expression, source)
            }
            _ => None,
        },
        Expression::ParenthesizedExpression(paren) => {
            reactive_plain_value_from_mutated_expression(result, &paren.expression, source)
        }
        Expression::TSAsExpression(ts_as) => {
            reactive_plain_value_from_mutated_expression(result, &ts_as.expression, source)
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            reactive_plain_value_from_mutated_expression(result, &ts_satisfies.expression, source)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            reactive_plain_value_from_mutated_expression(result, &ts_non_null.expression, source)
        }
        _ => None,
    }
}

fn reactive_plain_value_from_mutated_member(
    result: &ScriptParseResult,
    object: &Expression<'_>,
    source: &str,
    start: u32,
    end: u32,
) -> Option<ReactivePlainValue> {
    let mut value = reactive_plain_value_from_mutated_expression(result, object, source)?;
    value.argument_name =
        super::super::common::expression_label(source, oxc_span::Span::new(start, end));
    value.start = start;
    value.end = end;
    Some(value)
}

fn reactive_plain_mutation_identifier_value(
    result: &ScriptParseResult,
    binding_name: &str,
    start: u32,
    end: u32,
) -> Option<ReactivePlainValue> {
    let origin = result.reactive_value_origins.get(binding_name)?;
    if matches!(origin, ReactiveValueOrigin::PropsDestructure { .. }) {
        return None;
    }
    let (source_name, _) = plain_origin_labels(origin, binding_name);
    Some(ReactivePlainValue {
        source_name,
        argument_name: CompactString::new(binding_name),
        getter_name: CompactString::new(binding_name),
        start,
        end,
    })
}

fn getter_call_plain_value(
    result: &ScriptParseResult,
    expr: &Expression<'_>,
    source: &str,
) -> Option<ReactivePlainValue> {
    let (context_name, getter_name, source_name, _) = getter_call_source(result, expr)?;
    Some(ReactivePlainValue {
        source_name,
        argument_name: super::super::common::expression_label(source, expr.span()),
        getter_name,
        start: expr.span().start,
        end: expr.span().end,
    })
    .filter(|_| !context_name.is_empty())
}

pub(super) fn getter_call_source(
    result: &ScriptParseResult,
    expr: &Expression<'_>,
) -> Option<(CompactString, CompactString, CompactString, CompactString)> {
    match expr {
        Expression::CallExpression(call) => getter_call_source_from_call(result, call),
        Expression::ParenthesizedExpression(paren) => getter_call_source(result, &paren.expression),
        Expression::TSAsExpression(ts_as) => getter_call_source(result, &ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            getter_call_source(result, &ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            getter_call_source(result, &ts_non_null.expression)
        }
        _ => None,
    }
}

pub(super) fn getter_call_source_from_call(
    result: &ScriptParseResult,
    call: &CallExpression<'_>,
) -> Option<(CompactString, CompactString, CompactString, CompactString)> {
    let Expression::StaticMemberExpression(member) = &call.callee else {
        return None;
    };
    let Expression::Identifier(context) = &member.object else {
        return None;
    };

    let context_name = CompactString::new(context.name.as_str());
    let getter_name = CompactString::new(member.property.name.as_str());
    let context = result.reactive_getter_contexts.get(context_name.as_str())?;
    let source_name = context.getters.get(getter_name.as_str())?.clone();

    Some((
        context_name,
        getter_name,
        source_name,
        context.callee_name.clone(),
    ))
}

fn plain_origin_labels(
    origin: &ReactiveValueOrigin,
    binding_name: &str,
) -> (CompactString, CompactString) {
    match origin {
        ReactiveValueOrigin::PropsDestructure { prop_name } => {
            (prop_name.clone(), prop_name.clone())
        }
        ReactiveValueOrigin::ReactiveProperty {
            source_name,
            prop_name,
        } => (cstr!("{source_name}.{prop_name}"), prop_name.clone()),
        ReactiveValueOrigin::RefValue { source_name } => {
            (cstr!("{source_name}.value"), source_name.clone())
        }
        ReactiveValueOrigin::GetterCall {
            context_name: _context_name,
            getter_name,
            source_name,
        } => (source_name.clone(), getter_name.clone()),
        ReactiveValueOrigin::PlainAlias { source_name } => {
            (source_name.clone(), CompactString::new(binding_name))
        }
    }
}
