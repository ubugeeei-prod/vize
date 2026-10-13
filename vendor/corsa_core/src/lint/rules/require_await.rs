use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule};
use crate::lint::helpers::{
    is_obviously_promise_like, is_promise_like_node, strip_chain_expression,
};

/// Type-aware port of the typescript-eslint `require-await` rule.
///
/// Reports async functions whose body contains no `await` expression (and, for
/// async generators, that never yield a thenable / delegate an async iterable).
/// Faithful to tsgolint `require_await.go`.
#[derive(Clone, Copy, Debug, Default)]
pub struct RequireAwaitRule;

const MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "missingAwait",
    description: "Function has no 'await' expression.",
}];

// Mirrors the Go listener set:
//   FunctionDeclaration, MethodDeclaration, Constructor, GetAccessor,
//   SetAccessor, FunctionExpression, ArrowFunction.
// In TS-ESTree those surface as the three function nodes plus MethodDefinition
// (whose `value` is a FunctionExpression). We listen on the function nodes
// directly; MethodDefinition wrapping is handled because the wrapped
// FunctionExpression is itself visited.
const LISTENERS: &[&str] = &[
    "FunctionDeclaration",
    "FunctionExpression",
    "ArrowFunctionExpression",
];

impl RustLintRule for RequireAwaitRule {
    fn name(&self) -> &'static str {
        "require-await"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow async functions that contain no await expression."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        // Only async functions are candidates.
        if !node.field_bool("async").unwrap_or(false) {
            return;
        }

        // Go: `enterFunction` only records the async/generator function flags when
        // the body is absent OR a non-empty block (an empty `{}` body keeps the
        // flags Normal, so an empty async function is never reported).
        if has_empty_block_body(node) {
            return;
        }

        let is_generator = node.field_bool("generator").unwrap_or(false);

        // Body-less async arrow returning a thenable is treated as having await,
        // e.g. `async () => Promise.resolve()`.
        if node.kind == "ArrowFunctionExpression" {
            if let Some(body) = node.child("body") {
                let body = strip_chain_expression(body);
                if body.kind != "BlockStatement" && is_thenable_expression(body) {
                    return;
                }
            }
        }

        if scope_has_await(node) {
            return;
        }

        // For async generators, yielding a thenable or delegating to an async
        // iterable counts as "async use" and suppresses the report.
        if is_generator && scope_is_async_yield(node) {
            return;
        }

        ctx.report("missingAwait", node.range);
    }
}

/// Returns whether the function's body is an empty block statement.
fn has_empty_block_body(node: &LintNode) -> bool {
    node.child("body").is_some_and(|body| {
        body.kind == "BlockStatement"
            && body
                .child_list("body")
                .map(<[LintNode]>::is_empty)
                .unwrap_or(true)
    })
}

/// Walks the current function body (skipping nested functions) and returns
/// whether any `await`-like construct is present. Mirrors the Go listeners that
/// call `markAsHasAwait`: AwaitExpression, `for await`, `await using`, and a
/// `return`-of-thenable inside an async function.
fn scope_has_await(node: &LintNode) -> bool {
    let mut found = false;
    walk_function_body(node, &mut |candidate| {
        if found {
            return;
        }
        match candidate.kind.as_str() {
            "AwaitExpression" => found = true,
            // `for await (const x of y)`.
            "ForOfStatement" if candidate.field_bool("await").unwrap_or(false) => found = true,
            // `await using x = ...`.
            "VariableDeclaration" if matches!(candidate.field_str("kind"), Some("await using")) => {
                found = true;
            }
            // `return <thenable>` inside an async function counts as await.
            "ReturnStatement"
                if candidate
                    .child("argument")
                    .map(strip_chain_expression)
                    .is_some_and(is_thenable_expression) =>
            {
                found = true;
            }
            _ => {}
        }
    });
    found
}

/// Walks the current generator body and returns whether any `yield` produces an
/// async value: a thenable `yield`, or a `yield*` over an async-iterable.
/// Both require checker information, surfaced as raw facts on the yield node.
fn scope_is_async_yield(node: &LintNode) -> bool {
    let mut found = false;
    walk_function_body(node, &mut |candidate| {
        if found || candidate.kind != "YieldExpression" {
            return;
        }
        let Some(argument) = candidate.child("argument") else {
            return;
        };
        // Literal arguments are ignored, matching the Go fast-path.
        if argument.kind == "Literal" {
            return;
        }
        let is_delegate = candidate.field_bool("delegate").unwrap_or(false);
        if is_delegate {
            // `yield* x`: delegate is async if `x` exposes Symbol.asyncIterator.
            if candidate.field_bool("__yieldDelegatesAsyncIterable") == Some(true) {
                found = true;
            }
        } else if candidate.field_bool("__yieldArgumentThenable") == Some(true) {
            // `yield x`: async if `x` is thenable.
            found = true;
        }
    });
    found
}

/// Returns whether an expression is thenable, using both the structural
/// `Promise.resolve()` / `new Promise()` recognizers and the type-text facts.
fn is_thenable_expression(node: &LintNode) -> bool {
    is_obviously_promise_like(node) || is_promise_like_node(node)
}

