use super::super::{
    LintFix, LintNode, LintSuggestion, RuleContext, RuleMessage, RustLintRule, TextRange,
};
use crate::lint::helpers::{child_list, identifier_name, member_object, member_property_name};
use crate::utils::{TypeTextKind, classify_type_text, split_type_text};

/// Type-aware rule that flags conversions which do not change a value's type.
///
/// Faithful port of tsgolint `no-unnecessary-type-conversion`. The rule reports:
/// - `String(x)` / `Number(x)` / `Boolean(x)` / `BigInt(x)` when `x` already has
///   the matching primitive type and the callee is the global builtin.
/// - `x.toString()` when `x` is already string-like (and not an enum / enum member).
/// - `x + ''` / `'' + x` and `x += ''` when `x` is string-like.
/// - unary `+x` when `x` is number-like.
/// - `!!x` when `x` is boolean-like.
/// - `~~x` when `x` is made entirely of integer number literals.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoUnnecessaryTypeConversionRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "unnecessaryTypeConversion",
        description: "This type conversion does not change the type or value of the expression.",
    },
    RuleMessage {
        id: "suggestRemove",
        description: "Remove the type conversion.",
    },
    RuleMessage {
        id: "suggestSatisfies",
        description: "Instead, assert that the value satisfies the primitive type.",
    },
];

// The Go rule listens to BinaryExpression / CallExpression / PrefixUnaryExpression.
// In TS-ESTree, `+=` is an AssignmentExpression (not a BinaryExpression), and `+`,
// `!`, `~` unary operators are UnaryExpression nodes, so we listen to those kinds.
const LISTENERS: &[&str] = &[
    "BinaryExpression",
    "AssignmentExpression",
    "CallExpression",
    "UnaryExpression",
];

impl RustLintRule for NoUnnecessaryTypeConversionRule {
    fn name(&self) -> &'static str {
        "no-unnecessary-type-conversion"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow conversions that do not change the type or value of the expression."
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
        match node.kind.as_str() {
            "BinaryExpression" => report_binary_plus(ctx, node),
            "AssignmentExpression" => report_plus_equals(ctx, node),
            "CallExpression" => {
                report_builtin_conversion_call(ctx, node);
                report_string_to_string_call(ctx, node);
            }
            "UnaryExpression" => {
                report_unary_plus(ctx, node);
                report_double_bang(ctx, node);
                report_double_tilde(ctx, node);
            }
            _ => {}
        }
    }
}

/// Mirrors Go `doesUnderlyingTypeMatchFlag`: every union part of the operand
/// type must match the wanted coarse kind. The host bridge ships rendered
/// type texts, sourced from the TSGO checker's `typeToString`, rather than
/// the raw `Type.flags` bitmask; we classify each rendered atom via the
/// shared `classify_type_text` helper so this rule and the other type-aware
/// rules stay in lock-step with the upstream classifier.
fn underlying_type_matches(type_texts: &[String], wanted: TypeTextKind) -> bool {
    if type_texts.is_empty() {
        return false;
    }
    // A node carries the rendered type as a single text (e.g. `"string"` or
    // `"string | number"`). Treat each rendered text independently; if any
    // rendered text is fully composed of `wanted` union parts, it matches.
    type_texts.iter().any(|text| {
        let parts = split_type_text(text);
        !parts.is_empty()
            && parts
                .iter()
                .all(|part| classify_type_text(Some(part.as_str())) == wanted)
    })
}

/// Mirrors Go `isAllNumberLiteralIntegers`: every union part is an integer numeric
/// literal type (e.g. `3`, `3 | 4`). Non-integer / non-literal parts disqualify.
fn is_all_number_literal_integers(type_texts: &[String]) -> bool {
    if type_texts.is_empty() {
        return false;
    }
    type_texts.iter().any(|text| {
        let parts = split_type_text(text);
        !parts.is_empty()
            && parts.iter().all(|part| {
                let part = part.trim();
                match part.parse::<f64>() {
                    Ok(value) => value.fract() == 0.0,
                    Err(_) => false,
                }
            })
    })
}

fn is_empty_string_literal(node: &LintNode) -> bool {
    node.kind == "Literal" && node.field_str("value") == Some("")
}

