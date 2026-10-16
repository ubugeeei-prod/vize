use super::super::{
    LintFix, LintNode, LintSuggestion, RuleContext, RuleMessage, RustLintRule, TextRange,
};
use crate::lint::helpers::strip_chain_expression;
use crate::utils::{is_array_like_type_texts, is_string_like_type_texts};

/// Type-aware port of tsgolint `no-misused-spread`.
///
/// Disallows spreading values whose runtime behavior surprises authors: strings
/// in array/call spreads, and arrays / Maps / iterables / promises / functions /
/// class declarations / class instances in object (and JSX) spreads.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoMisusedSpreadRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "addAwait",
        description: "Add await operator.",
    },
    RuleMessage {
        id: "noArraySpreadInObject",
        description: "Using the spread operator on an array in an object will result in a list of indices.",
    },
    RuleMessage {
        id: "noClassDeclarationSpreadInObject",
        description: "Using the spread operator on class declarations will spread only their static properties, and will lose their class prototype.",
    },
    RuleMessage {
        id: "noClassInstanceSpreadInObject",
        description: "Using the spread operator on class instances will lose their class prototype.",
    },
    RuleMessage {
        id: "noFunctionSpreadInObject",
        description: "Using the spread operator on a function without additional properties can cause unexpected behavior.",
    },
    RuleMessage {
        id: "noIterableSpreadInObject",
        description: "Using the spread operator on an Iterable in an object can cause unexpected behavior.",
    },
    RuleMessage {
        id: "noMapSpreadInObject",
        description: "Using the spread operator on a Map in an object will result in an empty object.",
    },
    RuleMessage {
        id: "noPromiseSpreadInObject",
        description: "Using the spread operator on Promise in an object can cause unexpected behavior.",
    },
    RuleMessage {
        id: "noStringSpread",
        description: "Using the spread operator on a string can mishandle special characters, because it produces Unicode code points, which will break complex characters (like emojis) into multiple parts.",
    },
    RuleMessage {
        id: "replaceMapSpreadInObject",
        description: "Replace map spread in object with `Object.fromEntries()`",
    },
];

// The host emits one `SpreadElement` per array/call/object spread and one
// `JSXSpreadAttribute` per JSX spread; `__parentKind` disambiguates the position.
const LISTENERS: &[&str] = &["SpreadElement", "JSXSpreadAttribute"];

impl RustLintRule for NoMisusedSpreadRule {
    fn name(&self) -> &'static str {
        "no-misused-spread"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow spreading values into incompatible positions."
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
        // The spread argument expression. ESTree spells it `argument` for
        // SpreadElement and `argument`/`expression` for JSX spread; accept both.
        let Some(argument) = node.child("argument").or_else(|| node.child("expression")) else {
            return;
        };

        // `allow` option: a type matched by a configured specifier is never
        // reported. The host resolves the (type, specifier) match and records
        // the raw result on the spread node.
        if spread_is_allowed(node) {
            return;
        }

