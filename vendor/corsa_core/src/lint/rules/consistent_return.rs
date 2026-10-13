use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule};
use crate::lint::helpers::{is_identifier_named, rule_option_bool};

/// Type-aware rule that requires functions to return values consistently:
/// either always with a value or always without one.
///
/// This is a faithful port of the tsgolint `consistent-return` builtin. The Go
/// rule maintains a per-function state machine across `enterFunction` /
/// `exitFunction` / `onReturnStatement` listener callbacks. The Rust host model
/// instead drives this rule once per function-like node, so the per-function
/// state is reconstructed here by walking the return statements that belong to
/// the current function body (skipping nested functions, mirroring the upstream
/// scoping behaviour).
#[derive(Clone, Copy, Debug, Default)]
pub struct ConsistentReturnRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "missingReturnValue",
        description: "Function expected a return value.",
    },
    RuleMessage {
        id: "unexpectedReturnValue",
        description: "Function expected no return value.",
    },
];

const LISTENERS: &[&str] = &[
    "FunctionDeclaration",
    "FunctionExpression",
    "ArrowFunctionExpression",
    "MethodDefinition",
    "TSMethodSignature",
];

impl RustLintRule for ConsistentReturnRule {
    fn name(&self) -> &'static str {
        "consistent-return"
    }

    fn docs_description(&self) -> &'static str {
        "Require return statements to either always or never specify values."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        if !is_function_like_kind(&node.kind) {
            return;
        }

        let treat_undefined_as_unspecified =
            rule_option_bool(node, "treatUndefinedAsUnspecified").unwrap_or(false);

        // Mirror functionState.allowsVoidReturn: the host computes whether the
        // function's call signature returns void (or, for async functions, a
        // thenable resolving to void). When true, bare `return;` statements are
        // permitted alongside value returns.
        let allows_void_return = node.field_bool("__allowsVoidReturn").unwrap_or(false);

        // The presentational "Function 'foo'" / "Async function" label used in
        // diagnostics. Currently only used for static message text, but consumed
        // here so the verdict logic stays aligned with upstream.
        let _function_name_with_kind = node.field_str("__functionNameWithKind");

        let returns = direct_returns_of_function(node);

        let mut has_return = false;
        let mut expected_has_value = false;
        let mut message_id: Option<&'static str> = None;
        let mut has_mismatch = false;
        let mut has_return_value_seen = false;

        for return_node in returns {
            let argument = return_node.child("argument");

            // `return;` with no argument is allowed when the function may return
            // void; it does not participate in the consistency check.
            if argument.is_none() && allows_void_return {
                continue;
            }

            let has_value = match argument {
                None => false,
                Some(arg) => has_return_value(arg, treat_undefined_as_unspecified),
            };

            if !has_return {
                has_return = true;
                expected_has_value = has_value;
                has_return_value_seen = has_value;
                message_id = Some(if has_value {
                    "missingReturnValue"
                } else {
                    "unexpectedReturnValue"
                });
                continue;
            }

            if expected_has_value != has_value {
                has_mismatch = true;
                if let Some(id) = message_id {
                    ctx.report(id, return_node.range);
                }
            }
        }

        // exitFunction: a function that has only value-returning statements but
        // can also fall off its end (implicit `return;`) is inconsistent and is
        // reported on the function node itself, unless it is allowed to return
        // void.
        if !has_mismatch
            && has_return
            && has_return_value_seen
            && !allows_void_return
            && node.field_bool("__hasImplicitReturn").unwrap_or(false)
        {
            ctx.report("missingReturnValue", node.range);
        }
    }
}

/// Mirrors `getHasReturnValue`: a return statement specifies a value when it has
/// an argument, except that with `treatUndefinedAsUnspecified` an argument whose
/// type is exactly `undefined`, or which is the literal `undefined`, is treated
/// as specifying no value.
fn has_return_value(argument: &LintNode, treat_undefined_as_unspecified: bool) -> bool {
    if !treat_undefined_as_unspecified {
        return true;
    }
    if is_exactly_undefined_type(argument) {
        return false;
    }
    !is_undefined_literal(argument)
}