/// The single positional argument of a call, rejecting spread and extra-args
/// shapes (Go bails when `len(Arguments) != 1` or the single arg is a spread).
fn single_call_argument(node: &LintNode) -> Option<&LintNode> {
    let args = child_list(node, "arguments");
    if args.len() != 1 {
        return None;
    }
    let arg = &args[0];
    if arg.kind == "SpreadElement" {
        return None;
    }
    Some(arg)
}

// ---------------------------------------------------------------------------
// Suggestion fix construction
// ---------------------------------------------------------------------------

/// Returns whether the inner expression binds tightly enough that it needs no
/// parentheses when extracted. Mirrors Go `utils.IsStrongPrecedenceNode` for the
/// node kinds we can observe from facts; defaults to "needs parens" when unknown.
fn is_strong_precedence(node: &LintNode) -> bool {
    // Mirrors Go `utils.IsStrongPrecedenceNode` restricted to the ESTree node kinds
    // that can appear as a conversion operand. Go's list: literals (incl. boolean),
    // parenthesized expressions, Identifier, ArrayLiteral, ObjectLiteral,
    // PropertyAccess/ElementAccess (`MemberExpression`), Call, New, TaggedTemplate.
    // (TypeReference / TypeOperator / ExpressionWithTypeArguments are type-level and
    // never reached here.) Anything else defaults to "needs parens", matching Go.
    matches!(
        node.kind.as_str(),
        "Identifier"
            | "Literal"
            | "MemberExpression"
            | "CallExpression"
            | "NewExpression"
            | "ArrayExpression"
            | "ObjectExpression"
            | "TaggedTemplateExpression"
    )
}

/// Renders the inner node's source text, parenthesizing weak-precedence nodes so
/// the resulting fix keeps the original meaning. Returns `None` when the host did
/// not provide inline text for the node (in which case we still report, but emit
/// no suggestions for that case).
fn inner_code(node: &LintNode) -> Option<String> {
    let text = node.text.as_ref()?;
    if is_strong_precedence(node) {
        Some(text.clone())
    } else {
        Some(format!("({text})"))
    }
}

/// Whether `node` sits in a parent context whose precedence is weaker than the
/// replacement, requiring the rewritten code to be wrapped in parens. Approximated
/// from `__parentKind` (the host fact recording the parent ESTree node kind).
fn is_weak_precedence_parent(node: &LintNode) -> bool {
    matches!(
        node.field_str("__parentKind"),
        Some(
            "UnaryExpression"
                | "UpdateExpression"
                | "BinaryExpression"
                | "LogicalExpression"
                | "ConditionalExpression"
                | "AwaitExpression"
                | "MemberExpression"
                | "CallExpression"
                | "NewExpression"
                | "TaggedTemplateExpression"
        )
    )
}

fn is_node_parenthesized(node: &LintNode) -> bool {
    // TS-ESTree does not expose ParenthesizedExpression by default; the host marks
    // it via `__parenthesized` when the conversion node is directly wrapped.
    node.field_bool("__parenthesized") == Some(true)
}

/// Builds the two suggestions (`suggestRemove` and `suggestSatisfies`) that strip
/// the conversion, replacing the whole `outer` range with the inner code, mirroring
/// Go `buildSuggestions` + `buildWrappingFix`.
fn build_suggestions(
    outer: &LintNode,
    primitive_type: &str,
    inner: &LintNode,
) -> Vec<LintSuggestion> {
    let Some(code) = inner_code(inner) else {
        return Vec::new();
    };

    // suggestRemove: replace the conversion with the bare inner code.
    let remove = LintSuggestion {
        message_id: "suggestRemove".to_owned(),
        message: "Remove the type conversion.".to_owned(),
        fixes: vec![LintFix::replace_range(outer.range, code.clone())],
    };

    // suggestSatisfies: replace with `<code> satisfies <primitive>`, wrapping the
    // whole thing in parens when the parent binds tighter and it is not already
    // parenthesized.
    let mut satisfies_code = format!("{code} satisfies {primitive_type}");
    if is_weak_precedence_parent(outer) && !is_node_parenthesized(outer) {
        satisfies_code = format!("({satisfies_code})");
    }
    let satisfies = LintSuggestion {
        message_id: "suggestSatisfies".to_owned(),
        message: "Instead, assert that the value satisfies the primitive type.".to_owned(),
        fixes: vec![LintFix::replace_range(outer.range, satisfies_code)],
    };

    vec![remove, satisfies]
}

