use super::super::{
    LintFix, LintNode, LintSuggestion, RuleContext, RuleMessage, RustLintRule, TextRange,
};
use crate::lint::helpers::{
    child_list, is_identifier_named, member_object, member_property_name, rule_allow_list_names,
    rule_option_bool, strip_chain_expression,
};
use crate::utils::split_top_level_type_text;

/// Type-aware rule that requires promises to be awaited or explicitly handled.
///
/// Faithful port of the upstream `no-floating-promises` decision flow,
/// including the `ignoreVoid`, `ignoreIIFE`, `checkThenables`,
/// `allowForKnownSafeCalls`, and `allowForKnownSafePromises` options and the
/// upstream message catalog. `TypeOrValueSpecifier` entries are matched by
/// name; the `from` source domain is not derivable from host facts, so a name
/// match is honoured for every domain (erring toward silence).
#[derive(Clone, Copy, Debug, Default)]
pub struct NoFloatingPromisesRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "floating",
        description: "Promises must be awaited, end with a call to .catch, or end with a call to .then with a rejection handler.",
    },
    RuleMessage {
        id: "floatingVoid",
        description: "Promises must be awaited, end with a call to .catch, end with a call to .then with a rejection handler or be explicitly marked as ignored with the `void` operator.",
    },
    RuleMessage {
        id: "floatingUselessRejectionHandler",
        description: "Promises must be awaited, end with a call to .catch, or end with a call to .then with a rejection handler. A rejection handler that is not a function will be ignored.",
    },
    RuleMessage {
        id: "floatingUselessRejectionHandlerVoid",
        description: "Promises must be awaited, end with a call to .catch, end with a call to .then with a rejection handler or be explicitly marked as ignored with the `void` operator. A rejection handler that is not a function will be ignored.",
    },
    RuleMessage {
        id: "floatingPromiseArray",
        description: "An array of Promises may be unintentional. Consider handling the promises' fulfillment or rejection with Promise.all or similar.",
    },
    RuleMessage {
        id: "floatingPromiseArrayVoid",
        description: "An array of Promises may be unintentional. Consider handling the promises' fulfillment or rejection with Promise.all or similar, or explicitly marking the expression as ignored with the `void` operator.",
    },
    RuleMessage {
        id: "floatingFixAwait",
        description: "Await promise.",
    },
    RuleMessage {
        id: "floatingFixVoid",
        description: "Add void operator to ignore.",
    },
];
const LISTENERS: &[&str] = &["ExpressionStatement"];

#[derive(Clone, Debug)]
struct Options {
    ignore_void: bool,
    ignore_iife: bool,
    check_thenables: bool,
    allow_for_known_safe_calls: Vec<String>,
    allow_for_known_safe_promises: Vec<String>,
}

impl Options {
    fn from_node(node: &LintNode) -> Self {
        Self {
            ignore_void: rule_option_bool(node, "ignoreVoid").unwrap_or(true),
            ignore_iife: rule_option_bool(node, "ignoreIIFE").unwrap_or(false),
            check_thenables: rule_option_bool(node, "checkThenables").unwrap_or(false),
            allow_for_known_safe_calls: rule_allow_list_names(node, "allowForKnownSafeCalls"),
            allow_for_known_safe_promises: rule_allow_list_names(node, "allowForKnownSafePromises"),
        }
    }
}

struct UnhandledPromise {
    range: TextRange,
    promise_array: bool,
    non_function_handler: bool,
}