        match spread_position(node) {
            SpreadPosition::ArrayOrCall => check_array_or_call_spread(ctx, node, argument),
            SpreadPosition::Object => check_object_spread(ctx, node, argument),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SpreadPosition {
    ArrayOrCall,
    Object,
}

fn spread_position(node: &LintNode) -> SpreadPosition {
    if node.kind == "JSXSpreadAttribute" {
        return SpreadPosition::Object;
    }
    match node.field_str("__parentKind") {
        // ObjectExpression is the ESTree object literal; JSX handled above.
        Some("ObjectExpression") => SpreadPosition::Object,
        // ArrayExpression, CallExpression, NewExpression all behave like
        // array/call spread positions in the Go rule.
        _ => SpreadPosition::ArrayOrCall,
    }
}

/// Mirrors `checkArrayOrCallSpread`: only `noStringSpread` is reported here.
fn check_array_or_call_spread(ctx: &mut RuleContext<'_>, node: &LintNode, argument: &LintNode) {
    if is_string(argument) {
        // Report on the SpreadElement node (matching Go's ReportNode(node)).
        ctx.report("noStringSpread", node.range);
    }
}

/// Mirrors `checkObjectSpread`: ordered category checks, first match wins.
fn check_object_spread(ctx: &mut RuleContext<'_>, node: &LintNode, argument: &LintNode) {
    if is_promise(argument) {
        ctx.report_with_suggestions(
            "noPromiseSpreadInObject",
            node.range,
            add_await_suggestion(argument),
        );
        return;
    }

    if is_function_without_props(argument) {
        ctx.report("noFunctionSpreadInObject", node.range);
        return;
    }

    if is_map(argument) {
        ctx.report_with_suggestions(
            "noMapSpreadInObject",
            node.range,
            map_spread_suggestion(node, argument),
        );
        return;
    }

    if is_array(argument) {
        ctx.report("noArraySpreadInObject", node.range);
        return;
    }

    // Don't report when the type is string, since TS will flag it already.
    if is_iterable(argument) && !is_string(argument) {
        ctx.report("noIterableSpreadInObject", node.range);
        return;
    }

    if is_class_instance(argument) {
        ctx.report("noClassInstanceSpreadInObject", node.range);
        return;
    }

    if is_class_declaration(argument) {
        ctx.report("noClassDeclarationSpreadInObject", node.range);
    }
}

// ----- category predicates -------------------------------------------------
//
// Each predicate mirrors a Go `is*` helper. Where the decision can be made from
// already-available type text / property facts we do so; otherwise we consume a
// single RAW checker fact (`__spread*`) that the host computes via the matching
// checker query (see newFacts). The ordering / verdict stays here in Rust.

fn is_string(argument: &LintNode) -> bool {
    if argument.field_bool("__spreadIsString") == Some(true) {
        return true;
    }
    is_string_like_type_texts(&argument.type_texts)
}

fn is_promise(argument: &LintNode) -> bool {
    argument.field_bool("__spreadIsPromise") == Some(true)
        || argument.property_names.iter().any(|name| name == "then")
}

fn is_function_without_props(argument: &LintNode) -> bool {
    argument.field_bool("__spreadIsFunctionWithoutProps") == Some(true)
}

fn is_map(argument: &LintNode) -> bool {
    argument.field_bool("__spreadIsMap") == Some(true)
}

/// Suggestion gate mirroring Go's `getMapSpreadSuggestions` union loop: every
/// `UnionTypeParts(t)` part must be a Map for the `Object.fromEntries(...)` fix
/// to be offered. The host computes this as a single raw fact; when it is absent
/// we fall back to the report-level `__spreadIsMap` (ANY part) so that the common
/// single-type-Map case still receives a suggestion.
fn all_union_parts_map(argument: &LintNode) -> bool {
    match argument.field_bool("__spreadAllUnionPartsMap") {
        Some(value) => value,
        None => argument.field_bool("__spreadIsMap") == Some(true),
    }
}

fn is_array(argument: &LintNode) -> bool {
    if argument.field_bool("__spreadIsArray") == Some(true) {
        return true;
    }
    is_array_like_type_texts(&argument.type_texts)
}

fn is_iterable(argument: &LintNode) -> bool {
    if argument.field_bool("__spreadIsIterable") == Some(true) {
        return true;
    }
    argument
        .property_names
        .iter()
        .any(|name| name == "Symbol.iterator" || name == "[Symbol.iterator]")
}

fn is_class_instance(argument: &LintNode) -> bool {
    argument.field_bool("__spreadIsClassInstance") == Some(true)
}

fn is_class_declaration(argument: &LintNode) -> bool {
    argument.field_bool("__spreadIsClassDeclaration") == Some(true)
}

/// `allow` option: the host pre-resolves whether the spread argument's type is
/// matched by a configured `TypeOrValueSpecifier` and records the raw boolean.
fn spread_is_allowed(node: &LintNode) -> bool {
    node.field_bool("__spreadAllowed") == Some(true)
}

// ----- suggestion builders -------------------------------------------------

/// Mirrors `insertAwaitFix`: wrap the (paren-skipped) argument with `await`.
fn add_await_suggestion(argument: &LintNode) -> Vec<LintSuggestion> {
    let target = skip_parentheses(argument);
    let start = target.range.start;
    let end = target.range.end;

    // `IsHigherPrecedenceThanAwait`: identifiers, member access, calls, literals
    // and already-parenthesized expressions don't need extra wrapping.
    let fixes = if is_higher_precedence_than_await(target) {
        vec![LintFix::replace_range(
            TextRange::new(start, start),
            "await ",
        )]
    } else {
        vec![
            LintFix::replace_range(TextRange::new(start, start), "await ("),
            LintFix::replace_range(TextRange::new(end, end), ")"),
        ]
    };

    vec![LintSuggestion {
        message_id: "addAwait".to_owned(),
        message: "Add await operator.".to_owned(),
        fixes,
    }]
}

/// Mirrors `getMapSpreadSuggestions`. When the spread is the sole property of an
/// object literal, replace the whole literal with `Object.fromEntries(arg)`;
/// otherwise wrap just the argument with `Object.fromEntries( ... )`.
fn map_spread_suggestion(node: &LintNode, argument: &LintNode) -> Vec<LintSuggestion> {
    // Go gates the *suggestion* (not the report) behind "every union part is a
    // Map": `for _, t := range UnionTypeParts(t) { if !isMap(t) { return [] } }`.
    // `__spreadIsMap` is the report gate (ANY part is a Map); the suggestion needs
    // the stricter ALL-parts signal. When the host has not supplied the stricter
    // fact, fall back to `__spreadIsMap` so single-type Maps still get a fix.
    if !all_union_parts_map(argument) {
        return Vec::new();
    }

    let message = "Replace map spread in object with `Object.fromEntries()`".to_owned();

    if let Some(parent_range) = sole_property_object_parent_range(node) {
        let arg_end = argument.range.end;
        // Mirror Go's getMapSpreadSuggestions sole-property branch token-for-token:
        //   1. remove the `{` token only (keeps the space that followed it),
        //   2. replace the `...` token with `Object.fromEntries(`,
        //   3. insert `)` right after the argument (properties.End() == argument.End()
        //      for a sole spread, so this is a zero-width insert, not a removal),
        //   4. remove the trailing `}` char only (keeps the space before it).
        // Go uses scanner token ranges; `{` and `}` are single ASCII chars and the
        // spread `...` token is exactly three chars, so the offsets are exact here.
        return vec![LintSuggestion {
            message_id: "replaceMapSpreadInObject".to_owned(),
            message,
            fixes: vec![
                // 1. remove `{`
                LintFix::replace_range(
                    TextRange::new(parent_range.start, parent_range.start + 1),
                    "",
                ),
                // 2. replace `...` token with the call open
                LintFix::replace_range(
                    TextRange::new(node.range.start, node.range.start + 3),
                    "Object.fromEntries(",
                ),
                // 3. close the call right after the argument (zero-width insert)
                LintFix::replace_range(TextRange::new(arg_end, arg_end), ")"),
                // 4. remove the trailing `}`
                LintFix::replace_range(TextRange::new(parent_range.end - 1, parent_range.end), ""),
            ],
        }];
    }

    let start = argument.range.start;
    let end = argument.range.end;
    vec![LintSuggestion {
        message_id: "replaceMapSpreadInObject".to_owned(),
        message,
        fixes: vec![
            LintFix::replace_range(TextRange::new(start, start), "Object.fromEntries("),
            LintFix::replace_range(TextRange::new(end, end), ")"),
        ],
    }]
}

/// Returns the range of the enclosing object literal when this spread is its
/// only property. The host records the parent object literal range plus its
/// property count as raw facts.
fn sole_property_object_parent_range(node: &LintNode) -> Option<TextRange> {
    if node.field_str("__parentKind") != Some("ObjectExpression") {
        return None;
    }
    if node.field_f64("__objectPropertyCount") != Some(1.0) {
        return None;
    }
    let start = node.field_f64("__objectStart")? as u32;
    let end = node.field_f64("__objectEnd")? as u32;
    Some(TextRange::new(start, end))
}

fn skip_parentheses(node: &LintNode) -> &LintNode {
    let mut current = strip_chain_expression(node);
    while current.kind == "TSNonNullExpression" {
        // Defensive: not paren but transparent for await precedence.
        if let Some(inner) = current.child("expression") {
            current = strip_chain_expression(inner);
        } else {
            break;
        }
    }
    current
}

fn is_higher_precedence_than_await(node: &LintNode) -> bool {
    matches!(
        node.kind.as_str(),
        "Identifier"
            | "Literal"
            | "MemberExpression"
            | "CallExpression"
            | "NewExpression"
            | "TemplateLiteral"
            | "TaggedTemplateExpression"
            | "ThisExpression"
            | "ArrayExpression"
            | "ObjectExpression"
            | "AwaitExpression"
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::super::super::{LintNode, TextRange};

    fn spread(parent_kind: &str, argument: LintNode) -> LintNode {
        let mut fields = BTreeMap::new();
        fields.insert("__parentKind".to_owned(), json!(parent_kind));
        let mut children = BTreeMap::new();
        children.insert("argument".to_owned(), argument);
        LintNode {
            kind: "SpreadElement".to_owned(),
            range: TextRange::new(0, 10),
            text: None,
            type_texts: Vec::new(),
            property_names: Vec::new(),
            fields,
            children,
            child_lists: BTreeMap::new(),
        }
    }

    fn arg(type_texts: &[&str]) -> LintNode {
        LintNode {
            kind: "Identifier".to_owned(),
            range: TextRange::new(3, 9),
            text: None,
            type_texts: type_texts.iter().map(|t| (*t).to_owned()).collect(),
            property_names: Vec::new(),
            fields: BTreeMap::new(),
            children: BTreeMap::new(),
            child_lists: BTreeMap::new(),
        }
    }

    fn run(node: &LintNode) -> Vec<super::super::super::LintDiagnostic> {
        let rule = super::NoMisusedSpreadRule;
        let mut ctx = super::super::super::RuleContext::new(&rule);
        super::RustLintRule::check(&rule, &mut ctx, node);
        ctx.finish()
    }

    #[test]
    fn reports_string_spread_in_array() {
        // const a = [...'test'];
        let node = spread("ArrayExpression", arg(&["string"]));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "noStringSpread");
    }

    #[test]
    fn reports_array_spread_in_object() {
        // const o = { ...[1, 2, 3] };
        let node = spread("ObjectExpression", arg(&["number[]"]));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "noArraySpreadInObject");
    }

    #[test]
    fn reports_map_spread_in_object_with_suggestion() {
        // declare const map: Map<string, number>; const o = { ...map };
        let mut argument = arg(&["Map<string, number>"]);
        argument
            .fields
            .insert("__spreadIsMap".to_owned(), json!(true));
        let node = spread("ObjectExpression", argument);
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "noMapSpreadInObject");
        assert_eq!(diagnostics[0].suggestions.len(), 1);
        assert_eq!(
            diagnostics[0].suggestions[0].message_id,
            "replaceMapSpreadInObject"
        );
    }