// ---------------------------------------------------------------------------
// BinaryExpression: `x + ''` and `'' + x`
// ---------------------------------------------------------------------------

fn report_binary_plus(ctx: &mut RuleContext<'_>, node: &LintNode) {
    if node.field_str("operator") != Some("+") {
        return;
    }
    let (Some(left), Some(right)) = (node.child("left"), node.child("right")) else {
        return;
    };

    if is_empty_string_literal(right) {
        if underlying_type_matches(&left.type_texts, TypeTextKind::String) {
            // Conversion range covers `left.end .. node.end` (the ` + ''` tail).
            let range = TextRange::new(left.range.end, node.range.end);
            ctx.report_with_suggestions(
                "unnecessaryTypeConversion",
                range,
                build_suggestions(node, "string", left),
            );
        }
        return;
    }

    if is_empty_string_literal(left)
        && underlying_type_matches(&right.type_texts, TypeTextKind::String)
    {
        // Conversion range covers `node.start .. right.start` (the `'' + ` head).
        let range = TextRange::new(node.range.start, right.range.start);
        ctx.report_with_suggestions(
            "unnecessaryTypeConversion",
            range,
            build_suggestions(node, "string", right),
        );
    }
}

// ---------------------------------------------------------------------------
// AssignmentExpression: `x += ''`
// ---------------------------------------------------------------------------

fn report_plus_equals(ctx: &mut RuleContext<'_>, node: &LintNode) {
    if node.field_str("operator") != Some("+=") {
        return;
    }
    let (Some(left), Some(right)) = (node.child("left"), node.child("right")) else {
        return;
    };
    if !is_empty_string_literal(right) {
        return;
    }
    // Go requires the assignment target to be an identifier.
    if left.kind != "Identifier" {
        return;
    }
    if !underlying_type_matches(&left.type_texts, TypeTextKind::String) {
        return;
    }

    let suggestions = build_plus_equals_suggestions(node, left);
    ctx.report_with_suggestions("unnecessaryTypeConversion", node.range, suggestions);
}

fn build_plus_equals_suggestions(node: &LintNode, left: &LintNode) -> Vec<LintSuggestion> {
    let Some(code) = inner_code(left) else {
        return Vec::new();
    };

    // suggestRemove: if the assignment is the whole statement, remove the statement;
    // otherwise replace the assignment with the bare target code.
    let remove_fix = if node.field_str("__parentKind") == Some("ExpressionStatement") {
        // Remove the entire statement range when available, else fall back to the
        // assignment node range. `__statementRange` is the parent statement bounds.
        match statement_range(node) {
            Some(range) => LintFix::remove_range(range),
            None => LintFix::replace_range(node.range, code.clone()),
        }
    } else {
        LintFix::replace_range(node.range, code.clone())
    };

    vec![
        LintSuggestion {
            message_id: "suggestRemove".to_owned(),
            message: "Remove the type conversion.".to_owned(),
            fixes: vec![remove_fix],
        },
        LintSuggestion {
            message_id: "suggestSatisfies".to_owned(),
            message: "Instead, assert that the value satisfies the primitive type.".to_owned(),
            fixes: vec![LintFix::replace_range(
                node.range,
                format!("{code} satisfies string"),
            )],
        },
    ]
}

/// Parent statement byte range, provided by the host as `__statementRange`
/// `[start, end]` when the assignment's parent is an ExpressionStatement.
fn statement_range(node: &LintNode) -> Option<TextRange> {
    let arr = node.fields.get("__statementRange")?.as_array()?;
    let start = arr.first()?.as_u64()? as u32;
    let end = arr.get(1)?.as_u64()? as u32;
    Some(TextRange::new(start, end))
}

// ---------------------------------------------------------------------------
// CallExpression: `String(x)` / `Number(x)` / `Boolean(x)` / `BigInt(x)`
// ---------------------------------------------------------------------------

