use serde_json::Value;

use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule, TextRange};
use crate::lint::helpers::child_list;
use crate::utils::{TypeTextKind, classify_type_text, split_top_level_type_text};

/// Message catalog for `strict-void-return`, mirroring the tsgolint Go rule's
/// three message IDs verbatim.
const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "asyncFunc",
        description: "Async function used in a context where a void function is expected.",
    },
    RuleMessage {
        id: "nonVoidFunc",
        description: "Value-returning function used in a context where a void function is expected.",
    },
    RuleMessage {
        id: "nonVoidReturn",
        description: "Value returned in a context where a void return is expected.",
    },
];

/// Faithful native port of the tsgolint `strict-void-return` rule.
///
/// The host adapter is responsible for the type-driven *positional* facts: for
/// every node that the Go rule passes to `reportIfNonVoidFunction`/
/// `checkExpressionNode`, the host marks the value node with
/// `__voidReturnExpected = true` (the raw result of `isVoidReturningFunctionType`
/// applied to the contextual/parameter/member type) and supplies
/// `__functionApparentReturnTypeTexts` (the rendered return-type texts of the
/// value's own apparent call signatures). All *verdict* logic — deciding async
/// vs. value-returning vs. concise-body vs. per-`return` — stays in Rust here.
#[derive(Clone, Copy, Debug, Default)]
pub struct StrictVoidReturnRule;

impl RustLintRule for StrictVoidReturnRule {
    fn name(&self) -> &'static str {
        "strict-void-return"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow returning a value from a function in a context expecting a void return."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        // Mirrors the Go RuleListeners map keys (mapped to TS-ESTree kinds).
        &[
            "ArrayExpression",
            "ArrowFunctionExpression",
            "AssignmentExpression",
            "CallExpression",
            "NewExpression",
            "JSXAttribute",
            "MethodDefinition",
            "Property",
            "PropertyDefinition",
            "ReturnStatement",
            "VariableDeclaration",
        ]
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        check_strict_void_return(ctx, node);
    }
}

