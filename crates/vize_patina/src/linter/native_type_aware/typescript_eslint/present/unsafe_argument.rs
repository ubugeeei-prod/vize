//! `no-unsafe-argument` past a variadic tuple spread.
use corsa::lint::LintNode;
use serde_json::Value;

/// Compatibility boundary for corsa_core 2.0.0-beta.5. Its resolved-signature
/// check stops at a spread whose tuple has a rest element, since argument
/// positions no longer line up with parameters after it. typescript-eslint's
/// `FunctionSignature.consumeRemainingArguments` instead matches every later
/// argument against the rest type (`foo(...[a, ...rest], 1 as any)`). The
/// host resolves that type (`__callParameters.rest`); this presents the later
/// arguments as a call of their own whose every position is the rest type, so
/// the rule's own unsafe-assignment decision runs on them. `None` when there
/// is nothing past such a spread.
pub(super) fn after_variadic_spread(node: &LintNode) -> Option<LintNode> {
    if !matches!(node.kind.as_str(), "CallExpression" | "NewExpression") {
        return None;
    }
    let parameters = node.fields.get("__callParameters")?;
    let rest = parameters.get("rest").filter(|rest| !rest.is_null())?;
    let arguments = node.child_list("arguments")?;
    let spread = arguments.iter().position(is_variadic_tuple_spread)?;
    let tail = arguments
        .get(spread + 1..)
        .filter(|tail| !tail.is_empty())?;
    let positions = tail.iter().map(consumed_positions).sum::<usize>();
    let mut call = parameters.clone();
    call.as_object_mut()?.insert(
        "positions".into(),
        Value::Array(vec![rest.clone(); positions]),
    );
    let mut presented = node.clone();
    presented
        .child_lists
        .insert("arguments".into(), tail.to_vec());
    presented.fields.insert("__callParameters".into(), call);
    Some(presented)
}

fn tuple_detail(argument: &LintNode) -> Option<&Value> {
    if argument.kind != "SpreadElement" {
        return None;
    }
    argument
        .child("argument")?
        .fields
        .get("__typeDetail")
        .filter(|detail| detail.get("isTuple").and_then(Value::as_bool) == Some(true))
}

/// The spread the rule stops at: a tuple whose type text has a rest element
/// (or no text to tell).
fn is_variadic_tuple_spread(argument: &LintNode) -> bool {
    tuple_detail(argument).is_some_and(|detail| {
        detail
            .get("text")
            .and_then(Value::as_str)
            .is_none_or(|text| text.contains("..."))
    })
}

/// Parameter positions an argument consumes: one, or each element of a
/// spread tuple.
fn consumed_positions(argument: &LintNode) -> usize {
    tuple_detail(argument)
        .and_then(|detail| detail.get("typeArguments"))
        .and_then(Value::as_array)
        .map_or(1, Vec::len)
}

#[cfg(test)]
mod tests {
    use super::after_variadic_spread;
    use corsa::lint::LintNode;
    use serde_json::{Value, json};

    fn call(rest: Value) -> LintNode {
        serde_json::from_value(json!({
            "kind": "CallExpression",
            "range": { "start": 0, "end": 30 },
            "fields": {
                "__callParameters": {
                    "parameterCount": 1,
                    "hasRest": true,
                    "positions": [{ "texts": ["string"] }, { "texts": ["string"] }],
                    "rest": rest,
                }
            },
            "childLists": {
                "arguments": [
                    {
                        "kind": "SpreadElement",
                        "range": { "start": 4, "end": 13 },
                        "children": {
                            "argument": {
                                "kind": "Identifier",
                                "range": { "start": 7, "end": 13 },
                                "fields": { "__typeDetail": {
                                    "flags": 1 << 20,
                                    "isTuple": true,
                                    "text": "[string, ...string[]]",
                                    "typeArguments": [{ "flags": 32 }, { "flags": 32 }],
                                } }
                            }
                        }
                    },
                    { "kind": "TSAsExpression", "range": { "start": 15, "end": 23 } }
                ]
            }
        }))
        .expect("call node")
    }

    #[test]
    fn later_arguments_meet_the_rest_type() {
        let rest = json!({ "texts": ["string"], "detail": { "flags": 32 } });
        let presented = after_variadic_spread(&call(rest.clone())).expect("tail");
        let arguments = presented.child_list("arguments").expect("arguments");
        assert_eq!(arguments.len(), 1);
        assert_eq!(arguments[0].range.start, 15);
        assert_eq!(
            presented.fields["__callParameters"]["positions"],
            json!([rest])
        );
        assert!(after_variadic_spread(&call(Value::Null)).is_none());
    }
}
