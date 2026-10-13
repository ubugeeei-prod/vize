use super::super::{LintFix, LintNode, LintSuggestion, RuleContext, RuleMessage, RustLintRule};
use crate::lint::helpers::{
    is_obviously_promise_like, is_promise_like_node, strip_chain_expression,
};

/// Type-aware rule that enforces consistent use of `return await` based on the
/// configured option and whether the awaited value affects error handling.
///
/// This is a faithful port of the tsgolint `return-await` rule. The decision
/// logic (option semantics, conditional-expression recursion, await/non-await
/// classification, and message/range selection) all live here in Rust. Raw
/// TypeScript checker results arrive as `__camelCase` facts on the relevant
/// returned-expression nodes (see `newFacts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct ReturnAwaitRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "disallowedPromiseAwait",
        description: "Returning an awaited promise is not allowed in this context.",
    },
    RuleMessage {
        id: "disallowedPromiseAwaitSuggestion",
        description: "Remove `await` before the expression. Use caution as this may impact control flow.",
    },
    RuleMessage {
        id: "nonPromiseAwait",
        description: "Returning an awaited value that is not a promise is not allowed.",
    },
    RuleMessage {
        id: "requiredPromiseAwait",
        description: "Returning an awaited promise is required in this context.",
    },
    RuleMessage {
        id: "requiredPromiseAwaitSuggestion",
        description: "Add `await` before the expression. Use caution as this may impact control flow.",
    },
];