fn report_builtin_conversion_call(ctx: &mut RuleContext<'_>, node: &LintNode) {
    let Some(callee) = node.child("callee") else {
        return;
    };
    let Some(name) = identifier_name(callee) else {
        return;
    };
    let Some(arg) = single_call_argument(node) else {
        return;
    };

    let (wanted, primitive) = match name.as_str() {
        "BigInt" => (TypeTextKind::Bigint, "bigint"),
        "Boolean" => (TypeTextKind::Boolean, "boolean"),
        "Number" => (TypeTextKind::Number, "number"),
        "String" => (TypeTextKind::String, "string"),
        _ => return,
    };

    // The host may explicitly mark a shadowing local. When no symbol fact is
    // available, keep linting the known global constructor names instead of
    // silently dropping syntax-only cases like `String('value')`.
    if node.field_bool("__calleeIsGlobalBuiltin") == Some(false) {
        return;
    }

    if !underlying_type_matches(&arg.type_texts, wanted) {
        return;
    }

    // Conversion range is the callee identifier itself (Go uses callee text range).
    ctx.report_with_suggestions(
        "unnecessaryTypeConversion",
        callee.range,
        build_suggestions(node, primitive, arg),
    );
}

// ---------------------------------------------------------------------------
// CallExpression: `x.toString()`
// ---------------------------------------------------------------------------

fn report_string_to_string_call(ctx: &mut RuleContext<'_>, node: &LintNode) {
    let Some(callee) = node.child("callee") else {
        return;
    };
    // Must be a member call with zero arguments.
    if !child_list(node, "arguments").is_empty() {
        return;
    }
    if member_property_name(callee).as_deref() != Some("toString") {
        return;
    }
    let Some(object) = member_object(callee) else {
        return;
    };

    // Enum types / enum members keep their distinct value semantics; skip them.
    if object.field_bool("__objectTypeIsEnumLike") == Some(true) {
        return;
    }

    if !underlying_type_matches(&object.type_texts, TypeTextKind::String) {
        return;
    }

    // Conversion range covers `.toString()` i.e. property name start to call end.
    let Some(property) = callee.child("property") else {
        return;
    };
    let range = TextRange::new(property.range.start, node.range.end);
    ctx.report_with_suggestions(
        "unnecessaryTypeConversion",
        range,
        build_suggestions(node, "string", object),
    );
}

// ---------------------------------------------------------------------------
// UnaryExpression: `+x`
// ---------------------------------------------------------------------------

fn report_unary_plus(ctx: &mut RuleContext<'_>, node: &LintNode) {
    if node.field_str("operator") != Some("+") || node.field_bool("prefix") == Some(false) {
        return;
    }
    let Some(operand) = node.child("argument") else {
        return;
    };
    if !underlying_type_matches(&operand.type_texts, TypeTextKind::Number) {
        return;
    }

    // Conversion range is the `+` token: node start to operand start.
    let range = TextRange::new(node.range.start, operand.range.start);
    ctx.report_with_suggestions(
        "unnecessaryTypeConversion",
        range,
        build_suggestions(node, "number", operand),
    );
}

// ---------------------------------------------------------------------------
// UnaryExpression: `!!x` (this node is the inner `!`, parent is the outer `!`)
// ---------------------------------------------------------------------------

fn report_double_bang(ctx: &mut RuleContext<'_>, node: &LintNode) {
    if node.field_str("operator") != Some("!") {
        return;
    }
    // The parent must be another `!` whose operand is this node. The host marks the
    // inner node of a `!!` pair with `__doubleNegation = true` and provides the
    // outer node range via `__outerRange`.
    if node.field_bool("__doubleNegation") != Some(true) {
        return;
    }
    let Some(operand) = node.child("argument") else {
        return;
    };
    if !underlying_type_matches(&operand.type_texts, TypeTextKind::Boolean) {
        return;
    }

    let Some(outer_range) = outer_range(node) else {
        return;
    };
    // Range covers both `!` tokens: outer start to inner start + 1.
    let range = TextRange::new(outer_range.start, node.range.start + 1);
    ctx.report_with_suggestions(
        "unnecessaryTypeConversion",
        range,
        build_double_suggestions(outer_range, "boolean", operand),
    );
}

// ---------------------------------------------------------------------------
// UnaryExpression: `~~x` (this node is the inner `~`, parent is the outer `~`)
// ---------------------------------------------------------------------------