    #[test]
    fn union_mixed_map_spread_reports_without_suggestion() {
        // declare const map: Map<string, number> | { a: number }; const o = { ...map };
        // Go reports noMapSpreadInObject but offers NO suggestion because the
        // `{ a: number }` union part is not a Map.
        let mut argument = arg(&["Map<string, number> | { a: number }"]);
        argument
            .fields
            .insert("__spreadIsMap".to_owned(), json!(true));
        argument
            .fields
            .insert("__spreadAllUnionPartsMap".to_owned(), json!(false));
        let node = spread("ObjectExpression", argument);
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "noMapSpreadInObject");
        assert!(diagnostics[0].suggestions.is_empty());
    }

    #[test]
    fn reports_promise_spread_in_object_with_await_suggestion() {
        let mut argument = arg(&["Promise<{ a: number }>"]);
        argument
            .fields
            .insert("__spreadIsPromise".to_owned(), json!(true));
        let node = spread("ObjectExpression", argument);
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "noPromiseSpreadInObject");
        assert_eq!(diagnostics[0].suggestions[0].message_id, "addAwait");
    }

    #[test]
    fn sole_property_map_spread_rewrites_whole_object() {
        // `const o = { ...map };` — the spread is the only property, so the
        // suggestion rewrites the whole object literal to Object.fromEntries(map).
        // Layout: object literal `{ ...map }` at [10, 20); `...map` at [12, 18);
        // `map` at [15, 18).
        let mut argument = arg(&["Map<string, number>"]);
        argument.range = TextRange::new(15, 18);
        argument
            .fields
            .insert("__spreadIsMap".to_owned(), json!(true));
        let mut node = spread("ObjectExpression", argument);
        node.range = TextRange::new(12, 18);
        node.fields
            .insert("__objectPropertyCount".to_owned(), json!(1.0));
        node.fields.insert("__objectStart".to_owned(), json!(10.0));
        node.fields.insert("__objectEnd".to_owned(), json!(20.0));

        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "noMapSpreadInObject");
        let fixes = &diagnostics[0].suggestions[0].fixes;
        assert_eq!(fixes.len(), 4);
        // 1. remove `{`
        assert_eq!(fixes[0].range, TextRange::new(10, 11));
        assert_eq!(fixes[0].replacement_text, "");
        // 2. replace `...` token
        assert_eq!(fixes[1].range, TextRange::new(12, 15));
        assert_eq!(fixes[1].replacement_text, "Object.fromEntries(");
        // 3. close call right after `map`
        assert_eq!(fixes[2].range, TextRange::new(18, 18));
        assert_eq!(fixes[2].replacement_text, ")");
        // 4. remove trailing `}`
        assert_eq!(fixes[3].range, TextRange::new(19, 20));
        assert_eq!(fixes[3].replacement_text, "");
    }

    #[test]
    fn multi_property_map_spread_wraps_only_argument() {
        // `const o = { ...map, ...others };` — more than one property, so only
        // the argument is wrapped with Object.fromEntries( ... ).
        let mut argument = arg(&["Map<string, number>"]);
        argument.range = TextRange::new(15, 18);
        argument
            .fields
            .insert("__spreadIsMap".to_owned(), json!(true));
        let mut node = spread("ObjectExpression", argument);
        node.range = TextRange::new(12, 18);
        node.fields
            .insert("__objectPropertyCount".to_owned(), json!(2.0));
        node.fields.insert("__objectStart".to_owned(), json!(10.0));
        node.fields.insert("__objectEnd".to_owned(), json!(32.0));

        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        let fixes = &diagnostics[0].suggestions[0].fixes;
        assert_eq!(fixes.len(), 2);
        assert_eq!(fixes[0].range, TextRange::new(15, 15));
        assert_eq!(fixes[0].replacement_text, "Object.fromEntries(");
        assert_eq!(fixes[1].range, TextRange::new(18, 18));
        assert_eq!(fixes[1].replacement_text, ")");
    }

    #[test]
    fn ignores_array_spread_in_array() {
        // const a = [...[1, 2, 3]];
        let node = spread("ArrayExpression", arg(&["number[]"]));
        assert!(run(&node).is_empty());
    }

    #[test]
    fn ignores_object_spread_of_plain_object() {
        // const o = { ...{ a: 1, b: 2 } };
        let node = spread("ObjectExpression", arg(&["{ a: number; b: number }"]));
        assert!(run(&node).is_empty());
    }

    #[test]
    fn ignores_allowed_type() {
        // allow option matched by host.
        let mut argument = arg(&["string"]);
        let node = spread("ArrayExpression", {
            argument
                .fields
                .insert("__spreadIsString".to_owned(), json!(true));
            argument
        });
        // Mark the spread itself as allowed.
        let mut node = node;
        node.fields
            .insert("__spreadAllowed".to_owned(), json!(true));
        assert!(run(&node).is_empty());
    }
}