impl RustLintRule for NoFloatingPromisesRule {
    fn name(&self) -> &'static str {
        "no-floating-promises"
    }

    fn docs_description(&self) -> &'static str {
        "Require promises to be awaited or otherwise handled."
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
        if node.kind != "ExpressionStatement" {
            return;
        }
        let options = Options::from_node(node);
        let Some(expression) = node.child("expression").map(strip_chain_expression) else {
            return;
        };
        if options.ignore_iife && is_async_iife(expression) {
            return;
        }
        if is_known_safe_call(expression, &options) {
            return;
        }
        let Some(unhandled) = is_unhandled_promise(expression, &options) else {
            return;
        };

        if unhandled.promise_array {
            let message_id = if options.ignore_void {
                "floatingPromiseArrayVoid"
            } else {
                "floatingPromiseArray"
            };
            ctx.report(message_id, unhandled.range);
            return;
        }

        if options.ignore_void {
            let message_id = if unhandled.non_function_handler {
                "floatingUselessRejectionHandlerVoid"
            } else {
                "floatingVoid"
            };
            ctx.report_with_suggestions(
                message_id,
                unhandled.range,
                vec![
                    void_suggestion(expression),
                    await_suggestion(node, expression),
                ],
            );
        } else {
            let message_id = if unhandled.non_function_handler {
                "floatingUselessRejectionHandler"
            } else {
                "floating"
            };
            ctx.report_with_suggestions(
                message_id,
                unhandled.range,
                vec![await_suggestion(node, expression)],
            );
        }
    }
}

fn is_unhandled_promise(node: &LintNode, options: &Options) -> Option<UnhandledPromise> {
    let node = strip_chain_expression(node);

    if node.kind == "AssignmentExpression" {
        return None;
    }

    // A comma expression can hide an unhandled promise in any operand, so all
    // of them are checked regardless of the final value's type.
    if node.kind == "SequenceExpression" {
        return child_list(node, "expressions")
            .iter()
            .find_map(|part| is_unhandled_promise(part, options));
    }

    // A `void` expression always evaluates to `undefined`, so with
    // `ignoreVoid: false` the wrapped expression is inspected directly.
    if !options.ignore_void
        && node.kind == "UnaryExpression"
        && node.field_str("operator") == Some("void")
    {
        return node
            .child("argument")
            .and_then(|argument| is_unhandled_promise(argument, options));
    }

    if is_promise_array(node, options) {
        return Some(UnhandledPromise {
            range: node.range,
            promise_array: true,
            non_function_handler: false,
        });
    }

    // `await` addresses promises (but not promise arrays, handled above).
    if node.kind == "AwaitExpression" {
        return None;
    }

    if !is_promise_like(node, options) {
        return None;
    }

    if node.kind == "CallExpression" {
        let callee = node.child("callee").map(strip_chain_expression);
        if let Some(callee) = callee
            && callee.kind == "MemberExpression"
            && let Some(method_name) = member_property_name(callee)
        {
            let arguments = child_list(node, "arguments");
            match method_name.as_str() {
                "catch" if !arguments.is_empty() => {
                    if has_spread_through(arguments, 0) {
                        return Some(unhandled_plain(node));
                    }
                    return if is_valid_rejection_handler(&arguments[0]) {
                        None
                    } else {
                        Some(UnhandledPromise {
                            range: node.range,
                            promise_array: false,
                            non_function_handler: true,
                        })
                    };
                }
                "then" if arguments.len() >= 2 => {
                    if has_spread_through(arguments, 1) {
                        return Some(unhandled_plain(node));
                    }
                    return if is_valid_rejection_handler(&arguments[1]) {
                        None
                    } else {
                        Some(UnhandledPromise {
                            range: node.range,
                            promise_array: false,
                            non_function_handler: true,
                        })
                    };
                }
                // `x.finally()` is transparent to promise resolution: check `x`.
                "finally" => {
                    let mut unhandled =
                        member_object(callee).and_then(|obj| is_unhandled_promise(obj, options))?;
                    unhandled.range = node.range;
                    return Some(unhandled);
                }
                _ => {}
            }
        }
        return Some(unhandled_plain(node));
    }

    if node.kind == "ConditionalExpression" {
        return node
            .child("alternate")
            .and_then(|branch| is_unhandled_promise(branch, options))
            .or_else(|| {
                node.child("consequent")
                    .and_then(|branch| is_unhandled_promise(branch, options))
            });
    }

    if node.kind == "LogicalExpression" {
        return node
            .child("left")
            .and_then(|operand| is_unhandled_promise(operand, options))
            .or_else(|| {
                node.child("right")
                    .and_then(|operand| is_unhandled_promise(operand, options))
            });
    }

    Some(unhandled_plain(node))
}

