use oxc_ast::ast::{CallExpression, Expression};

use super::super::super::ScriptParseResult;

pub(super) fn is_reactivity_loss_value_sink_call(
    result: &ScriptParseResult,
    call: &CallExpression<'_>,
) -> bool {
    match &call.callee {
        Expression::Identifier(id) => {
            is_reactivity_loss_value_sink_identifier(id.name.as_str())
                || super::super::common::resolved_call_name(result, call)
                    .is_some_and(|name| is_reactivity_loss_value_sink_identifier(name.as_str()))
        }
        Expression::StaticMemberExpression(member) => {
            let Some(root) = super::super::common::member_chain_root_identifier(&member.object)
            else {
                return false;
            };
            is_reactivity_loss_value_sink_member(root.as_str(), member.property.name.as_str())
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            oxc_ast::ast::ChainElement::StaticMemberExpression(member) => {
                let Some(root) = super::super::common::member_chain_root_identifier(&member.object)
                else {
                    return false;
                };
                is_reactivity_loss_value_sink_member(root.as_str(), member.property.name.as_str())
            }
            _ => false,
        },
        _ => false,
    }
}

fn is_reactivity_loss_value_sink_identifier(name: &str) -> bool {
    matches!(
        name,
        "emit"
            | "$emit"
            | "Number"
            | "String"
            | "Boolean"
            | "BigInt"
            | "Symbol"
            | "parseInt"
            | "parseFloat"
            | "isFinite"
            | "isNaN"
            | "encodeURI"
            | "encodeURIComponent"
            | "decodeURI"
            | "decodeURIComponent"
    )
}

fn is_reactivity_loss_value_sink_member(root: &str, property: &str) -> bool {
    match root {
        "console" => matches!(
            property,
            "assert"
                | "debug"
                | "dir"
                | "error"
                | "group"
                | "groupCollapsed"
                | "info"
                | "log"
                | "table"
                | "time"
                | "timeEnd"
                | "timeLog"
                | "trace"
                | "warn"
        ),
        "Math" => true,
        "JSON" => matches!(property, "parse" | "stringify"),
        "Number" => matches!(
            property,
            "isFinite" | "isInteger" | "isNaN" | "isSafeInteger"
        ),
        "String" => matches!(property, "fromCharCode" | "fromCodePoint" | "raw"),
        "Array" => matches!(property, "isArray"),
        _ => false,
    }
}