/// `allowReturnAny` option, read from `__ruleOptions[0].allowReturnAny`.
fn allow_return_any(node: &LintNode) -> bool {
    node.fields
        .get("__ruleOptions")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(|opts| opts.get("allowReturnAny"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

/// Mirrors `isAllowedType`: every union part must be void/never/undefined (or
/// any when `allowReturnAny`).
///
/// `empty_is_allowed` controls the treatment of an empty `type_texts` slice:
///   * For the apparent-call-signature short-circuit (`reportIfNonVoidFunction`),
///     Go evaluates `utils.Every(GetCallSignatures(actualType), ...)`, which is
///     vacuously `true` when the value has *no* call signatures. That path must
///     pass `empty_is_allowed = true` so a non-callable value silently returns
///     instead of being reported as `nonVoidFunc`.
///   * For the per-`return` argument check, Go always has a concrete type
///     (`UnionTypeParts` is never empty), so an empty/missing host type-text set
///     means "unknown" and must be treated as *not* allowed (the conservative
///     `empty_is_allowed = false`).
fn is_allowed_return_type_texts(
    type_texts: &[String],
    allow_any: bool,
    empty_is_allowed: bool,
) -> bool {
    if type_texts.is_empty() {
        return empty_is_allowed;
    }
    type_texts.iter().all(|text| {
        split_top_level_type_text(text, '|').iter().all(|part| {
            let part = part.trim();
            matches!(part, "void" | "never" | "undefined")
                || (allow_any
                    && (part == "any" || classify_type_text(Some(part)) == TypeTextKind::Any))
        })
    })
}

/// True when the host marked this node as sitting in a void-returning-function
/// position (`isVoidReturningFunctionType(expectedType)` upstream).
fn void_return_expected(node: &LintNode) -> bool {
    node.field_bool("__voidReturnExpected").unwrap_or(false)
}

/// Rendered return-type texts of the value's own apparent call signatures,
/// used for the early `isAllowedType` short-circuit in `reportIfNonVoidFunction`.
fn apparent_return_type_texts(node: &LintNode) -> Vec<String> {
    node.fields
        .get("__functionApparentReturnTypeTexts")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn is_function_like(kind: &str) -> bool {
    matches!(
        kind,
        "ArrowFunctionExpression" | "FunctionExpression" | "FunctionDeclaration"
    )
}

fn is_arrow_func_expr_or_method(kind: &str) -> bool {
    matches!(
        kind,
        "ArrowFunctionExpression" | "FunctionExpression" | "MethodDefinition"
    )
}

/// The explicit return-type annotation text of a function node, if any
/// (TS-ESTree exposes it under `returnType` -> `typeAnnotation`).
fn return_type_annotation(func: &LintNode) -> Option<&LintNode> {
    func.child("returnType")
        .and_then(|rt| rt.child("typeAnnotation"))
}

/// Faithful port of `reportIfNonVoidFunction`. `func_node` is the value node
/// (function/expression) that occupies a void-expected position.
fn report_if_non_void_function(ctx: &mut RuleContext<'_>, func_node: &LintNode, allow_any: bool) {
    // Early return: the value's own apparent call signatures all return an
    // allowed (void-like) type. Go's `utils.Every` over the (possibly empty)
    // set of call signatures is vacuously true when the value has no call
    // signatures, so an empty apparent-type set also short-circuits.
    let apparent = apparent_return_type_texts(func_node);
    if is_allowed_return_type_texts(&apparent, allow_any, true) {
        return;
    }

    let kind = func_node.kind.as_str();

    // Not an arrow/function-expression/method -> it cannot be inspected as a
    // function literal; report it as a value-returning function.
    if !is_arrow_func_expr_or_method(kind) {
        ctx.report("nonVoidFunc", func_node.range);
        return;
    }

    if func_node.field_bool("generator").unwrap_or(false) {
        ctx.report("nonVoidFunc", func_node.range);
        return;
    }
    if func_node.field_bool("async").unwrap_or(false) {
        ctx.report("asyncFunc", func_node.range);
        return;
    }

    // Concise (expression) arrow body.
    if let Some(body) = func_node.child("body") {
        if body.kind != "BlockStatement" {
            ctx.report("nonVoidReturn", body.range);
            return;
        }
    }

    // Explicit non-void return type annotation.
    if let Some(rt) = return_type_annotation(func_node) {
        if rt.kind != "TSVoidKeyword" {
            ctx.report("nonVoidFunc", rt.range);
            return;
        }
    }

    // Walk the body, reporting each value-bearing `return` whose argument type
    // is not allowed, skipping nested functions.
    report_bad_returns_in_body(ctx, func_node, allow_any);
}

/// Visits return statements belonging to `func_node` (skipping nested
/// functions) and reports those whose argument type is not allowed.
fn report_bad_returns_in_body(ctx: &mut RuleContext<'_>, func_node: &LintNode, allow_any: bool) {
    let mut bad = Vec::new();
    for child in func_node.children.values() {
        collect_bad_returns(child, allow_any, &mut bad);
    }
    for list in func_node.child_lists.values() {
        for child in list {
            collect_bad_returns(child, allow_any, &mut bad);
        }
    }
    for range in bad {
        ctx.report("nonVoidReturn", range);
    }
}

fn collect_bad_returns(node: &LintNode, allow_any: bool, out: &mut Vec<TextRange>) {
    if is_function_like(&node.kind) {
        return;
    }
    if node.kind == "ReturnStatement" {
        if let Some(argument) = node.child("argument") {
            if !is_allowed_return_type_texts(&argument.type_texts, allow_any, false) {
                out.push(node.range);
            }
        }
    }
    for child in node.children.values() {
        collect_bad_returns(child, allow_any, out);
    }
    for list in node.child_lists.values() {
        for child in list {
            collect_bad_returns(child, allow_any, out);
        }
    }
}

/// Port of `checkExpressionNode`: if the host marked this value node as being
/// in a void-expected position, run the function inspection on it.
fn check_expression_node(ctx: &mut RuleContext<'_>, node: &LintNode, allow_any: bool) -> bool {
    if void_return_expected(node) {
        report_if_non_void_function(ctx, node, allow_any);
        return true;
    }
    false
}

fn check_strict_void_return(ctx: &mut RuleContext<'_>, node: &LintNode) {
    let allow_any = allow_return_any(node);
    match node.kind.as_str() {
        "ArrayExpression" => {
            for elem in child_list(node, "elements") {
                if elem.kind != "SpreadElement" {
                    check_expression_node(ctx, elem, allow_any);
                }
            }
        }
        "ArrowFunctionExpression" => {
            // Concise-body arrows are themselves expressions in a position; the
            // host marks the *body* with __voidReturnExpected when contextual.
            if let Some(body) = node.child("body") {
                if body.kind != "BlockStatement" {
                    check_expression_node(ctx, body, allow_any);
                }
            }
        }
        "AssignmentExpression" => {
            if let Some(right) = node.child("right") {
                check_expression_node(ctx, right, allow_any);
            }
        }
        "CallExpression" | "NewExpression" => {
            for arg in child_list(node, "arguments") {
                if arg.kind == "SpreadElement" {
                    continue;
                }
                check_expression_node(ctx, arg, allow_any);
            }
        }
        "JSXAttribute" => {
            if let Some(value) = node.child("value") {
                if value.kind == "JSXExpressionContainer" {
                    if let Some(expr) = value.child("expression") {
                        if expr.kind != "JSXEmptyExpression" {
                            check_expression_node(ctx, expr, allow_any);
                        }
                    }
                }
            }
        }
        "MethodDefinition" => {
            if let Some(value) = node.child("value") {
                check_expression_node(ctx, value, allow_any);
            }
        }
        "Property" => {
            if let Some(value) = node.child("value") {
                check_expression_node(ctx, value, allow_any);
            }
        }
        "PropertyDefinition" => {
            if let Some(value) = node.child("value") {
                check_expression_node(ctx, value, allow_any);
            }
        }
        "ReturnStatement" => {
            if let Some(argument) = node.child("argument") {
                check_expression_node(ctx, argument, allow_any);
            }
        }
        "VariableDeclaration" => {
            for declarator in child_list(node, "declarations") {
                if let Some(init) = declarator.child("init") {
                    check_expression_node(ctx, init, allow_any);
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::super::{LintNode, LintRuleRegistry};

    fn run(node: &LintNode) -> Vec<String> {
        LintRuleRegistry::with_default_type_aware_rules()
            .run_rule("strict-void-return", node)
            .unwrap()
            .into_iter()
            .map(|d| d.message_id)
            .collect()
    }

    // declare function foo(cb: () => void): void; foo(() => null);
    // -> nonVoidReturn on the concise body.
    #[test]
    fn reports_non_void_concise_arrow_argument() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "CallExpression",
            "range": {"start": 0, "end": 16},
            "childLists": {
                "arguments": [{
                    "kind": "ArrowFunctionExpression",
                    "range": {"start": 4, "end": 14},
                    "fields": {"__voidReturnExpected": true, "__functionApparentReturnTypeTexts": ["null"]},
                    "children": {
                        "body": {
                            "kind": "Literal",
                            "range": {"start": 10, "end": 14},
                            "typeTexts": ["null"]
                        }
                    }
                }]
            }
        }))
        .unwrap();
        assert_eq!(run(&node), vec!["nonVoidReturn"]);
    }

    // foo(async () => {}) where void expected -> asyncFunc.
    #[test]
    fn reports_async_function_argument() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "CallExpression",
            "range": {"start": 0, "end": 20},
            "childLists": {
                "arguments": [{
                    "kind": "ArrowFunctionExpression",
                    "range": {"start": 4, "end": 18},
                    "fields": {"__voidReturnExpected": true, "async": true, "__functionApparentReturnTypeTexts": ["Promise<void>"]},
                    "children": {
                        "body": {"kind": "BlockStatement", "range": {"start": 12, "end": 18}}
                    }
                }]
            }
        }))
        .unwrap();
        assert_eq!(run(&node), vec!["asyncFunc"]);
    }

    // const foo: () => void = cb; where cb(): boolean -> nonVoidFunc (identifier,
    // not a function literal).
    #[test]
    fn reports_value_returning_identifier_in_void_position() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "VariableDeclaration",
            "range": {"start": 0, "end": 26},
            "childLists": {
                "declarations": [{
                    "kind": "VariableDeclarator",
                    "range": {"start": 6, "end": 26},
                    "children": {
                        "init": {
                            "kind": "Identifier",
                            "range": {"start": 24, "end": 26},
                            "fields": {
                                "__voidReturnExpected": true,
                                "__functionApparentReturnTypeTexts": ["boolean"]
                            }
                        }
                    }
                }]
            }
        }))
        .unwrap();
        assert_eq!(run(&node), vec!["nonVoidFunc"]);
    }

    // NOT reported: function whose apparent return type is void.
    #[test]
    fn ignores_void_returning_function_argument() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "CallExpression",
            "range": {"start": 0, "end": 16},
            "childLists": {
                "arguments": [{
                    "kind": "Identifier",
                    "range": {"start": 4, "end": 6},
                    "fields": {
                        "__voidReturnExpected": true,
                        "__functionApparentReturnTypeTexts": ["void"]
                    }
                }]
            }
        }))
        .unwrap();
        assert!(run(&node).is_empty());
    }

    // NOT reported: no void-expected position marked.
    #[test]
    fn ignores_when_not_void_position() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "CallExpression",
            "range": {"start": 0, "end": 16},
            "childLists": {
                "arguments": [{
                    "kind": "ArrowFunctionExpression",
                    "range": {"start": 4, "end": 14},
                    "children": {
                        "body": {"kind": "Literal", "range": {"start": 10, "end": 14}, "typeTexts": ["number"]}
                    }
                }]
            }
        }))
        .unwrap();
        assert!(run(&node).is_empty());
    }

    // NOT reported: throw-only block body returns never-ish, no value return.
    #[test]
    fn ignores_block_body_without_value_return() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "CallExpression",
            "range": {"start": 0, "end": 30},
            "childLists": {
                "arguments": [{
                    "kind": "ArrowFunctionExpression",
                    "range": {"start": 4, "end": 28},
                    "fields": {"__voidReturnExpected": true},
                    "children": {
                        "body": {
                            "kind": "BlockStatement",
                            "range": {"start": 10, "end": 28},
                            "childLists": {
                                "body": [{
                                    "kind": "ThrowStatement",
                                    "range": {"start": 12, "end": 26}
                                }]
                            }
                        }
                    }
                }]
            }
        }))
        .unwrap();
        assert!(run(&node).is_empty());
    }

    // NOT reported: a non-callable value (no apparent call signatures) flagged
    // in a void position. Go's `Every` over zero call signatures is vacuously
    // true, so reportIfNonVoidFunction returns early without reporting.
    #[test]
    fn ignores_non_callable_value_in_void_position() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "CallExpression",
            "range": {"start": 0, "end": 16},
            "childLists": {
                "arguments": [{
                    "kind": "Identifier",
                    "range": {"start": 4, "end": 6},
                    "fields": {"__voidReturnExpected": true}
                }]
            }
        }))
        .unwrap();
        assert!(run(&node).is_empty());
    }

    // block body with a value-bearing return -> nonVoidReturn at the return.
    #[test]
    fn reports_value_return_inside_block_body() {
        let node: LintNode = serde_json::from_value(json!({
            "kind": "CallExpression",
            "range": {"start": 0, "end": 40},
            "childLists": {
                "arguments": [{
                    "kind": "ArrowFunctionExpression",
                    "range": {"start": 4, "end": 38},
                    "fields": {"__voidReturnExpected": true, "__functionApparentReturnTypeTexts": ["number"]},
                    "children": {
                        "body": {
                            "kind": "BlockStatement",
                            "range": {"start": 10, "end": 38},
                            "childLists": {
                                "body": [{
                                    "kind": "ReturnStatement",
                                    "range": {"start": 14, "end": 24},
                                    "children": {
                                        "argument": {
                                            "kind": "Literal",
                                            "range": {"start": 21, "end": 23},
                                            "typeTexts": ["number"]
                                        }
                                    }
                                }]
                            }
                        }
                    }
                }]
            }
        }))
        .unwrap();
        assert_eq!(run(&node), vec!["nonVoidReturn"]);
    }
}