fn unhandled_plain(node: &LintNode) -> UnhandledPromise {
    UnhandledPromise {
        range: node.range,
        promise_array: false,
        non_function_handler: false,
    }
}

/// Whether the first `index + 1` arguments contain a spread element, which
/// makes the positional rejection handler unknowable.
fn has_spread_through(arguments: &[LintNode], index: usize) -> bool {
    arguments
        .iter()
        .take(index + 1)
        .any(|argument| argument.kind == "SpreadElement")
}

/// Mirrors the upstream callable check on rejection handlers, degrading
/// conservatively: a handler is treated as valid unless the facts prove it
/// cannot be called.
fn is_valid_rejection_handler(handler: &LintNode) -> bool {
    let handler = strip_chain_expression(handler);
    if handler.kind.contains("Function") {
        return true;
    }
    if handler.type_texts.is_empty() {
        // No type facts: prefer silence over a false positive.
        return true;
    }
    handler.type_texts.iter().any(|text| {
        split_top_level_type_text(text, '|')
            .iter()
            .any(|part| is_callable_type_text(part.trim()))
    })
}

fn is_callable_type_text(text: &str) -> bool {
    text.contains("=>") || text.starts_with("new ") || text == "Function"
}

fn is_promise_like(node: &LintNode, options: &Options) -> bool {
    if type_matches_names(node, &options.allow_for_known_safe_promises) {
        return false;
    }
    if node
        .type_texts
        .iter()
        .any(|text| any_union_part(text, is_promise_atom))
    {
        return true;
    }
    if is_obviously_promise_producing(node) {
        return true;
    }
    if !options.check_thenables {
        return false;
    }
    node.property_names.iter().any(|name| name == "then")
}

fn is_obviously_promise_producing(node: &LintNode) -> bool {
    let current = strip_chain_expression(node);
    if current.kind == "NewExpression" {
        return current
            .child("callee")
            .is_some_and(|callee| is_identifier_named(callee, "Promise"));
    }
    if current.kind != "CallExpression" {
        return false;
    }
    let Some(callee) = current.child("callee").map(strip_chain_expression) else {
        return false;
    };
    // An immediately-invoked async function literal produces a promise by
    // construction; anonymous function literals have no position-addressable
    // checker type, so this is exact rather than a heuristic.
    if (callee.kind == "ArrowFunctionExpression" || callee.kind == "FunctionExpression")
        && callee.field_bool("async") == Some(true)
    {
        return true;
    }
    member_property_name(callee).as_deref() == Some("resolve")
        && member_object(callee).is_some_and(|object| is_identifier_named(object, "Promise"))
}

fn is_promise_array(node: &LintNode, options: &Options) -> bool {
    node.type_texts.iter().any(|text| {
        any_union_part(text, |part| {
            promise_array_element(part)
                .is_some_and(|element| any_union_part(element, is_promise_atom))
                && !name_of_type_part(part).is_some_and(|name| {
                    options
                        .allow_for_known_safe_promises
                        .iter()
                        .any(|allowed| allowed == name)
                })
        })
    })
}

/// Returns the element type text when `part` denotes an array or tuple.
fn promise_array_element(part: &str) -> Option<&str> {
    let part = part.trim();
    if let Some(element) = part.strip_suffix("[]") {
        return Some(element.trim_start_matches('(').trim_end_matches(')'));
    }
    for wrapper in ["Array<", "ReadonlyArray<", "readonly "] {
        if let Some(rest) = part.strip_prefix(wrapper) {
            return Some(
                rest.strip_suffix('>')
                    .unwrap_or(rest)
                    .trim_end_matches("[]"),
            );
        }
    }
    if part.starts_with('[') && part.ends_with(']') {
        // Tuple: report when any element is promise-like.
        return Some(&part[1..part.len() - 1]);
    }
    None
}