/// `Type_flags(returnValueType) == TypeFlagsUndefined`: the argument's type is
/// exactly `undefined` (not a union that merely contains it).
fn is_exactly_undefined_type(argument: &LintNode) -> bool {
    matches!(argument.type_texts.as_slice(), [only] if only.trim() == "undefined")
}

/// `IsUndefinedLiteral`: the argument is the `undefined` identifier or a
/// `void`-operator expression such as `void 0`.
fn is_undefined_literal(argument: &LintNode) -> bool {
    if is_identifier_named(argument, "undefined") {
        return true;
    }
    argument.kind == "UnaryExpression" && argument.field_str("operator") == Some("void")
}

/// Collects return statements that belong directly to this function's body,
/// skipping any nested function-like declarations (which form their own scope).
fn direct_returns_of_function(node: &LintNode) -> Vec<&LintNode> {
    let mut out = Vec::new();
    // Visit the function body and any other direct children (arrow bodies live
    // under `body`); never re-enter the function node itself.
    for child in node.children.values() {
        collect_returns(child, &mut out);
    }
    for list in node.child_lists.values() {
        for child in list {
            collect_returns(child, &mut out);
        }
    }
    out
}

fn collect_returns<'a>(node: &'a LintNode, out: &mut Vec<&'a LintNode>) {
    if is_function_like_kind(&node.kind) {
        return;
    }
    if node.kind == "ReturnStatement" {
        out.push(node);
    }
    for child in node.children.values() {
        collect_returns(child, out);
    }
    for list in node.child_lists.values() {
        for child in list {
            collect_returns(child, out);
        }
    }
}

