use serde_json::Value;

use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule, TextRange};
use crate::lint::helpers::child_list;
use crate::utils::{TypeTextKind, classify_type_text, split_type_text};

/// Type-aware rule that flags template literal interpolations that are
/// unnecessary because they wrap a literal/identifier value (or are a trivial
/// single interpolation of a string-typed expression) and could be inlined.
///
/// Faithful Rust port of tsgolint `no-unnecessary-template-expression`.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoUnnecessaryTemplateExpressionRule;

const MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "noUnnecessaryTemplateExpression",
    description: "Template literal expression is unnecessary and can be simplified.",
}];

// `TemplateLiteral` is the ESTree node for `KindTemplateExpression`; the
// TS-ESTree `TSTemplateLiteralType` corresponds to `KindTemplateLiteralType`.
const LISTENERS: &[&str] = &["TemplateLiteral", "TSTemplateLiteralType"];

impl RustLintRule for NoUnnecessaryTemplateExpressionRule {
    fn name(&self) -> &'static str {
        "no-unnecessary-template-expression"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow template literals that wrap a single expression unnecessarily."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        match node.kind.as_str() {
            "TemplateLiteral" => check_template(ctx, node, /* is_type = */ false),
            "TSTemplateLiteralType" => check_template(ctx, node, /* is_type = */ true),
            _ => {}
        }
    }
}

/// Drives both the value-position template literal and the type-position
/// template literal type. `is_type` selects the expression child-list key and
/// enables the type-only enum-member guard for the trivial special case.
fn check_template(ctx: &mut RuleContext<'_>, node: &LintNode, is_type: bool) {
    // `tag` ${...}` `` is excluded from the rule entirely.
    if !is_type && is_tagged_template(node) {
        return;
    }

    let quasis = child_list(node, "quasis");
    // The expression positions live under `expressions` for value templates and
    // `types` for type templates (mirrors the host ESTree/TS-ESTree shape).
    let expressions = if is_type {
        child_list(node, "types")
    } else {
        child_list(node, "expressions")
    };
    if expressions.is_empty() || quasis.is_empty() {
        return;
    }

    // Special-case: a single trivial interpolation `${expr}` (no surrounding
    // literal text) whose underlying type is string-like is always reportable.
    if is_trivial_interpolation(node, quasis, expressions) {
        let first = &expressions[0];
        let underlying_is_string = underlying_type_is_string(first);
        let enum_member = is_enum_member_type(first);
        if underlying_is_string && (!is_type || !enum_member) {
            // reportSingleInterpolation: span from `${` to `}`.
            report_span(ctx, first);
            return;
        }
    }

    // checkTemplateSpans: iterate spans from last to first. Span i pairs
    // expressions[i] with the following quasi quasis[i + 1].
    for i in (0..expressions.len()).rev() {
        let expr = &expressions[i];
        let Some(next_quasi) = quasis.get(i + 1) else {
            continue;
        };
        if !is_unnecessary_value_interpolation(node, i, expr, next_quasi) {
            continue;
        }
        report_span(ctx, expr);
    }
}

fn is_tagged_template(node: &LintNode) -> bool {
    node.field_str("__parentKind") == Some("TaggedTemplateExpression")
}

/// Reports the `${ ... }` span around `expr`. The `${` opens two bytes before
/// the expression and the matching `}` closes one byte after it, matching the
/// upstream `prevQuasiEnd - 2 .. literalPos + 1` range.
fn report_span(ctx: &mut RuleContext<'_>, expr: &LintNode) {
    let start = expr.range.start.saturating_sub(2);
    let end = expr.range.end.saturating_add(1);
    ctx.report(
        "noUnnecessaryTemplateExpression",
        TextRange::new(start, end),
    );
}

/// Mirrors `isTrivialInterpolation`: exactly one span, an empty template head,
/// an empty first-span literal, and no comments around the interpolation.
fn is_trivial_interpolation(
    node: &LintNode,
    quasis: &[LintNode],
    expressions: &[LintNode],
) -> bool {
    if expressions.len() != 1 || quasis.len() != 2 {
        return false;
    }
    // head.Text == "" && firstSpanLiteral.Text() == ""
    quasi_cooked_text(&quasis[0]).is_empty()
        && quasi_cooked_text(&quasis[1]).is_empty()
        && !span_has_comments(node, 0)
}

