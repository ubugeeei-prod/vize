use serde_json::Value;

use super::super::{LintFix, LintNode, LintSuggestion, RuleContext, RuleMessage, RustLintRule};
use crate::lint::helpers::rule_option_bool;
use crate::utils::{is_any_like_type_texts, is_unknown_like_type_texts, split_top_level_type_text};

/// Type-aware rule that requires functions returning promises to be `async`.
///
/// Faithful port of the tsgolint builtin `promise-function-async` rule. All
/// decision logic lives here over [`LintNode`] facts; the host only supplies
/// raw type-checker query results (per-signature return type texts and whether
/// an explicit return-type annotation is present).
#[derive(Clone, Copy, Debug, Default)]
pub struct PromiseFunctionAsyncRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "missingAsync",
        description: "Functions that return promises must be async.",
    },
    RuleMessage {
        id: "missingAsyncHybridReturn",
        description: "Functions that return promises must be async.",
    },
    RuleMessage {
        id: "missingAsyncHybridReturnSuggestion",
        description: "Add `async` keyword to the function.",
    },
];

// Go installs listeners for arrow functions, function declarations, function
// expressions, and method declarations. In the TypeScript-Go AST a
// `MethodDeclaration` covers both class methods and object-literal methods; in
// TS-ESTree those split into `MethodDefinition` (class) and `Property`
// (object-literal method shorthand), so both kinds are observed here and gated
// behind `checkMethodDeclarations`.
const LISTENERS: &[&str] = &[
    "FunctionDeclaration",
    "FunctionExpression",
    "ArrowFunctionExpression",
    "MethodDefinition",
    "Property",
];

impl RustLintRule for PromiseFunctionAsyncRule {
    fn name(&self) -> &'static str {
        "promise-function-async"
    }

    fn docs_description(&self) -> &'static str {
        "Require functions that return promises to be async."
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
        let options = Options::from_node(node);

        // Honor the per-kind `check*` options (Go installs listeners selectively).
        if !options.enabled_for(&node.kind) {
            return;
        }

        // Class members arrive as `MethodDefinition`. In the TS-Go AST,
        // accessors and constructors are *distinct* node kinds (GetAccessor /
        // SetAccessor / Constructor) and never reach `validateNode`; in TS-ESTree
        // they are all `MethodDefinition` discriminated by the ESTree `kind`
        // string. Only plain methods (`kind === "method"`) must be checked.
        if node.kind == "MethodDefinition" {
            if node.field_str("kind") != Some("method") {
                return;
            }
            // Abstract methods can't be async (Go guards this).
            if node.field_bool("__isAbstract").unwrap_or(false) {
                return;
            }
        }

        // Object-literal members arrive as `Property`. Only the method shorthand
        // (`{ foo() {} }`) corresponds to Go's `MethodDeclaration`; data
        // properties, getters and setters (`kind` get/set, `method === false`)
        // are not method declarations and are skipped.
        if node.kind == "Property"
            && (!node.field_bool("method").unwrap_or(false)
                || node.field_str("kind") != Some("init"))
        {
            return;
        }

        validate_node(ctx, node, &options);
    }
}

struct Options {
    allow_any: bool,
    allowed_promise_names: Vec<String>,
    check_arrow_functions: bool,
    check_function_declarations: bool,
    check_function_expressions: bool,
    check_method_declarations: bool,
}

impl Options {
    fn from_node(node: &LintNode) -> Self {
        let mut allowed_promise_names = vec!["Promise".to_owned()];
        if let Some(names) = node
            .fields
            .get("__ruleOptions")
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .and_then(|first| first.get("allowedPromiseNames"))
            .and_then(Value::as_array)
        {
            for name in names.iter().filter_map(Value::as_str) {
                allowed_promise_names.push(name.to_owned());
            }
        }
        Self {
            allow_any: rule_option_bool(node, "allowAny").unwrap_or(true),
            allowed_promise_names,
            check_arrow_functions: rule_option_bool(node, "checkArrowFunctions").unwrap_or(true),
            check_function_declarations: rule_option_bool(node, "checkFunctionDeclarations")
                .unwrap_or(true),
            check_function_expressions: rule_option_bool(node, "checkFunctionExpressions")
                .unwrap_or(true),
            check_method_declarations: rule_option_bool(node, "checkMethodDeclarations")
                .unwrap_or(true),
        }
    }

