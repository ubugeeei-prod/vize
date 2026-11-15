//! Rule-local compatibility for `require-await` return values.
use corsa::lint::LintNode;
use serde_json::Value;

const TYPE_FLAG_ANY: u64 = 1;

/// Expose non-thenable `any` returns to `require-await` without changing the
/// checker facts shared with the other type-aware rules.
///
/// `corsa_core` 2.0.0-beta.5 treats every `any` return as possibly thenable,
/// while typescript-eslint's rule asks whether the return type is actually
/// thenable. The rule receives a clone, so clearing this one compatibility bit
/// cannot change `any` handling in the unsafe, await or rejection rules.
pub(super) fn expose_non_thenable_any_returns(function: &mut LintNode) {
    if function.kind == "ArrowFunctionExpression"
        && let Some(body) = function.children.get_mut("body")
        && body.kind != "BlockStatement"
    {
        expose_expression(body);
        return;
    }
    if let Some(body) = function.children.get_mut("body") {
        visit_scope(body);
    }
}

fn visit_scope(node: &mut LintNode) {
    if is_function(node) {
        return;
    }
    if node.kind == "ReturnStatement"
        && let Some(argument) = node.children.get_mut("argument")
    {
        expose_expression(argument);
    }
    for child in node.children.values_mut() {
        visit_scope(child);
    }
    for children in node.child_lists.values_mut() {
        for child in children {
            visit_scope(child);
        }
    }
}

fn expose_expression(node: &mut LintNode) {
    if let Some(facts) = node
        .fields
        .get_mut("__promiseFacts")
        .and_then(Value::as_object_mut)
        && facts.get("thenable").and_then(Value::as_bool) == Some(false)
        && let Some(flags) = facts.get("flags").and_then(Value::as_u64)
    {
        facts.insert("flags".into(), Value::from(flags & !TYPE_FLAG_ANY));
    }
    if node.kind == "ChainExpression"
        && let Some(expression) = node.children.get_mut("expression")
    {
        expose_expression(expression);
    }
}

fn is_function(node: &LintNode) -> bool {
    matches!(
        node.kind.as_str(),
        "FunctionDeclaration"
            | "FunctionExpression"
            | "ArrowFunctionExpression"
            | "TSDeclareFunction"
            | "TSEmptyBodyFunctionExpression"
    )
}

#[cfg(test)]
mod tests {
    use corsa::lint::{LintNode, run_default_type_aware_rule_owned};
    use serde_json::{Value, json};

    use super::super::prepare;
    use super::TYPE_FLAG_ANY;

    fn function(argument: Value) -> LintNode {
        serde_json::from_value(json!({
            "kind": "FunctionDeclaration",
            "range": { "start": 0, "end": 48 },
            "fields": {
                "async": true,
                "generator": false,
                "__functionName": "foo"
            },
            "children": {
                "body": {
                    "kind": "BlockStatement",
                    "range": { "start": 20, "end": 48 },
                    "childLists": {
                        "body": [{
                            "kind": "ReturnStatement",
                            "range": { "start": 24, "end": 44 },
                            "children": { "argument": argument }
                        }]
                    }
                }
            }
        }))
        .expect("function node")
    }

    fn expression(flags: u64, thenable: bool) -> Value {
        json!({
            "kind": "Identifier",
            "range": { "start": 31, "end": 39 },
            "fields": {
                "name": "value",
                "__promiseFacts": { "flags": flags, "thenable": thenable }
            }
        })
    }

    fn promise_flags(node: &LintNode) -> u64 {
        node.child("body")
            .and_then(|body| body.child_list("body"))
            .and_then(|body| body.first())
            .and_then(|statement| statement.child("argument"))
            .and_then(|argument| argument.fields.get("__promiseFacts"))
            .and_then(|facts| facts.get("flags"))
            .and_then(Value::as_u64)
            .expect("promise flags")
    }

    #[test]
    fn non_thenable_any_is_exposed_only_to_require_await() {
        let original = function(expression(TYPE_FLAG_ANY, false));
        assert!(
            run_default_type_aware_rule_owned("require-await", original.clone())
                .expect("registered rule")
                .is_empty()
        );

        let prepared = prepare("require-await", &original, None);
        let diagnostics = run_default_type_aware_rule_owned("require-await", prepared.clone())
            .expect("registered rule");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message,
            "Async function 'foo' has no 'await' expression."
        );
        assert_eq!(promise_flags(&original), TYPE_FLAG_ANY);
        assert_eq!(promise_flags(&prepared), 0);
        assert_eq!(
            promise_flags(&prepare("prefer-promise-reject-errors", &original, None)),
            TYPE_FLAG_ANY
        );
    }

    #[test]
    fn thenable_any_remains_an_async_return() {
        let original = function(expression(TYPE_FLAG_ANY, true));
        let prepared = prepare("require-await", &original, None);
        assert_eq!(promise_flags(&prepared), TYPE_FLAG_ANY);
        assert!(
            run_default_type_aware_rule_owned("require-await", prepared)
                .expect("registered rule")
                .is_empty()
        );
    }
}