/// Mirrors `isUnnecessaryValueInterpolation` for span `index` (expression +
/// following `next_quasi`).
fn is_unnecessary_value_interpolation(
    node: &LintNode,
    index: usize,
    expression: &LintNode,
    next_quasi: &LintNode,
) -> bool {
    // Comments anywhere inside the interpolation make it intentional.
    if span_has_comments(node, index) {
        return false;
    }

    // A `TSLiteralType` wraps the actual literal in `.literal`; unwrap it so the
    // literal classification below applies to type-position templates too.
    let expression = unwrap_literal_type(expression);

    if is_fixable_identifier(expression) {
        return true;
    }

    if is_string_literal_like(expression) {
        // Allow trailing whitespace literal: when the next quasi's raw text
        // starts with a newline and the interpolated string is all whitespace,
        // the interpolation is meaningful (preserves the whitespace).
        let raw = quasi_raw_text(next_quasi);
        let text = string_literal_value(expression);
        return !starts_with_newline(&raw) || !is_whitespace(&text);
    }

    is_any_literal(expression) || expression.kind == "TemplateLiteral"
}

/// `isFixableIdentifier`: `undefined`/`Infinity`/`NaN` identifiers, or the
/// `undefined` keyword node.
fn is_fixable_identifier(node: &LintNode) -> bool {
    if node.kind == "Identifier" {
        return matches!(
            node.field_str("name"),
            Some("undefined" | "Infinity" | "NaN")
        );
    }
    // TS-ESTree surfaces a bare `undefined` type as a keyword node.
    node.kind == "TSUndefinedKeyword" || node.kind == "UndefinedKeyword"
}

/// `isStringLiteralLike`: a string literal, or a template literal with no
/// substitutions (a plain `` `...` `` template).
fn is_string_literal_like(node: &LintNode) -> bool {
    if node.kind == "Literal" && node.fields.get("value").is_some_and(Value::is_string) {
        return true;
    }
    // No-substitution template literal: `` `text` `` with no expressions.
    node.kind == "TemplateLiteral" && child_list(node, "expressions").is_empty()
}

/// `isAnyLiteral`: any literal expression, a boolean literal, or `null`.
fn is_any_literal(node: &LintNode) -> bool {
    match node.kind.as_str() {
        // ESTree `Literal` covers number/string/bigint/regex/boolean/null.
        "Literal" => true,
        // TS-ESTree keyword/literal nodes that may appear in literal types.
        "TSNullKeyword" | "NullKeyword" | "TSTrueKeyword" | "TSFalseKeyword" => true,
        _ => false,
    }
}

/// Unwraps a `TSLiteralType` to its inner literal (mirrors
/// `ast.IsLiteralTypeNode(expression)` → `expression.Literal`).
fn unwrap_literal_type(node: &LintNode) -> &LintNode {
    if node.kind == "TSLiteralType" {
        if let Some(literal) = node.child("literal") {
            return literal;
        }
    }
    node
}

/// Returns the cooked text of a template element quasi, defaulting to empty.
fn quasi_cooked_text(quasi: &LintNode) -> String {
    quasi
        .fields
        .get("value")
        .and_then(|value| value.get("cooked").or_else(|| value.get("raw")))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned()
}

/// Returns the raw text of a template element quasi, defaulting to empty.
fn quasi_raw_text(quasi: &LintNode) -> String {
    quasi
        .fields
        .get("value")
        .and_then(|value| value.get("raw").or_else(|| value.get("cooked")))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned()
}

/// Returns the string value of a string literal (or the text of a
/// no-substitution template literal).
fn string_literal_value(node: &LintNode) -> String {
    if node.kind == "Literal" {
        if let Some(value) = node.field_str("value") {
            return value.to_owned();
        }
    }
    if node.kind == "TemplateLiteral" {
        if let Some(first) = child_list(node, "quasis").first() {
            return quasi_cooked_text(first);
        }
    }
    String::new()
}

fn starts_with_newline(text: &str) -> bool {
    text.starts_with('\n') || text.starts_with("\r\n")
}

/// `isWhitespace`: empty or all-whitespace (per the upstream definition that
/// also treats the empty string as whitespace).
fn is_whitespace(text: &str) -> bool {
    text.chars().all(char::is_whitespace)
}