fn any_union_part(text: &str, predicate: impl Fn(&str) -> bool) -> bool {
    split_top_level_type_text(text, '|')
        .iter()
        .any(|part| predicate(part.trim()))
}

fn is_promise_atom(text: &str) -> bool {
    matches!(text, "Promise" | "PromiseLike")
        || text.starts_with("Promise<")
        || text.starts_with("PromiseLike<")
}

fn type_matches_names(node: &LintNode, names: &[String]) -> bool {
    if names.is_empty() {
        return false;
    }
    node.type_texts.iter().any(|text| {
        any_union_part(text, |part| {
            name_of_type_part(part).is_some_and(|name| names.iter().any(|allowed| allowed == name))
        })
    })
}

/// Returns the bare reference name of a type part (`SafePromise<number>` →
/// `SafePromise`).
fn name_of_type_part(part: &str) -> Option<&str> {
    let part = part.trim();
    let end = part.find('<').unwrap_or(part.len());
    let name = &part[..end];
    (!name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_alphanumeric() || ch == '_' || ch == '$' || ch == '.'))
    .then_some(name)
}

fn is_known_safe_call(expression: &LintNode, options: &Options) -> bool {
    if options.allow_for_known_safe_calls.is_empty() || expression.kind != "CallExpression" {
        return false;
    }
    let Some(callee) = expression.child("callee").map(strip_chain_expression) else {
        return false;
    };
    let callee_name =
        crate::lint::helpers::identifier_name(callee).or_else(|| member_property_name(callee));
    if callee_name.is_some_and(|name| options.allow_for_known_safe_calls.contains(&name)) {
        return true;
    }
    type_matches_names(callee, &options.allow_for_known_safe_calls)
}

fn is_async_iife(expression: &LintNode) -> bool {
    let expression = strip_chain_expression(expression);
    if expression.kind != "CallExpression" {
        return false;
    }
    expression
        .child("callee")
        .map(strip_chain_expression)
        .is_some_and(|callee| {
            callee.kind == "ArrowFunctionExpression" || callee.kind == "FunctionExpression"
        })
}

fn void_suggestion(expression: &LintNode) -> LintSuggestion {
    let insertion = TextRange::new(expression.range.start, expression.range.start);
    LintSuggestion {
        message_id: "floatingFixVoid".to_owned(),
        message: "Add void operator to ignore.".to_owned(),
        fixes: vec![LintFix::replace_range(insertion, "void ")],
    }
}