    fn enabled_for(&self, kind: &str) -> bool {
        match kind {
            "ArrowFunctionExpression" => self.check_arrow_functions,
            "FunctionDeclaration" => self.check_function_declarations,
            "FunctionExpression" => self.check_function_expressions,
            // Both class methods (MethodDefinition) and object-literal method
            // shorthand (Property) map to Go's MethodDeclaration listener.
            "MethodDefinition" | "Property" => self.check_method_declarations,
            _ => false,
        }
    }
}

fn validate_node(ctx: &mut RuleContext<'_>, node: &LintNode, options: &Options) {
    // Skip already-async functions and declarations without a body (overload
    // signatures, abstract members, declare functions).
    if is_async(node) || !has_body(node) {
        return;
    }

    let signatures = signature_return_type_texts(node);
    if signatures.is_empty() {
        return;
    }

    let has_explicit_return_type = node.field_bool("__hasExplicitReturnType").unwrap_or(false);

    let mut every_signature_returns_promise = true;
    for return_type_texts in &signatures {
        // Report without fixer when the return type is unknown/any and allowAny
        // is disabled (Go reports `missingAsync` and returns early).
        if !options.allow_any
            && (is_any_like_type_texts(return_type_texts)
                || is_unknown_like_type_texts(return_type_texts))
        {
            ctx.report("missingAsync", node.range);
            return;
        }

        // Require every potential return type to be promise/any/unknown. When no
        // explicit return type is set, any matching part is sufficient
        // (match_any_instead = has_explicit_return_type).
        every_signature_returns_promise = every_signature_returns_promise
            && contains_all_types_by_name(
                return_type_texts,
                has_explicit_return_type,
                &options.allowed_promise_names,
            );
    }

    if !every_signature_returns_promise {
        return;
    }

    // Detect a hybrid return type: a union containing both promise and
    // non-promise parts. Only relevant without an explicit return annotation.
    let mut is_hybrid_return_type = false;
    if !has_explicit_return_type {
        'outer: for return_type_texts in &signatures {
            for text in return_type_texts {
                let parts = split_top_level_type_text(text, '|');
                if parts.len() < 2 {
                    continue;
                }
                let all_parts_are_promise = parts.iter().all(|part| {
                    contains_all_types_by_name(
                        std::slice::from_ref(part),
                        true,
                        &options.allowed_promise_names,
                    )
                });
                if !all_parts_are_promise {
                    is_hybrid_return_type = true;
                    break 'outer;
                }
            }
        }
    }

    if is_hybrid_return_type {
        let suggestion = LintSuggestion {
            message_id: "missingAsyncHybridReturnSuggestion".to_owned(),
            message: "Add `async` keyword to the function.".to_owned(),
            fixes: insert_async_fix(node),
        };
        ctx.report_with_suggestions("missingAsyncHybridReturn", node.range, vec![suggestion]);
    } else {
        // Go emits `missingAsync` with an auto-fix here. Suggestions carry the
        // fix in this binding's diagnostic shape.
        let suggestion = LintSuggestion {
            message_id: "missingAsync".to_owned(),
            message: "Functions that return promises must be async.".to_owned(),
            fixes: insert_async_fix(node),
        };
        ctx.report_with_suggestions("missingAsync", node.range, vec![suggestion]);
    }
}

/// Returns whether the function under `node` carries the `async` modifier.
///
/// For `MethodDefinition`/`Property` nodes the `async` flag lives on the inner
/// function (`value`), not on the member node itself.
fn is_async(node: &LintNode) -> bool {
    match node.kind.as_str() {
        "MethodDefinition" | "Property" => node
            .child("value")
            .and_then(|value| value.field_bool("async"))
            .unwrap_or(false),
        _ => node.field_bool("async").unwrap_or(false),
    }
}