/// Whether the span at `index` has comments around its interpolation.
///
/// The host reports a `__spanCommentIndices` array of span indices that contain
/// comments either before/after the expression or before the following quasi.
/// The verdict (whether the comment makes the span intentional) stays in Rust.
fn span_has_comments(node: &LintNode, index: usize) -> bool {
    node.fields
        .get("__spanCommentIndices")
        .and_then(Value::as_array)
        .is_some_and(|indices| {
            indices
                .iter()
                .filter_map(Value::as_u64)
                .any(|value| value as usize == index)
        })
}

/// `isUnderlyingTypeString`: every union part has some intersection part that is
/// string-like. Approximated over the host-rendered type texts of the
/// expression: every top-level union member must be a string-like type.
fn underlying_type_is_string(expression: &LintNode) -> bool {
    // Prefer an explicit raw fact when present (constraint type rendering),
    // otherwise fall back to the expression's own type texts.
    let texts: Vec<&str> = if let Some(constraint) = node_constraint_type_texts(expression) {
        constraint
    } else {
        expression.type_texts.iter().map(String::as_str).collect()
    };
    if texts.is_empty() {
        return false;
    }
    texts.iter().all(|text| {
        // Each union member: some intersection part is string-like.
        split_type_text(text)
            .iter()
            .any(|part| classify_type_text(Some(part.as_str())) == TypeTextKind::String)
    })
}

/// Reads the host-provided constraint type texts (`__constraintTypeTexts`) for a
/// template span expression, used for the trivial single-interpolation check.
fn node_constraint_type_texts(node: &LintNode) -> Option<Vec<&str>> {
    let array = node.fields.get("__constraintTypeTexts")?.as_array()?;
    Some(array.iter().filter_map(Value::as_str).collect())
}