// The Go rule listens on every function-like node (to track the async scope) and
// on return statements. In the corsa host model the async-scope tracking is
// resolved host-side: the host only emits the relevant returned-expression facts
// when the owning function is async, so the Rust rule only needs the
// `ReturnStatement` listener plus the arrow-function expression-body listener.
const LISTENERS: &[&str] = &["ReturnStatement", "ArrowFunctionExpression"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReturnAwaitOption {
    Always,
    ErrorHandlingCorrectnessOnly,
    InTryCatch,
    Never,
}

impl ReturnAwaitOption {
    fn parse(value: Option<&str>) -> Self {
        match value {
            Some("always") => Self::Always,
            Some("error-handling-correctness-only") => Self::ErrorHandlingCorrectnessOnly,
            Some("never") => Self::Never,
            // Schema default is "in-try-catch" (also used when option is null/absent).
            _ => Self::InTryCatch,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WhetherToAwait {
    DontCare,
    Await,
    NoAwait,
}

fn get_whether_to_await(affects_error_handling: bool, option: ReturnAwaitOption) -> WhetherToAwait {
    match option {
        ReturnAwaitOption::Always => WhetherToAwait::Await,
        ReturnAwaitOption::ErrorHandlingCorrectnessOnly => {
            if affects_error_handling {
                WhetherToAwait::Await
            } else {
                WhetherToAwait::DontCare
            }
        }
        ReturnAwaitOption::InTryCatch => {
            if affects_error_handling {
                WhetherToAwait::Await
            } else {
                WhetherToAwait::NoAwait
            }
        }
        ReturnAwaitOption::Never => WhetherToAwait::NoAwait,
    }
}

/// Mirror of `utils.TypeAwaitable`. The host computes `NeedsToBeAwaited` and
/// emits it as the `__awaitableCertainty` fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AwaitableCertainty {
    Always,
    May,
    Never,
}

impl AwaitableCertainty {
    fn parse(value: Option<&str>) -> Self {
        match value {
            Some("always") => Self::Always,
            Some("may") => Self::May,
            // Anything else (including the absence of the fact) is treated as a
            // non-thenable value, matching the Go default for non-promise types.
            _ => Self::Never,
        }
    }
}

impl RustLintRule for ReturnAwaitRule {
    fn name(&self) -> &'static str {
        "return-await"
    }

    fn docs_description(&self) -> &'static str {
        "Enforce consistent use of `return await` based on context and configuration."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn has_suggestions(&self) -> bool {
        true
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        let option = ReturnAwaitOption::parse(rule_option_string(node));

        match node.kind.as_str() {
            "ReturnStatement" => {
                // The host only attaches the `__inAsyncScope` fact when the
                // nearest enclosing function is async; bare/non-async returns
                // are skipped, matching `scope.hasAsync` in the Go source.
                if !node.field_bool("__inAsyncScope").unwrap_or(false) {
                    return;
                }
                let Some(argument) = node.child("argument") else {
                    return;
                };
                test_each_possibly_returned_node(
                    ctx,
                    argument,
                    option,
                    node.field_bool("__returnAwaitRequiresAwait")
                        .unwrap_or(false),
                );
            }
            "ArrowFunctionExpression" => {
                // Only concise-body (non-block) async arrow functions return a
                // value implicitly; block bodies are handled via ReturnStatement.
                if !node.field_bool("async").unwrap_or(false) {
                    return;
                }
                let Some(body) = node.child("body") else {
                    return;
                };
                if body.kind == "BlockStatement" {
                    return;
                }
                test_each_possibly_returned_node(ctx, body, option, false);
            }
            _ => {}
        }
    }
}

/// Reads the rule's enum string option from `__ruleOptions[0]`.
fn rule_option_string(node: &LintNode) -> Option<&str> {
    node.fields
        .get("__ruleOptions")?
        .as_array()?
        .first()?
        .as_str()
}

/// Mirror of `testEachPossiblyReturnedNode`: skips parentheses (handled host-side
/// via ESTree, which has no parenthesized-expression nodes by default) and
/// recurses into both branches of a conditional expression.
fn test_each_possibly_returned_node(
    ctx: &mut RuleContext<'_>,
    node: &LintNode,
    option: ReturnAwaitOption,
    affects_error_handling: bool,
) {
    let node = strip_chain_expression(node);
    if node.kind == "ConditionalExpression" {
        if let Some(when_false) = node.child("alternate") {
            test_each_possibly_returned_node(ctx, when_false, option, affects_error_handling);
        }
        if let Some(when_true) = node.child("consequent") {
            test_each_possibly_returned_node(ctx, when_true, option, affects_error_handling);
        }
    } else {
        test(ctx, node, option, affects_error_handling);
    }
}

/// Mirror of the Go `test` closure.
fn test(
    ctx: &mut RuleContext<'_>,
    node: &LintNode,
    option: ReturnAwaitOption,
    affects_error_handling: bool,
) {
    let is_await = node.kind == "AwaitExpression";

    // `child` is the value whose type matters: the awaited operand for an await
    // expression, otherwise the node itself.
    let value_node = if is_await {
        node.child("argument").unwrap_or(node)
    } else {
        node
    };

    let certainty = awaitable_certainty(value_node);

    if certainty != AwaitableCertainty::Always {
        if is_await {
            if certainty == AwaitableCertainty::May {
                return;
            }
            // Definitely-not-a-promise awaited value: remove the await.
            ctx.report_with_suggestions(
                "nonPromiseAwait",
                node.range,
                vec![remove_await_suggestion(node, "nonPromiseAwait")],
            );
        }
        return;
    }

    // At this point the value is definitely a thenable.
    let affects_error_handling = node
        .field_bool("__affectsErrorHandling")
        .unwrap_or(affects_error_handling);
    let use_auto_fix = !affects_error_handling;

    match get_whether_to_await(affects_error_handling, option) {
        WhetherToAwait::Await => {
            if is_await {
                return;
            }
            report_required(ctx, node, use_auto_fix);
        }
        WhetherToAwait::DontCare => {}
        WhetherToAwait::NoAwait => {
            if !is_await {
                return;
            }
            report_disallowed(ctx, node, use_auto_fix);
        }
    }
}

fn awaitable_certainty(node: &LintNode) -> AwaitableCertainty {
    if let Some(certainty) = node.field_str("__awaitableCertainty") {
        return AwaitableCertainty::parse(Some(certainty));
    }
    if is_obviously_promise_like(node) || is_promise_like_node(node) {
        return AwaitableCertainty::Always;
    }
    if node.type_texts.is_empty() && node.property_names.is_empty() {
        return AwaitableCertainty::May;
    }
    AwaitableCertainty::Never
}

/// Emits the `requiredPromiseAwait` diagnostic, inserting `await` before `node`.
///
/// In the Go source `use_auto_fix` selects between an auto-applied fix (when the
/// expression does not affect error handling) and a manual suggestion. The corsa
/// diagnostic model carries only suggestions, so the same fixes are attached as a
/// suggestion in both cases; `use_auto_fix` is retained for parity bookkeeping.
fn report_required(ctx: &mut RuleContext<'_>, node: &LintNode, use_auto_fix: bool) {
    let _ = use_auto_fix;
    let high_precedence = node
        .field_bool("__isHigherPrecedenceThanAwait")
        .unwrap_or(false);
    let fixes = insert_await_fixes(node, high_precedence);
    ctx.report_with_suggestions(
        "requiredPromiseAwait",
        node.range,
        vec![LintSuggestion {
            message_id: "requiredPromiseAwaitSuggestion".to_owned(),
            message: message_for("requiredPromiseAwaitSuggestion"),
            fixes,
        }],
    );
}

/// Emits the `disallowedPromiseAwait` diagnostic, removing the `await` keyword.
fn report_disallowed(ctx: &mut RuleContext<'_>, node: &LintNode, use_auto_fix: bool) {
    let _ = use_auto_fix;
    ctx.report_with_suggestions(
        "disallowedPromiseAwait",
        node.range,
        vec![remove_await_suggestion(
            node,
            "disallowedPromiseAwaitSuggestion",
        )],
    );
}

/// Builds a suggestion that removes the `await` keyword token at the start of
/// the await expression.
///
/// Mirrors the Go `removeAwaitFix`, which removes only the single `await` keyword
/// token (`scanner.GetRangeOfTokenAtPosition` at the await expression's start),
/// not the whitespace/comments between the keyword and its operand. The keyword
/// is always exactly the 5 bytes `await` at the start of an `AwaitExpression`, so
/// this is a pure positional computation needing no host fact. Removing through
/// to the operand start would collapse the inter-token spacing (e.g. `=>  1` vs
/// `=> 1`) and, worse, delete any comments between `await` and the operand
/// (`await /* c */ 1`), diverging from upstream fixture output.
fn remove_await_suggestion(await_node: &LintNode, message_id: &'static str) -> LintSuggestion {
    const AWAIT_KEYWORD_LEN: u32 = 5;
    let keyword_end = await_node
        .range
        .start
        .saturating_add(AWAIT_KEYWORD_LEN)
        .min(await_node.range.end);
    let remove_range = super::super::TextRange::new(await_node.range.start, keyword_end);
    LintSuggestion {
        message_id: message_id.to_owned(),
        message: message_for(message_id),
        fixes: vec![LintFix::remove_range(remove_range)],
    }
}

/// Builds the fixes that insert `await` before `node`. When the node is not of
/// higher precedence than `await`, it must be parenthesized.
fn insert_await_fixes(node: &LintNode, high_precedence: bool) -> Vec<LintFix> {
    let start = super::super::TextRange::new(node.range.start, node.range.start);
    if high_precedence {
        vec![LintFix::replace_range(start, "await ")]
    } else {
        let end = super::super::TextRange::new(node.range.end, node.range.end);
        vec![
            LintFix::replace_range(start, "await ("),
            LintFix::replace_range(end, ")"),
        ]
    }
}

fn message_for(message_id: &str) -> String {
    MESSAGES
        .iter()
        .find(|message| message.id == message_id)
        .map(|message| message.description)
        .unwrap_or(message_id)
        .to_owned()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::super::{LintNode, RuleContext, RustLintRule};
    use super::ReturnAwaitRule;

    fn run(node: &LintNode) -> Vec<super::super::super::LintDiagnostic> {
        let rule = ReturnAwaitRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, node);
        ctx.finish()
    }

    /// Builds a return statement whose argument is the given JSON node, in an
    /// async scope, with the given rule option string (or none for default).
    fn return_stmt(argument: serde_json::Value, option: Option<&str>) -> LintNode {
        let mut fields = json!({ "__inAsyncScope": true });
        if let Some(option) = option {
            fields["__ruleOptions"] = json!([option]);
        }
        serde_json::from_value(json!({
            "kind": "ReturnStatement",
            "range": { "start": 0, "end": 40 },
            "fields": fields,
            "children": { "argument": argument },
        }))
        .unwrap()
    }

    fn promise_value(start: u32, end: u32) -> serde_json::Value {
        json!({
            "kind": "CallExpression",
            "range": { "start": start, "end": end },
            "fields": { "__awaitableCertainty": "always", "__isHigherPrecedenceThanAwait": true },
        })
    }

    fn promise_resolve_value(start: u32, end: u32) -> serde_json::Value {
        json!({
            "kind": "CallExpression",
            "range": { "start": start, "end": end },
            "fields": { "__isHigherPrecedenceThanAwait": true },
            "children": {
                "callee": {
                    "kind": "MemberExpression",
                    "range": { "start": start, "end": start + 15 },
                    "children": {
                        "object": {
                            "kind": "Identifier",
                            "range": { "start": start, "end": start + 7 },
                            "fields": { "name": "Promise" }
                        },
                        "property": {
                            "kind": "Identifier",
                            "range": { "start": start + 8, "end": start + 15 },
                            "fields": { "name": "resolve" }
                        }
                    }
                }
            }
        })
    }

    fn await_promise(start: u32, end: u32, affects_error_handling: bool) -> serde_json::Value {
        json!({
            "kind": "AwaitExpression",
            "range": { "start": start, "end": end },
            "fields": { "__affectsErrorHandling": affects_error_handling },
            "children": {
                "argument": {
                    "kind": "CallExpression",
                    "range": { "start": start + 6, "end": end },
                    "fields": { "__awaitableCertainty": "always" },
                },
            },
        })
    }

    #[test]
    fn reports_disallowed_await_with_never_option() {
        // `return await Promise.resolve(1)` with option "never" => disallowed.
        let node = return_stmt(await_promise(7, 33, false), Some("never"));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "disallowedPromiseAwait");
        assert_eq!(
            diagnostics[0].suggestions[0].message_id,
            "disallowedPromiseAwaitSuggestion"
        );
        // The removal fix deletes only the 5-byte `await` keyword token, not the
        // whitespace through to the operand (upstream `=>  Promise...` parity).
        let fix = &diagnostics[0].suggestions[0].fixes[0];
        assert_eq!(fix.range, super::super::super::TextRange::new(7, 12));
        assert_eq!(fix.replacement_text, "");
    }

    #[test]
    fn reports_non_promise_await() {
        // `return await 1` => nonPromiseAwait (certainty never on the operand).
        // The operand is far from the keyword to exercise the keyword-only fix
        // range (mirrors `await /* comment */ 1` where the comment must survive).
        let await_node = json!({
            "kind": "AwaitExpression",
            "range": { "start": 7, "end": 25 },
            "fields": {},
            "children": {
                "argument": {
                    "kind": "Literal",
                    "range": { "start": 24, "end": 25 },
                    "fields": { "__awaitableCertainty": "never", "value": 1 },
                },
            },
        });
        let node = return_stmt(await_node, None);
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "nonPromiseAwait");
        // Only `await` (bytes 7..12) is removed; the gap (e.g. a comment) at
        // 12..24 is preserved, matching Go's GetRangeOfTokenAtPosition.
        let fix = &diagnostics[0].suggestions[0].fixes[0];
        assert_eq!(fix.range, super::super::super::TextRange::new(7, 12));
    }

    #[test]
    fn reports_required_await_in_try_catch() {
        // `return Promise.resolve(1)` inside try/catch (default in-try-catch,
        // affectsErrorHandling) => requiredPromiseAwait.
        let mut promise = promise_value(7, 33);
        promise["fields"]["__affectsErrorHandling"] = json!(true);
        let node = return_stmt(promise, None);
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "requiredPromiseAwait");
        assert_eq!(
            diagnostics[0].suggestions[0].message_id,
            "requiredPromiseAwaitSuggestion"
        );
    }

    #[test]
    fn uses_return_statement_error_handling_context() {
        // Host facts attach try/catch context to the return statement; the Rust
        // rule carries that context to the returned promise expression.
        let mut node = return_stmt(promise_value(7, 33), None);
        node.fields
            .insert("__returnAwaitRequiresAwait".to_owned(), json!(true));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "requiredPromiseAwait");
    }

    #[test]
    fn allows_bare_promise_in_default_outside_try() {
        // `return Promise.resolve(1)` at top level of async fn, default option,
        // not in try/catch => NoAwait but it isn't an await, so no report.
        let node = return_stmt(promise_value(7, 33), None);
        let diagnostics = run(&node);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn treats_obvious_promise_call_as_awaitable_without_checker_fact() {
        let await_node = json!({
            "kind": "AwaitExpression",
            "range": { "start": 7, "end": 33 },
            "children": {
                "argument": promise_resolve_value(13, 33),
            },
        });
        let node = return_stmt(await_node, None);
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "disallowedPromiseAwait");
    }

    #[test]
    fn allows_await_in_try_catch_default() {
        // `return await Promise.resolve(1)` inside try/catch, default option =>
        // affectsErrorHandling so await is required; it already awaits => no report.
        let node = return_stmt(await_promise(7, 33, true), None);
        let diagnostics = run(&node);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn skips_non_async_return() {
        let mut node = return_stmt(await_promise(7, 14, false), None);
        node.fields
            .insert("__inAsyncScope".to_owned(), json!(false));
        let diagnostics = run(&node);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn recurses_into_conditional_branches_with_always() {
        // `return cond ? bar() : baz()` with option "always" => both branches
        // are bare promises that require await => two reports.
        let conditional = json!({
            "kind": "ConditionalExpression",
            "range": { "start": 7, "end": 30 },
            "fields": {},
            "children": {
                "test": { "kind": "Identifier", "range": { "start": 7, "end": 11 }, "fields": { "name": "cond" } },
                "consequent": promise_value(14, 19),
                "alternate": promise_value(22, 27),
            },
        });
        let node = return_stmt(conditional, Some("always"));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 2);
        assert!(
            diagnostics
                .iter()
                .all(|d| d.message_id == "requiredPromiseAwait")
        );
    }
}
