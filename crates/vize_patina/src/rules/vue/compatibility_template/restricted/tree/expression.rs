use super::super::super::origin::Origin;
use super::{node, span};
use crate::context::LintContext;
use oxc_ast::ast::Expression;
use oxc_estree::{CompactSerializer, ESTree};
use oxc_span::Span as OxcSpan;
use serde_json::{Value, json};
use vize_l1::markup::entity::EntityContext;
use vize_relief::ExpressionNode;

pub(super) fn expression_span(expression: &ExpressionNode<'_>) -> (usize, usize) {
    span(expression.loc().span)
}

/// The expression a container holds, as vue-eslint-parser shapes it.
pub(super) fn expression_value<'a>(
    ctx: &mut LintContext<'a>,
    expression: &ExpressionNode<'a>,
    entity_context: EntityContext,
    directive: Option<&str>,
) -> Value {
    let ExpressionNode::Simple(simple) = expression else {
        return Value::Null;
    };
    let Some(origin) = Origin::new(ctx.source, simple.content, simple.loc.span, entity_context)
    else {
        return Value::Null;
    };
    let (start, end) = span(simple.loc.span);
    let retained = simple
        .js_ast
        .filter(|retained| retained.raw == simple.content);
    if let Some(expression) = retained.and_then(|retained| retained.as_expression()) {
        let value = estree(expression.ast, &origin, 0);
        // A v-on value that is not a function or a member path is a statement list.
        return if directive == Some("on") && !is_handler_path(expression.ast) {
            node(
                "VOnExpression",
                start,
                end,
                [(
                    "body",
                    json!([{ "type": "ExpressionStatement", "expression": value,
                    "start": start, "end": end }]),
                )],
            )
        } else {
            value
        };
    }
    if let Some(Ok(parameters)) = retained.and_then(|retained| retained.as_slot_parameters()) {
        return node(
            "VSlotScopeExpression",
            start,
            end,
            [("params", estree(parameters, &origin, 0))],
        );
    }
    match directive {
        Some("for") => ctx
            .compatibility_v_for(simple.loc.span, simple.content)
            .map_or(Value::Null, |retained| {
                node(
                    "VForExpression",
                    start,
                    end,
                    [
                        (
                            "left",
                            estree(retained.parameters, &origin, retained.parameters_offset),
                        ),
                        (
                            "right",
                            estree(retained.source, &origin, retained.source_offset),
                        ),
                    ],
                )
            }),
        Some("on") => ctx
            .compatibility_handler(simple.loc.span, simple.content)
            .map_or(Value::Null, |program| {
                let body = program
                    .body
                    .iter()
                    .map(|statement| estree(statement, &origin, 0))
                    .collect();
                node("VOnExpression", start, end, [("body", Value::Array(body))])
            }),
        _ => Value::Null,
    }
}

fn is_handler_path(expression: &Expression<'_>) -> bool {
    match expression.without_parentheses() {
        Expression::Identifier(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::FunctionExpression(_) => true,
        expression => expression
            .as_member_expression()
            .is_some_and(|member| is_handler_path(member.object())),
    }
}

/// OXC's TS-ESTree JSON of `node`, with authored offsets and no parentheses.
fn estree(node: &impl ESTree, origin: &Origin, offset: u32) -> Value {
    let mut serializer = CompactSerializer::new(true, false);
    node.serialize(&mut serializer);
    let mut value = serde_json::from_str(&serializer.into_string()).unwrap_or(Value::Null);
    normalize(&mut value, origin, offset);
    value
}

fn normalize(value: &mut Value, origin: &Origin, offset: u32) {
    while value.get("type").and_then(Value::as_str) == Some("ParenthesizedExpression") {
        let inner = value
            .get_mut("expression")
            .map(Value::take)
            .unwrap_or(Value::Null);
        *value = inner;
    }
    match value {
        Value::Object(map) => {
            for key in ["start", "end"] {
                if let Some(position) = map.get(key).and_then(Value::as_u64) {
                    let position = u32::try_from(position)
                        .ok()
                        .and_then(|position| position.checked_add(offset))
                        .and_then(|position| origin.range(OxcSpan::new(position, position)));
                    map.insert(
                        key.into(),
                        position.map_or(Value::Null, |range| json!(range.start)),
                    );
                }
            }
            for (key, child) in map.iter_mut() {
                if key != "start" && key != "end" {
                    normalize(child, origin, offset);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                normalize(item, origin, offset);
            }
        }
        _ => {}
    }
}