/// `isEnumMemberType`: whether the constraint type recurses into an enum member.
/// Consumes the raw `__isEnumMemberType` fact computed by the host checker.
fn is_enum_member_type(node: &LintNode) -> bool {
    node.field_bool("__isEnumMemberType").unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::super::super::{LintNode, RuleContext, RustLintRule, TextRange};
    use super::NoUnnecessaryTemplateExpressionRule;

    fn run(node: &LintNode) -> Vec<super::super::super::LintDiagnostic> {
        let rule = NoUnnecessaryTemplateExpressionRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, node);
        ctx.finish()
    }

    fn from(value: Value) -> LintNode {
        serde_json::from_value(value).expect("valid LintNode")
    }

    fn quasi(start: u32, end: u32, cooked: &str) -> Value {
        json!({
            "kind": "TemplateElement",
            "range": { "start": start, "end": end },
            "fields": { "value": { "cooked": cooked, "raw": cooked } }
        })
    }

    // `${1}`  -> reports, range 1..5 (the `${1}` span)
    #[test]
    fn reports_number_literal_interpolation() {
        let node = from(json!({
            "kind": "TemplateLiteral",
            "range": { "start": 0, "end": 6 },
            "childLists": {
                "quasis": [quasi(1, 1, ""), quasi(5, 5, "")],
                "expressions": [{
                    "kind": "Literal",
                    "range": { "start": 3, "end": 4 },
                    "fields": { "value": 1 }
                }]
            }
        }));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "noUnnecessaryTemplateExpression");
        assert_eq!(diagnostics[0].range, TextRange::new(1, 5));
    }

    // `undefined` identifier interpolation -> reports.
    #[test]
    fn reports_undefined_identifier_interpolation() {
        let node = from(json!({
            "kind": "TemplateLiteral",
            "range": { "start": 0, "end": 14 },
            "childLists": {
                "quasis": [quasi(1, 1, ""), quasi(13, 13, "")],
                "expressions": [{
                    "kind": "Identifier",
                    "range": { "start": 3, "end": 12 },
                    "fields": { "name": "undefined" }
                }]
            }
        }));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].range, TextRange::new(1, 13));
    }

    // `use${'less'}` -> string literal with non-empty surrounding literal reports.
    #[test]
    fn reports_string_literal_in_middle() {
        let node = from(json!({
            "kind": "TemplateLiteral",
            "range": { "start": 0, "end": 13 },
            "childLists": {
                "quasis": [quasi(1, 4, "use"), quasi(12, 12, "")],
                "expressions": [{
                    "kind": "Literal",
                    "range": { "start": 6, "end": 11 },
                    "fields": { "value": "less" }
                }]
            }
        }));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
    }

    // Tagged template -> never reported.
    #[test]
    fn ignores_tagged_template() {
        let node = from(json!({
            "kind": "TemplateLiteral",
            "range": { "start": 0, "end": 6 },
            "fields": { "__parentKind": "TaggedTemplateExpression" },
            "childLists": {
                "quasis": [quasi(1, 1, ""), quasi(5, 5, "")],
                "expressions": [{
                    "kind": "Literal",
                    "range": { "start": 3, "end": 4 },
                    "fields": { "value": 1 }
                }]
            }
        }));
        assert!(run(&node).is_empty());
    }

    // `${NaN}` with a comment in the span -> NOT reported.
    #[test]
    fn ignores_interpolation_with_comment() {
        let node = from(json!({
            "kind": "TemplateLiteral",
            "range": { "start": 0, "end": 30 },
            "fields": { "__spanCommentIndices": [0] },
            "childLists": {
                "quasis": [quasi(1, 1, ""), quasi(20, 20, "")],
                "expressions": [{
                    "kind": "Identifier",
                    "range": { "start": 16, "end": 19 },
                    "fields": { "name": "NaN" }
                }]
            }
        }));
        assert!(run(&node).is_empty());
    }

    // `${number}` where number: 1 (string-NOT-like) trivial interpolation -> NOT reported.
    #[test]
    fn ignores_non_string_trivial_interpolation() {
        let node = from(json!({
            "kind": "TemplateLiteral",
            "range": { "start": 0, "end": 11 },
            "childLists": {
                "quasis": [quasi(1, 1, ""), quasi(10, 10, "")],
                "expressions": [{
                    "kind": "Identifier",
                    "range": { "start": 3, "end": 9 },
                    "typeTexts": ["1"],
                    "fields": { "name": "number" }
                }]
            }
        }));
        assert!(run(&node).is_empty());
    }

    // `${string}` where string: 'a' (string-like) trivial interpolation -> reports.
    #[test]
    fn reports_string_typed_trivial_interpolation() {
        let node = from(json!({
            "kind": "TemplateLiteral",
            "range": { "start": 0, "end": 11 },
            "childLists": {
                "quasis": [quasi(1, 1, ""), quasi(10, 10, "")],
                "expressions": [{
                    "kind": "Identifier",
                    "range": { "start": 3, "end": 9 },
                    "typeTexts": ["'a'"],
                    "fields": { "name": "string" }
                }]
            }
        }));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].range, TextRange::new(1, 10));
    }

    // Trailing whitespace literal: ` ${'    '}\n ` -> NOT reported (whitespace + newline quasi).
    #[test]
    fn ignores_trailing_whitespace_literal() {
        let node = from(json!({
            "kind": "TemplateLiteral",
            "range": { "start": 0, "end": 20 },
            "childLists": {
                // head has non-empty text so this is not trivial; next quasi starts with newline
                "quasis": [quasi(1, 2, " "), quasi(11, 13, "\n ")],
                "expressions": [{
                    "kind": "Literal",
                    "range": { "start": 4, "end": 10 },
                    "fields": { "value": "    " }
                }]
            }
        }));
        assert!(run(&node).is_empty());
    }

    // `${1}` as a template literal TYPE -> reports (TSTemplateLiteralType + types).
    #[test]
    fn reports_template_literal_type_number() {
        let node = from(json!({
            "kind": "TSTemplateLiteralType",
            "range": { "start": 0, "end": 6 },
            "childLists": {
                "quasis": [quasi(1, 1, ""), quasi(5, 5, "")],
                "types": [{
                    "kind": "TSLiteralType",
                    "range": { "start": 3, "end": 4 },
                    "children": {
                        "literal": {
                            "kind": "Literal",
                            "range": { "start": 3, "end": 4 },
                            "fields": { "value": 1 }
                        }
                    }
                }]
            }
        }));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].range, TextRange::new(1, 5));
    }

    // Enum-member typed trivial interpolation in a type -> NOT reported.
    #[test]
    fn ignores_enum_member_type_trivial_interpolation() {
        let node = from(json!({
            "kind": "TSTemplateLiteralType",
            "range": { "start": 0, "end": 11 },
            "childLists": {
                "quasis": [quasi(1, 1, ""), quasi(10, 10, "")],
                "types": [{
                    "kind": "TSTypeReference",
                    "range": { "start": 3, "end": 9 },
                    "typeTexts": ["Foo.A"],
                    "fields": { "__constraintTypeTexts": ["'A'"], "__isEnumMemberType": true }
                }]
            }
        }));
        assert!(run(&node).is_empty());
    }
}