fn report_double_tilde(ctx: &mut RuleContext<'_>, node: &LintNode) {
    if node.field_str("operator") != Some("~") {
        return;
    }
    if node.field_bool("__doubleNegation") != Some(true) {
        return;
    }
    let Some(operand) = node.child("argument") else {
        return;
    };
    if !is_all_number_literal_integers(&operand.type_texts) {
        return;
    }

    let Some(outer_range) = outer_range(node) else {
        return;
    };
    let range = TextRange::new(outer_range.start, node.range.start + 1);
    ctx.report_with_suggestions(
        "unnecessaryTypeConversion",
        range,
        build_double_suggestions(outer_range, "number", operand),
    );
}

/// Outer (parent) node byte range for a `!!` / `~~` pair, provided by the host as
/// `__outerRange` `[start, end]`.
fn outer_range(node: &LintNode) -> Option<TextRange> {
    let arr = node.fields.get("__outerRange")?.as_array()?;
    let start = arr.first()?.as_u64()? as u32;
    let end = arr.get(1)?.as_u64()? as u32;
    Some(TextRange::new(start, end))
}

/// Like `build_suggestions`, but the replaced range is the outer `!!` / `~~` node.
fn build_double_suggestions(
    outer_range: TextRange,
    primitive_type: &str,
    inner: &LintNode,
) -> Vec<LintSuggestion> {
    let Some(code) = inner_code(inner) else {
        return Vec::new();
    };
    vec![
        LintSuggestion {
            message_id: "suggestRemove".to_owned(),
            message: "Remove the type conversion.".to_owned(),
            fixes: vec![LintFix::replace_range(outer_range, code.clone())],
        },
        LintSuggestion {
            message_id: "suggestSatisfies".to_owned(),
            message: "Instead, assert that the value satisfies the primitive type.".to_owned(),
            fixes: vec![LintFix::replace_range(
                outer_range,
                format!("{code} satisfies {primitive_type}"),
            )],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::super::super::{LintNode, RuleContext, RustLintRule};
    use super::NoUnnecessaryTypeConversionRule;
    use serde_json::json;

    fn run(node: &LintNode) -> Vec<super::super::super::LintDiagnostic> {
        let rule = NoUnnecessaryTypeConversionRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, node);
        ctx.finish()
    }

    fn from(value: serde_json::Value) -> LintNode {
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn reports_string_call_on_string_arg() {
        // String('asdf');  -> callee range [0,6]
        let node = from(json!({
            "kind": "CallExpression",
            "range": { "start": 0, "end": 14 },
            "fields": { "__calleeIsGlobalBuiltin": true },
            "children": {
                "callee": {
                    "kind": "Identifier",
                    "range": { "start": 0, "end": 6 },
                    "fields": { "name": "String" },
                    "text": "String"
                }
            },
            "childLists": {
                "arguments": [
                    {
                        "kind": "Literal",
                        "range": { "start": 7, "end": 13 },
                        "typeTexts": ["string"],
                        "text": "'asdf'"
                    }
                ]
            }
        }));
        let diags = run(&node);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].message_id, "unnecessaryTypeConversion");
        assert_eq!(diags[0].range, super::TextRange::new(0, 6));
        assert_eq!(diags[0].suggestions.len(), 2);
        assert_eq!(diags[0].suggestions[0].message_id, "suggestRemove");
        assert_eq!(diags[0].suggestions[0].fixes[0].replacement_text, "'asdf'");
        assert_eq!(
            diags[0].suggestions[1].fixes[0].replacement_text,
            "'asdf' satisfies string"
        );
    }

    #[test]
    fn reports_unary_plus_on_number() {
        // +x  where x: number; range of `+` is [0,1]
        let node = from(json!({
            "kind": "UnaryExpression",
            "range": { "start": 0, "end": 2 },
            "fields": { "operator": "+", "prefix": true },
            "children": {
                "argument": {
                    "kind": "Identifier",
                    "range": { "start": 1, "end": 2 },
                    "typeTexts": ["number"],
                    "text": "x"
                }
            }
        }));
        let diags = run(&node);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].range, super::TextRange::new(0, 1));
        assert_eq!(diags[0].suggestions[0].fixes[0].replacement_text, "x");
    }

    #[test]
    fn reports_binary_plus_empty_string_right() {
        // x + '' where x: string
        let node = from(json!({
            "kind": "BinaryExpression",
            "range": { "start": 0, "end": 6 },
            "fields": { "operator": "+" },
            "children": {
                "left": {
                    "kind": "Identifier",
                    "range": { "start": 0, "end": 1 },
                    "typeTexts": ["string"],
                    "text": "x"
                },
                "right": {
                    "kind": "Literal",
                    "range": { "start": 4, "end": 6 },
                    "fields": { "value": "" }
                }
            }
        }));
        let diags = run(&node);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].range, super::TextRange::new(1, 6));
    }

    #[test]
    fn ignores_number_call_on_string_arg() {
        // Number('2'); arg is string, not number -> no report
        let node = from(json!({
            "kind": "CallExpression",
            "range": { "start": 0, "end": 11 },
            "fields": { "__calleeIsGlobalBuiltin": true },
            "children": {
                "callee": {
                    "kind": "Identifier",
                    "range": { "start": 0, "end": 6 },
                    "fields": { "name": "Number" },
                    "text": "Number"
                }
            },
            "childLists": {
                "arguments": [
                    {
                        "kind": "Literal",
                        "range": { "start": 7, "end": 10 },
                        "typeTexts": ["string"],
                        "text": "'2'"
                    }
                ]
            }
        }));
        assert!(run(&node).is_empty());
    }

    #[test]
    fn ignores_shadowed_string_function() {
        // function String(...) {} ; String('asdf'); callee not global builtin
        let node = from(json!({
            "kind": "CallExpression",
            "range": { "start": 0, "end": 14 },
            "fields": { "__calleeIsGlobalBuiltin": false },
            "children": {
                "callee": {
                    "kind": "Identifier",
                    "range": { "start": 0, "end": 6 },
                    "fields": { "name": "String" },
                    "text": "String"
                }
            },
            "childLists": {
                "arguments": [
                    {
                        "kind": "Literal",
                        "range": { "start": 7, "end": 13 },
                        "typeTexts": ["string"],
                        "text": "'asdf'"
                    }
                ]
            }
        }));
        assert!(run(&node).is_empty());
    }

    #[test]
    fn reports_string_call_when_builtin_fact_is_absent() {
        let node = from(json!({
            "kind": "CallExpression",
            "range": { "start": 0, "end": 14 },
            "children": {
                "callee": {
                    "kind": "Identifier",
                    "range": { "start": 0, "end": 6 },
                    "fields": { "name": "String" },
                    "text": "String"
                }
            },
            "childLists": {
                "arguments": [
                    {
                        "kind": "Literal",
                        "range": { "start": 7, "end": 13 },
                        "typeTexts": ["string"],
                        "text": "'asdf'"
                    }
                ]
            }
        }));
        let diags = run(&node);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].message_id, "unnecessaryTypeConversion");
    }

    #[test]
    fn ignores_string_call_with_extra_arg() {
        // String('asdf', console.log('extra')); two args -> no report
        let node = from(json!({
            "kind": "CallExpression",
            "range": { "start": 0, "end": 36 },
            "fields": { "__calleeIsGlobalBuiltin": true },
            "children": {
                "callee": {
                    "kind": "Identifier",
                    "range": { "start": 0, "end": 6 },
                    "fields": { "name": "String" },
                    "text": "String"
                }
            },
            "childLists": {
                "arguments": [
                    { "kind": "Literal", "range": { "start": 7, "end": 13 }, "typeTexts": ["string"] },
                    { "kind": "CallExpression", "range": { "start": 15, "end": 34 } }
                ]
            }
        }));
        assert!(run(&node).is_empty());
    }

    #[test]
    fn ignores_double_tilde_non_integer() {
        // ~~1.1 -> operand type 1.1 (non-integer literal) -> no report
        let node = from(json!({
            "kind": "UnaryExpression",
            "range": { "start": 1, "end": 5 },
            "fields": { "operator": "~", "prefix": true, "__doubleNegation": true,
                        "__outerRange": [0, 5] },
            "children": {
                "argument": {
                    "kind": "Literal",
                    "range": { "start": 2, "end": 5 },
                    "typeTexts": ["1.1"],
                    "text": "1.1"
                }
            }
        }));
        assert!(run(&node).is_empty());
    }

    #[test]
    fn reports_binary_plus_empty_string_left() {
        // '' + x where x: string ; conversion range is node.start .. right.start
        let node = from(json!({
            "kind": "BinaryExpression",
            "range": { "start": 0, "end": 6 },
            "fields": { "operator": "+" },
            "children": {
                "left": {
                    "kind": "Literal",
                    "range": { "start": 0, "end": 2 },
                    "fields": { "value": "" }
                },
                "right": {
                    "kind": "Identifier",
                    "range": { "start": 5, "end": 6 },
                    "typeTexts": ["string"],
                    "text": "x"
                }
            }
        }));
        let diags = run(&node);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].range, super::TextRange::new(0, 5));
        assert_eq!(diags[0].suggestions[0].fixes[0].replacement_text, "x");
    }

    #[test]
    fn satisfies_wraps_when_parent_binds_tighter() {
        // String(x).length where x: string ; the conversion node's parent is a
        // MemberExpression, so the satisfies suggestion must wrap in parens and the
        // strong-precedence Identifier inner must NOT be parenthesized.
        let node = from(json!({
            "kind": "CallExpression",
            "range": { "start": 0, "end": 9 },
            "fields": { "__calleeIsGlobalBuiltin": true, "__parentKind": "MemberExpression" },
            "children": {
                "callee": {
                    "kind": "Identifier",
                    "range": { "start": 0, "end": 6 },
                    "fields": { "name": "String" },
                    "text": "String"
                }
            },
            "childLists": {
                "arguments": [
                    {
                        "kind": "Identifier",
                        "range": { "start": 7, "end": 8 },
                        "typeTexts": ["string"],
                        "text": "x"
                    }
                ]
            }
        }));
        let diags = run(&node);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].suggestions[0].fixes[0].replacement_text, "x");
        assert_eq!(
            diags[0].suggestions[1].fixes[0].replacement_text,
            "(x satisfies string)"
        );
    }

    #[test]
    fn double_bang_reports_on_inner_node() {
        // !!x where x: boolean ; outer range [0,3], inner `!` at [1,3].
        let node = from(json!({
            "kind": "UnaryExpression",
            "range": { "start": 1, "end": 3 },
            "fields": { "operator": "!", "prefix": true, "__doubleNegation": true,
                        "__outerRange": [0, 3] },
            "children": {
                "argument": {
                    "kind": "Identifier",
                    "range": { "start": 2, "end": 3 },
                    "typeTexts": ["boolean"],
                    "text": "x"
                }
            }
        }));
        let diags = run(&node);
        assert_eq!(diags.len(), 1);
        // both `!` tokens: outer.start(0) .. inner.start+1 (2)
        assert_eq!(diags[0].range, super::TextRange::new(0, 2));
        assert_eq!(diags[0].suggestions[0].fixes[0].replacement_text, "x");
        assert_eq!(
            diags[0].suggestions[1].fixes[0].replacement_text,
            "x satisfies boolean"
        );
    }

    #[test]
    fn outer_double_negation_node_does_not_report() {
        // The OUTER `!` of a `!!` pair has no __doubleNegation flag -> no report,
        // so the diagnostic is emitted only once (on the inner node).
        let node = from(json!({
            "kind": "UnaryExpression",
            "range": { "start": 0, "end": 3 },
            "fields": { "operator": "!", "prefix": true },
            "children": {
                "argument": {
                    "kind": "UnaryExpression",
                    "range": { "start": 1, "end": 3 },
                    "fields": { "operator": "!", "prefix": true }
                }
            }
        }));
        assert!(run(&node).is_empty());
    }

    #[test]
    fn reports_double_tilde_integer_literals() {
        // ~~x where x: 3 | 4 ; outer range [0,5], inner `~` at [1,5]
        let node = from(json!({
            "kind": "UnaryExpression",
            "range": { "start": 1, "end": 5 },
            "fields": { "operator": "~", "prefix": true, "__doubleNegation": true,
                        "__outerRange": [0, 5] },
            "children": {
                "argument": {
                    "kind": "Identifier",
                    "range": { "start": 2, "end": 5 },
                    "typeTexts": ["3 | 4"],
                    "text": "x"
                }
            }
        }));
        let diags = run(&node);
        assert_eq!(diags.len(), 1);
        // both `~` tokens: outer.start(0) .. inner.start+1 (2)
        assert_eq!(diags[0].range, super::TextRange::new(0, 2));
        assert_eq!(
            diags[0].suggestions[1].fixes[0].replacement_text,
            "x satisfies number"
        );
    }
}