fn is_function_like_kind(kind: &str) -> bool {
    matches!(
        kind,
        "FunctionDeclaration"
            | "FunctionExpression"
            | "ArrowFunctionExpression"
            | "MethodDefinition"
            | "TSMethodSignature"
            | "TSDeclareFunction"
            | "TSEmptyBodyFunctionExpression"
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::super::super::{LintNode, RuleContext, RustLintRule, TextRange};
    use super::ConsistentReturnRule;

    fn node(kind: &str, range: TextRange) -> LintNode {
        LintNode {
            kind: kind.to_owned(),
            range,
            text: None,
            type_texts: Vec::new(),
            property_names: Vec::new(),
            fields: BTreeMap::new(),
            children: BTreeMap::new(),
            child_lists: BTreeMap::new(),
        }
    }

    fn return_with_value(range: TextRange, value: serde_json::Value) -> LintNode {
        let mut argument = node("Literal", TextRange::new(range.start + 7, range.end - 1));
        argument.fields.insert("value".to_owned(), value);
        let mut return_node = node("ReturnStatement", range);
        return_node.children.insert("argument".to_owned(), argument);
        return_node
    }

    fn bare_return(range: TextRange) -> LintNode {
        node("ReturnStatement", range)
    }

    fn function_with_returns(returns: Vec<LintNode>) -> LintNode {
        let body = {
            let mut body = node("BlockStatement", TextRange::new(10, 200));
            body.child_lists.insert("body".to_owned(), returns);
            body
        };
        let mut function = node("FunctionDeclaration", TextRange::new(0, 201));
        function.children.insert("body".to_owned(), body);
        function
    }

    fn run(function: &LintNode) -> Vec<super::super::super::LintDiagnostic> {
        let rule = ConsistentReturnRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, function);
        ctx.finish()
    }

    #[test]
    fn reports_value_then_bare_return_mismatch() {
        // function foo(flag) { if (flag) return true; else return; }
        let function = function_with_returns(vec![
            return_with_value(TextRange::new(30, 42), json!(true)),
            bare_return(TextRange::new(48, 55)),
        ]);

        let diagnostics = run(&function);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "missingReturnValue");
        assert_eq!(diagnostics[0].range, TextRange::new(48, 55));
    }

    #[test]
    fn reports_bare_then_value_return_mismatch() {
        // async function foo(flag) { if (flag) return; else return 'value'; }
        let function = function_with_returns(vec![
            bare_return(TextRange::new(30, 37)),
            return_with_value(TextRange::new(48, 65), json!("value")),
        ]);

        let diagnostics = run(&function);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "unexpectedReturnValue");
        assert_eq!(diagnostics[0].range, TextRange::new(48, 65));
    }

    #[test]
    fn reports_implicit_return_when_value_returns_exist() {
        // function foo(flag) { if (flag) { return 1; } }  -> falls off the end.
        let mut function =
            function_with_returns(vec![return_with_value(TextRange::new(40, 49), json!(1))]);
        function
            .fields
            .insert("__hasImplicitReturn".to_owned(), json!(true));

        let diagnostics = run(&function);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "missingReturnValue");
        assert_eq!(diagnostics[0].range, function.range);
    }

    #[test]
    fn allows_consistent_value_returns() {
        // const foo = (flag) => { if (flag) return true; return false; };
        let function = function_with_returns(vec![
            return_with_value(TextRange::new(30, 42), json!(true)),
            return_with_value(TextRange::new(48, 61), json!(false)),
        ]);

        let diagnostics = run(&function);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn allows_bare_returns_when_void_returning() {
        // declare function bar(): void;
        // function foo(flag): void { if (flag) return bar(); return; }
        let mut call = node("CallExpression", TextRange::new(37, 42));
        call.type_texts = vec!["void".to_owned()];
        let mut value_return = node("ReturnStatement", TextRange::new(30, 43));
        value_return.children.insert("argument".to_owned(), call);

        let mut function =
            function_with_returns(vec![value_return, bare_return(TextRange::new(49, 56))]);
        function
            .fields
            .insert("__allowsVoidReturn".to_owned(), json!(true));

        let diagnostics = run(&function);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn treats_undefined_as_unspecified_when_option_set() {
        // const foo = (flag) => { if (flag) return; else return undefined; };
        // with { treatUndefinedAsUnspecified: true } -> consistent (both bare).
        let mut undefined_identifier = node("Identifier", TextRange::new(55, 64));
        undefined_identifier
            .fields
            .insert("name".to_owned(), json!("undefined"));
        undefined_identifier.type_texts = vec!["undefined".to_owned()];
        let mut undefined_return = node("ReturnStatement", TextRange::new(48, 65));
        undefined_return
            .children
            .insert("argument".to_owned(), undefined_identifier);

        let mut function =
            function_with_returns(vec![bare_return(TextRange::new(30, 37)), undefined_return]);
        function.fields.insert(
            "__ruleOptions".to_owned(),
            json!([{ "treatUndefinedAsUnspecified": true }]),
        );

        let diagnostics = run(&function);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn ignores_nested_function_returns() {
        // Returns inside a nested function must not affect the outer function.
        let nested = {
            let mut nested_body = node("BlockStatement", TextRange::new(60, 120));
            nested_body.child_lists.insert(
                "body".to_owned(),
                vec![
                    bare_return(TextRange::new(70, 77)),
                    return_with_value(TextRange::new(85, 95), json!(1)),
                ],
            );
            let mut nested = node("FunctionDeclaration", TextRange::new(50, 121));
            nested.children.insert("body".to_owned(), nested_body);
            nested
        };
        let function = function_with_returns(vec![
            nested,
            return_with_value(TextRange::new(130, 140), json!(1)),
            return_with_value(TextRange::new(145, 158), json!(2)),
        ]);

        let diagnostics = run(&function);

        assert!(diagnostics.is_empty());
    }
}