/// Visits the function's body, descending into children but stopping at nested
/// function boundaries so `await`/`yield` inside nested functions do not leak
/// into the enclosing scope (matching the per-scope Go state machine).
fn walk_function_body<'a>(node: &'a LintNode, visit: &mut impl FnMut(&'a LintNode)) {
    let Some(body) = node.child("body") else {
        return;
    };
    walk_skipping_nested_functions(body, visit);
}

fn walk_skipping_nested_functions<'a>(node: &'a LintNode, visit: &mut impl FnMut(&'a LintNode)) {
    if is_function_like_kind(&node.kind) {
        return;
    }
    visit(node);
    for child in node.children.values() {
        walk_skipping_nested_functions(child, visit);
    }
    for list in node.child_lists.values() {
        for child in list {
            walk_skipping_nested_functions(child, visit);
        }
    }
}

fn is_function_like_kind(kind: &str) -> bool {
    matches!(
        kind,
        "FunctionDeclaration"
            | "FunctionExpression"
            | "ArrowFunctionExpression"
            | "TSDeclareFunction"
            | "TSEmptyBodyFunctionExpression"
    )
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::super::{LintNode, RuleContext, RustLintRule};
    use super::RequireAwaitRule;

    fn run(node: &LintNode) -> Vec<String> {
        let rule = RequireAwaitRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, node);
        ctx.finish()
            .into_iter()
            .map(|diagnostic| diagnostic.message_id)
            .collect()
    }

    fn from(value: serde_json::Value) -> LintNode {
        serde_json::from_value(value).expect("valid LintNode")
    }

    // async function numberOne(): Promise<number> { return 1; }  -> report
    #[test]
    fn reports_async_function_without_await() {
        let node = from(json!({
            "kind": "FunctionDeclaration",
            "range": { "start": 0, "end": 40 },
            "fields": { "async": true, "generator": false },
            "children": {
                "body": {
                    "kind": "BlockStatement",
                    "range": { "start": 30, "end": 40 },
                    "childLists": {
                        "body": [{
                            "kind": "ReturnStatement",
                            "range": { "start": 32, "end": 41 },
                            "children": {
                                "argument": {
                                    "kind": "Literal",
                                    "range": { "start": 39, "end": 40 },
                                    "fields": { "value": 1 }
                                }
                            }
                        }]
                    }
                }
            }
        }));
        assert_eq!(run(&node), vec!["missingAwait".to_owned()]);
    }

    // const numberOne = async (): Promise<number> => 1;  -> report
    #[test]
    fn reports_async_arrow_returning_plain_value() {
        let node = from(json!({
            "kind": "ArrowFunctionExpression",
            "range": { "start": 18, "end": 49 },
            "fields": { "async": true, "generator": false },
            "children": {
                "body": {
                    "kind": "Literal",
                    "range": { "start": 47, "end": 48 },
                    "fields": { "value": 1 }
                }
            }
        }));
        assert_eq!(run(&node), vec!["missingAwait".to_owned()]);
    }

    // async function* foo() { yield 1; }  -> report (no await, sync-style yield)
    #[test]
    fn reports_async_generator_without_await_or_async_yield() {
        let node = from(json!({
            "kind": "FunctionDeclaration",
            "range": { "start": 0, "end": 35 },
            "fields": { "async": true, "generator": true },
            "children": {
                "body": {
                    "kind": "BlockStatement",
                    "range": { "start": 22, "end": 35 },
                    "childLists": {
                        "body": [{
                            "kind": "ExpressionStatement",
                            "range": { "start": 26, "end": 33 },
                            "children": {
                                "expression": {
                                    "kind": "YieldExpression",
                                    "range": { "start": 26, "end": 33 },
                                    "fields": { "delegate": false },
                                    "children": {
                                        "argument": {
                                            "kind": "Literal",
                                            "range": { "start": 32, "end": 33 },
                                            "fields": { "value": 1 }
                                        }
                                    }
                                }
                            }
                        }]
                    }
                }
            }
        }));
        assert_eq!(run(&node), vec!["missingAwait".to_owned()]);
    }

    // async function numberOne(): Promise<number> { return await 1; }  -> ok
    #[test]
    fn ignores_async_function_with_await() {
        let node = from(json!({
            "kind": "FunctionDeclaration",
            "range": { "start": 0, "end": 46 },
            "fields": { "async": true, "generator": false },
            "children": {
                "body": {
                    "kind": "BlockStatement",
                    "range": { "start": 30, "end": 46 },
                    "childLists": {
                        "body": [{
                            "kind": "ReturnStatement",
                            "range": { "start": 32, "end": 45 },
                            "children": {
                                "argument": {
                                    "kind": "AwaitExpression",
                                    "range": { "start": 39, "end": 46 },
                                    "children": {
                                        "argument": {
                                            "kind": "Literal",
                                            "range": { "start": 45, "end": 46 },
                                            "fields": { "value": 1 }
                                        }
                                    }
                                }
                            }
                        }]
                    }
                }
            }
        }));
        assert!(run(&node).is_empty());
    }

    // function numberOne(): number { return 1; }  -> ok (not async)
    #[test]
    fn ignores_non_async_function() {
        let node = from(json!({
            "kind": "FunctionDeclaration",
            "range": { "start": 0, "end": 30 },
            "fields": { "async": false, "generator": false },
            "children": {
                "body": {
                    "kind": "BlockStatement",
                    "range": { "start": 20, "end": 30 },
                    "childLists": {
                        "body": [{
                            "kind": "ReturnStatement",
                            "range": { "start": 22, "end": 31 },
                            "children": {
                                "argument": {
                                    "kind": "Literal",
                                    "range": { "start": 29, "end": 30 },
                                    "fields": { "value": 1 }
                                }
                            }
                        }]
                    }
                }
            }
        }));
        assert!(run(&node).is_empty());
    }

    // async (): Promise<number> => Promise.resolve(1);  -> ok (body-less thenable arrow)
    #[test]
    fn ignores_async_arrow_returning_promise() {
        let node = from(json!({
            "kind": "ArrowFunctionExpression",
            "range": { "start": 0, "end": 48 },
            "fields": { "async": true, "generator": false },
            "children": {
                "body": {
                    "kind": "CallExpression",
                    "range": { "start": 29, "end": 48 },
                    "children": {
                        "callee": {
                            "kind": "MemberExpression",
                            "range": { "start": 29, "end": 44 },
                            "fields": { "computed": false },
                            "children": {
                                "object": {
                                    "kind": "Identifier",
                                    "range": { "start": 29, "end": 36 },
                                    "fields": { "name": "Promise" }
                                },
                                "property": {
                                    "kind": "Identifier",
                                    "range": { "start": 37, "end": 44 },
                                    "fields": { "name": "resolve" }
                                }
                            }
                        }
                    },
                    "childLists": { "arguments": [] }
                }
            }
        }));
        assert!(run(&node).is_empty());
    }

    // async function foo() { function nested() { await doSomething(); } }  -> report
    // (the await belongs to a nested non-async function, not foo)
    #[test]
    fn reports_when_await_is_inside_nested_function() {
        let node = from(json!({
            "kind": "FunctionDeclaration",
            "range": { "start": 0, "end": 90 },
            "fields": { "async": true, "generator": false },
            "children": {
                "body": {
                    "kind": "BlockStatement",
                    "range": { "start": 20, "end": 90 },
                    "childLists": {
                        "body": [{
                            "kind": "FunctionDeclaration",
                            "range": { "start": 24, "end": 80 },
                            "fields": { "async": false, "generator": false },
                            "children": {
                                "body": {
                                    "kind": "BlockStatement",
                                    "range": { "start": 40, "end": 80 },
                                    "childLists": {
                                        "body": [{
                                            "kind": "ExpressionStatement",
                                            "range": { "start": 44, "end": 62 },
                                            "children": {
                                                "expression": {
                                                    "kind": "AwaitExpression",
                                                    "range": { "start": 44, "end": 61 },
                                                    "children": {
                                                        "argument": {
                                                            "kind": "CallExpression",
                                                            "range": { "start": 50, "end": 61 },
                                                            "children": {
                                                                "callee": {
                                                                    "kind": "Identifier",
                                                                    "range": { "start": 50, "end": 59 },
                                                                    "fields": { "name": "doSomething" }
                                                                }
                                                            },
                                                            "childLists": { "arguments": [] }
                                                        }
                                                    }
                                                }
                                            }
                                        }]
                                    }
                                }
                            }
                        }]
                    }
                }
            }
        }));
        assert_eq!(run(&node), vec!["missingAwait".to_owned()]);
    }

    // async function* test1() { yield* asyncGenerator(); }  -> ok via raw fact
    #[test]
    fn ignores_async_generator_delegating_async_iterable() {
        let node = from(json!({
            "kind": "FunctionDeclaration",
            "range": { "start": 0, "end": 50 },
            "fields": { "async": true, "generator": true },
            "children": {
                "body": {
                    "kind": "BlockStatement",
                    "range": { "start": 24, "end": 50 },
                    "childLists": {
                        "body": [{
                            "kind": "ExpressionStatement",
                            "range": { "start": 28, "end": 48 },
                            "children": {
                                "expression": {
                                    "kind": "YieldExpression",
                                    "range": { "start": 28, "end": 47 },
                                    "fields": { "delegate": true, "__yieldDelegatesAsyncIterable": true },
                                    "children": {
                                        "argument": {
                                            "kind": "CallExpression",
                                            "range": { "start": 35, "end": 47 },
                                            "children": {
                                                "callee": {
                                                    "kind": "Identifier",
                                                    "range": { "start": 35, "end": 45 },
                                                    "fields": { "name": "asyncGenerator" }
                                                }
                                            },
                                            "childLists": { "arguments": [] }
                                        }
                                    }
                                }
                            }
                        }]
                    }
                }
            }
        }));
        assert!(run(&node).is_empty());
    }
}
