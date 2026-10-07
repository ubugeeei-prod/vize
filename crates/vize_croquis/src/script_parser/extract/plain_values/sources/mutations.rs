//! Resolve member-mutation origins without changing live proxy semantics.

use super::super::super::common;
use super::{
    CompactString, ReactivePlainValue, ReactiveValueOrigin, ScriptParseResult, plain_origin_labels,
};
use oxc_ast::ast::{AssignmentTarget, Expression, SimpleAssignmentTarget};

pub(in crate::script_parser::extract::plain_values) fn reactive_plain_value_from_assignment_target(
    result: &ScriptParseResult,
    target: &AssignmentTarget<'_>,
    source: &str,
) -> Option<ReactivePlainValue> {
    match target {
        // Replacing the local binding never writes through its previous value.
        AssignmentTarget::AssignmentTargetIdentifier(_) => None,
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

pub(in crate::script_parser::extract::plain_values) fn reactive_plain_value_from_simple_assignment_target(
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

pub(in crate::script_parser::extract::plain_values) fn reactive_plain_value_from_mutated_expression(
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
    if keeps_live_ref_value(result, value.source_name.as_str()) {
        return None;
    }
    value.argument_name = common::expression_label(source, oxc_span::Span::new(start, end));
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
    let origin = result.reactive_origin(binding_name)?;
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

/// Member writes through deep refs and template nodes do not lose a subscription.
pub(in crate::script_parser::extract::plain_values) fn keeps_live_ref_value(
    result: &ScriptParseResult,
    source_name: &str,
) -> bool {
    let Some((root, suffix)) = source_name.split_once(".value") else {
        return false;
    };
    (suffix.is_empty() || suffix.starts_with('.')) && result.live_ref_value_sources.contains(root)
}
