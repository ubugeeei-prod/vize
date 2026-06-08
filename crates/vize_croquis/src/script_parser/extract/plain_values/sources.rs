use oxc_ast::ast::{CallExpression, Expression};
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
        ReactiveValueOrigin::FunctionArgument {
            source_name,
            callee_name: _callee_name,
        } => (source_name.clone(), CompactString::new(binding_name)),
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