fn await_suggestion(_node: &LintNode, expression: &LintNode) -> LintSuggestion {
    // Replacing the `void` keyword with `await` mirrors the upstream fix for
    // `void somePromise;` under `ignoreVoid: false`.
    if expression.kind == "UnaryExpression" && expression.field_str("operator") == Some("void") {
        let keyword = TextRange::new(expression.range.start, expression.range.start + 4);
        return LintSuggestion {
            message_id: "floatingFixAwait".to_owned(),
            message: "Await promise.".to_owned(),
            fixes: vec![LintFix::replace_range(keyword, "await")],
        };
    }
    let insertion = TextRange::new(expression.range.start, expression.range.start);
    LintSuggestion {
        message_id: "floatingFixAwait".to_owned(),
        message: "Await promise.".to_owned(),
        fixes: vec![LintFix::replace_range(insertion, "await ")],
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::super::{LintNode, RuleContext, TextRange};
    use super::NoFloatingPromisesRule;
    use crate::lint::RustLintRule;

    fn run(node: &LintNode) -> Vec<(String, TextRange, Vec<String>)> {
        let rule = NoFloatingPromisesRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, node);
        ctx.finish()
            .into_iter()
            .map(|diagnostic| {
                (
                    diagnostic.message_id,
                    diagnostic.range,
                    diagnostic
                        .suggestions
                        .into_iter()
                        .map(|suggestion| suggestion.message_id)
                        .collect(),
                )
            })
            .collect()
    }

    fn node(value: serde_json::Value) -> LintNode {
        serde_json::from_value(value).expect("valid LintNode")
    }

    fn statement(expression: serde_json::Value, options: Option<serde_json::Value>) -> LintNode {
        let mut fields = json!({});
        if let Some(options) = options {
            fields = json!({ "__ruleOptions": [options] });
        }
        node(json!({
            "kind": "ExpressionStatement",
            "range": { "start": 0, "end": 40 },
            "fields": fields,
            "children": { "expression": expression }
        }))
    }

    fn promise_identifier() -> serde_json::Value {
        json!({
            "kind": "Identifier",
            "range": { "start": 0, "end": 9 },
            "typeTexts": ["Promise<void>"],
            "fields": { "name": "promise" }
        })
    }

    #[test]
    fn reports_floating_promise_identifier_with_void_message() {
        let diagnostics = run(&statement(promise_identifier(), None));
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].0, "floatingVoid");
        assert_eq!(diagnostics[0].1, TextRange::new(0, 9));
        assert_eq!(
            diagnostics[0].2,
            vec!["floatingFixVoid", "floatingFixAwait"]
        );
    }

    #[test]
    fn ignore_void_false_reports_plain_floating_with_await_fix_only() {
        let diagnostics = run(&statement(
            promise_identifier(),
            Some(json!({ "ignoreVoid": false })),
        ));
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].0, "floating");
        assert_eq!(diagnostics[0].2, vec!["floatingFixAwait"]);
    }

    #[test]
    fn ignore_void_false_looks_inside_void_expressions() {
        let void_wrapped = json!({
            "kind": "UnaryExpression",
            "range": { "start": 0, "end": 14 },
            "fields": { "operator": "void" },
            "children": { "argument": promise_identifier() }
        });
        assert!(run(&statement(void_wrapped.clone(), None)).is_empty());
        let diagnostics = run(&statement(
            void_wrapped,
            Some(json!({ "ignoreVoid": false })),
        ));
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].0, "floating");
    }

    #[test]
    fn ignore_iife_skips_async_iife() {
        let iife = json!({
            "kind": "CallExpression",
            "range": { "start": 0, "end": 20 },
            "typeTexts": ["Promise<void>"],
            "children": {
                "callee": {
                    "kind": "ArrowFunctionExpression",
                    "range": { "start": 1, "end": 16 },
                    "fields": { "async": true }
                }
            }
        });
        assert_eq!(run(&statement(iife.clone(), None)).len(), 1);
        assert!(run(&statement(iife, Some(json!({ "ignoreIIFE": true })))).is_empty());
    }

    #[test]
    fn thenables_require_check_thenables_option() {
        let thenable = json!({
            "kind": "Identifier",
            "range": { "start": 0, "end": 8 },
            "typeTexts": ["MyThenable"],
            "propertyNames": ["then"],
            "fields": { "name": "thenable" }
        });
        assert!(run(&statement(thenable.clone(), None)).is_empty());
        let diagnostics = run(&statement(
            thenable,
            Some(json!({ "checkThenables": true })),
        ));
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].0, "floatingVoid");
    }

    #[test]
    fn allow_for_known_safe_promises_matches_type_name() {
        let safe = json!({
            "kind": "Identifier",
            "range": { "start": 0, "end": 9 },
            "typeTexts": ["SafePromise<number>"],
            "propertyNames": ["then"],
            "fields": { "name": "promise" }
        });
        let options = json!({
            "checkThenables": true,
            "allowForKnownSafePromises": [{ "from": "file", "name": "SafePromise" }]
        });
        assert!(run(&statement(safe, Some(options))).is_empty());
    }

    #[test]
    fn allow_for_known_safe_calls_matches_callee_name() {
        let call = json!({
            "kind": "CallExpression",
            "range": { "start": 0, "end": 14 },
            "typeTexts": ["Promise<void>"],
            "children": {
                "callee": {
                    "kind": "Identifier",
                    "range": { "start": 0, "end": 12 },
                    "fields": { "name": "safeAsyncFn" }
                }
            }
        });
        assert_eq!(run(&statement(call.clone(), None)).len(), 1);
        let options = json!({ "allowForKnownSafeCalls": ["safeAsyncFn"] });
        assert!(run(&statement(call, Some(options))).is_empty());
    }

    #[test]
    fn reports_promise_array_without_suggestions() {
        let array = json!({
            "kind": "Identifier",
            "range": { "start": 0, "end": 8 },
            "typeTexts": ["Promise<string>[]"],
            "fields": { "name": "batch" }
        });
        let diagnostics = run(&statement(array.clone(), None));
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].0, "floatingPromiseArrayVoid");
        assert!(diagnostics[0].2.is_empty());

        let diagnostics = run(&statement(array, Some(json!({ "ignoreVoid": false }))));
        assert_eq!(diagnostics[0].0, "floatingPromiseArray");
    }

    #[test]
    fn reports_useless_rejection_handler_on_non_callable_catch_argument() {
        let handled = json!({
            "kind": "CallExpression",
            "range": { "start": 0, "end": 30 },
            "typeTexts": ["Promise<void>"],
            "children": {
                "callee": {
                    "kind": "MemberExpression",
                    "range": { "start": 0, "end": 22 },
                    "children": {
                        "object": promise_identifier(),
                        "property": {
                            "kind": "Identifier",
                            "range": { "start": 17, "end": 22 },
                            "fields": { "name": "catch" }
                        }
                    }
                }
            },
            "childLists": {
                "arguments": [{
                    "kind": "Identifier",
                    "range": { "start": 23, "end": 28 },
                    "typeTexts": ["null"],
                    "fields": { "name": "oops" }
                }]
            }
        });
        let diagnostics = run(&statement(handled, None));
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].0, "floatingUselessRejectionHandlerVoid");
    }

    #[test]
    fn callable_catch_argument_handles_the_promise() {
        let handled = json!({
            "kind": "CallExpression",
            "range": { "start": 0, "end": 30 },
            "typeTexts": ["Promise<void>"],
            "children": {
                "callee": {
                    "kind": "MemberExpression",
                    "range": { "start": 0, "end": 22 },
                    "children": {
                        "object": promise_identifier(),
                        "property": {
                            "kind": "Identifier",
                            "range": { "start": 17, "end": 22 },
                            "fields": { "name": "catch" }
                        }
                    }
                }
            },
            "childLists": {
                "arguments": [{
                    "kind": "Identifier",
                    "range": { "start": 23, "end": 28 },
                    "typeTexts": ["(reason: unknown) => void"],
                    "fields": { "name": "onError" }
                }]
            }
        });
        assert!(run(&statement(handled, None)).is_empty());
    }

    #[test]
    fn sequence_expression_checks_every_operand() {
        let sequence = json!({
            "kind": "SequenceExpression",
            "range": { "start": 0, "end": 20 },
            "childLists": {
                "expressions": [
                    { "kind": "Identifier", "range": { "start": 0, "end": 1 }, "typeTexts": ["number"] },
                    promise_identifier()
                ]
            }
        });
        let diagnostics = run(&statement(sequence, None));
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].1, TextRange::new(0, 9));
    }

    #[test]
    fn await_expression_is_handled() {
        let awaited = json!({
            "kind": "AwaitExpression",
            "range": { "start": 0, "end": 15 },
            "typeTexts": ["Promise<void>"],
            "children": { "argument": promise_identifier() }
        });
        assert!(run(&statement(awaited, None)).is_empty());
    }
}