fn has_body(node: &LintNode) -> bool {
    match node.kind.as_str() {
        // For a MethodDefinition / object-literal method the function lives
        // under `value`.
        "MethodDefinition" | "Property" => node
            .child("value")
            .is_some_and(|value| value.child("body").is_some()),
        _ => node.child("body").is_some(),
    }
}

/// Returns one type-text list per call signature, or an empty vector when the
/// host could not determine any call signature.
fn signature_return_type_texts(node: &LintNode) -> Vec<Vec<String>> {
    if let Some(signatures) = node
        .fields
        .get("__signatureReturnTypeTexts")
        .and_then(Value::as_array)
    {
        return signatures
            .iter()
            .map(|signature| {
                signature
                    .as_array()
                    .map(|texts| {
                        texts
                            .iter()
                            .filter_map(Value::as_str)
                            .map(str::to_owned)
                            .collect()
                    })
                    .unwrap_or_default()
            })
            .collect();
    }
    Vec::new()
}

/// Mirrors Go's `containsAllTypesByName` over rendered type texts.
///
/// `match_any_instead` corresponds to Go's `matchAnyInstead` flag: when `true`
/// every union/intersection constituent must match; when `false` any
/// constituent matching is sufficient.
fn contains_all_types_by_name(
    type_texts: &[String],
    match_any_instead: bool,
    allowed_promise_names: &[String],
) -> bool {
    if type_texts.is_empty() {
        return false;
    }
    type_texts
        .iter()
        .all(|text| type_text_matches(text, match_any_instead, allowed_promise_names))
}

fn type_text_matches(
    text: &str,
    match_any_instead: bool,
    allowed_promise_names: &[String],
) -> bool {
    let text = text.trim();
    if text.is_empty() {
        return false;
    }
    // any/unknown are never a match (Go bails on TypeFlagsAnyOrUnknown).
    if matches!(text, "any" | "unknown") {
        return false;
    }

    // Split top-level unions/intersections *before* matching a named type, so a
    // union such as `Promise<number> | number` is never mistaken for the single
    // named type `Promise` (which would wrongly match on the leading `<`).
    let union_parts = split_top_level_type_text(text, '|');
    if union_parts.len() > 1 {
        return if match_any_instead {
            union_parts
                .iter()
                .all(|part| type_text_matches(part, match_any_instead, allowed_promise_names))
        } else {
            union_parts
                .iter()
                .any(|part| type_text_matches(part, match_any_instead, allowed_promise_names))
        };
    }

    let intersection_parts = split_top_level_type_text(text, '&');
    if intersection_parts.len() > 1 {
        return if match_any_instead {
            intersection_parts
                .iter()
                .all(|part| type_text_matches(part, match_any_instead, allowed_promise_names))
        } else {
            intersection_parts
                .iter()
                .any(|part| type_text_matches(part, match_any_instead, allowed_promise_names))
        };
    }

    // Atomic (non-union, non-intersection) type text: match by name.
    type_name_is_allowed(text, allowed_promise_names)
}

/// Returns whether a single (non-union, non-intersection) type text names an
/// allowed promise type, e.g. `Promise<void>` or a user-configured name.
fn type_name_is_allowed(text: &str, allowed_promise_names: &[String]) -> bool {
    let text = text.trim();
    let base = text
        .split_once('<')
        .map(|(head, _)| head.trim())
        .unwrap_or(text);
    allowed_promise_names.iter().any(|name| name == base)
}

/// Builds the `async`-insertion fix. The exact insertion site differs by node
/// kind in Go; here it always inserts before the node start, matching the
/// observed fixture output (a leading space then `async `).
fn insert_async_fix(node: &LintNode) -> Vec<LintFix> {
    let insertion = super::super::TextRange::new(node.range.start, node.range.start);
    vec![LintFix::replace_range(insertion, "async ")]
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::super::{LintNode, RuleContext, RustLintRule};
    use super::PromiseFunctionAsyncRule;

    fn run(node: &LintNode) -> Vec<(String, Vec<String>)> {
        let rule = PromiseFunctionAsyncRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, node);
        ctx.finish()
            .into_iter()
            .map(|diagnostic| {
                (
                    diagnostic.message_id,
                    diagnostic
                        .suggestions
                        .into_iter()
                        .map(|suggestion| suggestion.message_id)
                        .collect(),
                )
            })
            .collect()
    }

    fn function(async_flag: bool, signatures: serde_json::Value, has_body: bool) -> LintNode {
        let mut fields = serde_json::Map::new();
        fields.insert("async".to_owned(), json!(async_flag));
        fields.insert("__signatureReturnTypeTexts".to_owned(), signatures);
        let mut children = serde_json::Map::new();
        if has_body {
            children.insert(
                "body".to_owned(),
                json!({
                    "kind": "BlockStatement",
                    "range": { "start": 30, "end": 40 }
                }),
            );
        }
        serde_json::from_value(json!({
            "kind": "FunctionDeclaration",
            "range": { "start": 0, "end": 40 },
            "fields": serde_json::Value::Object(fields),
            "children": serde_json::Value::Object(children),
        }))
        .unwrap()
    }

    /// Builds a class `MethodDefinition` whose inner function (`value`) carries
    /// the `async` flag, body, and signature facts (mirroring TS-ESTree shape).
    fn method(
        estree_kind: &str,
        async_flag: bool,
        signatures: serde_json::Value,
        has_body: bool,
    ) -> LintNode {
        let mut fields = serde_json::Map::new();
        fields.insert("kind".to_owned(), json!(estree_kind));
        fields.insert("__signatureReturnTypeTexts".to_owned(), signatures);

        let mut value_fields = serde_json::Map::new();
        value_fields.insert("async".to_owned(), json!(async_flag));
        let mut value_children = serde_json::Map::new();
        if has_body {
            value_children.insert(
                "body".to_owned(),
                json!({ "kind": "BlockStatement", "range": { "start": 30, "end": 40 } }),
            );
        }
        let value = json!({
            "kind": "FunctionExpression",
            "range": { "start": 5, "end": 40 },
            "fields": serde_json::Value::Object(value_fields),
            "children": serde_json::Value::Object(value_children),
        });

        let mut children = serde_json::Map::new();
        children.insert("value".to_owned(), value);

        serde_json::from_value(json!({
            "kind": "MethodDefinition",
            "range": { "start": 0, "end": 40 },
            "fields": serde_json::Value::Object(fields),
            "children": serde_json::Value::Object(children),
        }))
        .unwrap()
    }

    /// Builds an object-literal `Property` member.
    fn property(
        estree_kind: &str,
        is_method: bool,
        async_flag: bool,
        signatures: serde_json::Value,
    ) -> LintNode {
        let mut fields = serde_json::Map::new();
        fields.insert("kind".to_owned(), json!(estree_kind));
        fields.insert("method".to_owned(), json!(is_method));
        fields.insert("__signatureReturnTypeTexts".to_owned(), signatures);

        let mut value_fields = serde_json::Map::new();
        value_fields.insert("async".to_owned(), json!(async_flag));
        let mut value_children = serde_json::Map::new();
        value_children.insert(
            "body".to_owned(),
            json!({ "kind": "BlockStatement", "range": { "start": 30, "end": 40 } }),
        );
        let value = json!({
            "kind": "FunctionExpression",
            "range": { "start": 5, "end": 40 },
            "fields": serde_json::Value::Object(value_fields),
            "children": serde_json::Value::Object(value_children),
        });

        let mut children = serde_json::Map::new();
        children.insert("value".to_owned(), value);

        serde_json::from_value(json!({
            "kind": "Property",
            "range": { "start": 0, "end": 40 },
            "fields": serde_json::Value::Object(fields),
            "children": serde_json::Value::Object(children),
        }))
        .unwrap()
    }

    #[test]
    fn reports_non_async_promise_returning_function() {
        // function f() { return new Promise<void>(); }
        let node = function(false, json!([["Promise<void>"]]), true);
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].0, "missingAsync");
        assert_eq!(diagnostics[0].1, vec!["missingAsync".to_owned()]);
    }

    #[test]
    fn reports_hybrid_return_as_suggestion() {
        // function f(p) { return p ? Promise.resolve(5) : 5; } -- inferred union.
        let node = function(false, json!([["Promise<number> | number"]]), true);
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].0, "missingAsyncHybridReturn");
        assert_eq!(
            diagnostics[0].1,
            vec!["missingAsyncHybridReturnSuggestion".to_owned()]
        );
    }

    #[test]
    fn reports_any_return_without_fix_when_allow_any_false() {
        // function returnsAny(): any { return 0; } with { allowAny: false }.
        let mut node = function(false, json!([["any"]]), true);
        node.fields
            .insert("__ruleOptions".to_owned(), json!([{ "allowAny": false }]));
        node.fields
            .insert("__hasExplicitReturnType".to_owned(), json!(true));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].0, "missingAsync");
        // Early any/unknown report carries no fix.
        assert!(diagnostics[0].1.is_empty());
    }

    #[test]
    fn does_not_report_already_async_function() {
        let node = function(true, json!([["Promise<void>"]]), true);
        assert!(run(&node).is_empty());
    }

    #[test]
    fn does_not_report_non_promise_return() {
        // function f(n: number) { return n; }
        let node = function(false, json!([["number"]]), true);
        assert!(run(&node).is_empty());
    }

    #[test]
    fn does_not_report_explicit_union_with_promise_part() {
        // function f(): Promise<number> | number { return 5; }
        // Explicit return type => every part must be a promise; `number` is not.
        let mut node = function(false, json!([["Promise<number> | number"]]), true);
        node.fields
            .insert("__hasExplicitReturnType".to_owned(), json!(true));
        assert!(run(&node).is_empty());
    }

    #[test]
    fn does_not_report_without_body() {
        // Overload signature / abstract method: no body.
        let node = function(false, json!([["Promise<void>"]]), false);
        assert!(run(&node).is_empty());
    }

    #[test]
    fn respects_allowed_promise_names() {
        // function f(p) { return p ? new PromiseType() : 5; }
        // with { allowedPromiseNames: ["PromiseType"] } => hybrid suggestion.
        let mut node = function(false, json!([["PromiseType | number"]]), true);
        node.fields.insert(
            "__ruleOptions".to_owned(),
            json!([{ "allowedPromiseNames": ["PromiseType"] }]),
        );
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].0, "missingAsyncHybridReturn");
    }

    #[test]
    fn reports_non_async_promise_method() {
        // class { foo() { return new Promise<void>(); } }
        let node = method("method", false, json!([["Promise<void>"]]), true);
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].0, "missingAsync");
    }

    #[test]
    fn does_not_report_async_method() {
        // Async flag lives on the inner `value` function, not the member node.
        let node = method("method", true, json!([["Promise<void>"]]), true);
        assert!(run(&node).is_empty());
    }

    #[test]
    fn does_not_report_promise_returning_getter() {
        // get asyncGetter() { return new Promise<void>(); } -- accessors are a
        // distinct AST kind in Go and must never be reported.
        let node = method("get", false, json!([["Promise<void>"]]), true);
        assert!(run(&node).is_empty());
    }

    #[test]
    fn does_not_report_promise_returning_constructor() {
        // constructor() { return new Promise<void>(); }
        let node = method("constructor", false, json!([["Promise<void>"]]), true);
        assert!(run(&node).is_empty());
    }

    #[test]
    fn does_not_report_method_when_check_method_declarations_false() {
        let mut node = method("method", false, json!([["Promise<void>"]]), true);
        node.fields.insert(
            "__ruleOptions".to_owned(),
            json!([{ "checkMethodDeclarations": false }]),
        );
        assert!(run(&node).is_empty());
    }

    #[test]
    fn reports_object_literal_method_shorthand() {
        // const o = { foo() { return Promise.resolve(1); } };
        let node = property("init", true, false, json!([["Promise<number>"]]));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].0, "missingAsync");
    }

    #[test]
    fn does_not_report_object_literal_getter() {
        // const o = { get foo() { return new Promise<void>(); } };
        let node = property("get", false, false, json!([["Promise<void>"]]));
        assert!(run(&node).is_empty());
    }

    #[test]
    fn does_not_report_object_data_property() {
        // const o = { foo: () => {} }; -- not a method shorthand.
        let node = property("init", false, false, json!([["Promise<void>"]]));
        assert!(run(&node).is_empty());
    }
}
