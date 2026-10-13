use serde_json::Value;

use super::super::{LintFix, LintNode, LintSuggestion, RuleContext, RuleMessage, RustLintRule};
use crate::lint::helpers::rule_option_bool;

/// Type-aware rule that prefers dot notation over computed bracket access and
/// flags reserved-word property access that is a syntax error in ES3.
///
/// This is a faithful port of the tsgolint `dot-notation` builtin rule. The
/// rule has two arms:
///
/// * Computed `MemberExpression` (ESTree `computed: true`, equivalent to the Go
///   `ElementAccessExpression` listener) is reported with `useDot` when the
///   property is a string / template / boolean / null literal whose value is a
///   valid identifier (subject to the keyword / pattern / type-aware allowances).
/// * Non-computed `MemberExpression` (the Go `PropertyAccessExpression`
///   listener) is reported with `useBrackets` when the accessed identifier is a
///   reserved keyword and `allowKeywords` is `false`.
#[derive(Clone, Copy, Debug, Default)]
pub struct DotNotationRule;

const MESSAGES: &[RuleMessage] = &[
    RuleMessage {
        id: "useDot",
        description: "[{{key}}] is better written in dot notation.",
    },
    RuleMessage {
        id: "useBrackets",
        description: ".{{key}} is a syntax error.",
    },
];

const LISTENERS: &[&str] = &["MemberExpression"];

/// ES3 reserved words, matching `es3Keywords` in the Go source.
const ES3_KEYWORDS: &[&str] = &[
    "abstract",
    "boolean",
    "break",
    "byte",
    "case",
    "catch",
    "char",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "double",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "final",
    "finally",
    "float",
    "for",
    "function",
    "goto",
    "if",
    "implements",
    "import",
    "in",
    "instanceof",
    "int",
    "interface",
    "long",
    "native",
    "new",
    "null",
    "package",
    "private",
    "protected",
    "public",
    "return",
    "short",
    "static",
    "super",
    "switch",
    "synchronized",
    "this",
    "throw",
    "throws",
    "transient",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "volatile",
    "while",
    "with",
];

impl RustLintRule for DotNotationRule {
    fn name(&self) -> &'static str {
        "dot-notation"
    }

    fn docs_description(&self) -> &'static str {
        "Enforce dot notation whenever possible."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn has_suggestions(&self) -> bool {
        // Reported as auto-fixes by the host; modeled here as suggestions so the
        // fix edits flow through the suggestion channel.
        true
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        if node.kind != "MemberExpression" {
            return;
        }
        if node.field_bool("computed").unwrap_or(false) {
            report_computed_property(ctx, node);
        } else {
            report_dot_keyword_access(ctx, node);
        }
    }
}

/// `allowKeywords` defaults to `true` in the upstream options.
fn allow_keywords(node: &LintNode) -> bool {
    rule_option_bool(node, "allowKeywords").unwrap_or(true)
}

fn allow_private_class_property_access(node: &LintNode) -> bool {
    rule_option_bool(node, "allowPrivateClassPropertyAccess").unwrap_or(false)
}

fn allow_protected_class_property_access(node: &LintNode) -> bool {
    rule_option_bool(node, "allowProtectedClassPropertyAccess").unwrap_or(false)
}

/// Honors both the explicit `allowIndexSignaturePropertyAccess` option and the
/// `noPropertyAccessFromIndexSignature` compiler option, exactly as the Go rule
/// folds them together.
fn allow_index_signature_property_access(node: &LintNode) -> bool {
    rule_option_bool(node, "allowIndexSignaturePropertyAccess").unwrap_or(false)
        || node
            .field_bool("__noPropertyAccessFromIndexSignature")
            .unwrap_or(false)
}

fn is_keyword(name: &str) -> bool {
    ES3_KEYWORDS.contains(&name)
}

/// Port of `isDotNotationIdentifier` from the Go source: ASCII identifier rules
/// (first char alpha / `_` / `$`, remaining chars alphanumeric / `_` / `$`).
fn is_dot_notation_identifier(name: &str) -> bool {
    let bytes = name.as_bytes();
    let Some(&first) = bytes.first() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == b'_' || first == b'$') {
        return false;
    }
    bytes[1..]
        .iter()
        .all(|&ch| ch.is_ascii_alphanumeric() || ch == b'_' || ch == b'$')
}

/// Extracts the computed-property value the way Go's `getComputedPropertyValue`
/// does: string literals, `true`/`false`/`null` keywords, and no-substitution
/// template literals.
fn computed_property_value(property: &LintNode) -> Option<String> {
    match property.kind.as_str() {
        "Literal" => match property.fields.get("value") {
            Some(Value::String(value)) => Some(value.clone()),
            Some(Value::Bool(value)) => Some(value.to_string()),
            // `null` literal: ESTree stores `value: null`, so fall back to the
            // raw text which is exactly `null`.
            Some(Value::Null) | None => match property.field_str("raw") {
                Some("null") => Some("null".to_owned()),
                _ => None,
            },
            _ => None,
        },
        "TemplateLiteral" => no_substitution_template_value(property),
        _ => None,
    }
}

/// Returns the cooked text of a no-substitution template literal
/// (`` `foo` ``), or `None` when the template has interpolations.
fn no_substitution_template_value(node: &LintNode) -> Option<String> {
    let expressions = node.child_list("expressions").unwrap_or(&[]);
    if !expressions.is_empty() {
        return None;
    }
    let quasis = node.child_list("quasis").unwrap_or(&[]);
    if quasis.len() != 1 {
        return None;
    }
    quasis[0]
        .fields
        .get("value")
        .and_then(|value| value.get("cooked").or_else(|| value.get("raw")))
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn report_computed_property(ctx: &mut RuleContext<'_>, node: &LintNode) {
    if should_skip_for_type_aware_allowances(node) {
        return;
    }

    let Some(property) = node.child("property") else {
        return;
    };
    let Some(value) = computed_property_value(property) else {
        return;
    };

    if !is_dot_notation_identifier(&value) {
        return;
    }
    if !allow_keywords(node) && is_keyword(&value) {
        return;
    }
    if node
        .field_bool("__propertyMatchesAllowPattern")
        .unwrap_or(false)
    {
        return;
    }

    let key = format_use_dot_key(property, &value);
    let suggestions = computed_fix(node, property, &value)
        .map(|fixes| {
            vec![LintSuggestion {
                message_id: "useDot".to_owned(),
                message: format!("[{key}] is better written in dot notation."),
                fixes,
            }]
        })
        .unwrap_or_default();
    ctx.report_with_suggestions("useDot", property.range, suggestions);
}

/// Mirrors `buildUseDotMessage`, whose `key` is the *formatted* literal: quoted
/// for strings, backtick-wrapped for templates, and bare for `true`/`false`/`null`.
fn format_use_dot_key(property: &LintNode, value: &str) -> String {
    match property.kind.as_str() {
        "TemplateLiteral" => format!("`{value}`"),
        "Literal" => match property.fields.get("value") {
            Some(Value::String(_)) => format!("{value:?}"),
            _ => value.to_owned(),
        },
        _ => value.to_owned(),
    }
}

/// Builds the auto-fix edits for a computed property, matching the Go fixer.
///
/// Replaces `[ ... ]` (from the left bracket to the right bracket) with
/// `.value`. When the access is optional (`?.[`), the `.` is preserved and only
/// the bracket portion after the question-dot token is rewritten.
fn computed_fix(node: &LintNode, property: &LintNode, value: &str) -> Option<Vec<LintFix>> {
    let object = node.child("object")?;
    let object_end = object.range.end;
    let optional = node.field_bool("optional").unwrap_or(false);
    let close_bracket = node.range.end; // `]` is the last char of the member expression
    let mut fixes = Vec::with_capacity(2);

    if optional {
        // `obj?.[k]` -> `obj?.k`: keep `?.`, replace `[k]` with `k`.
        // The replacement starts just after the question-dot token. We locate
        // the opening bracket by trimming to the first `[` after the object.
        let bracket_start = node
            .field_f64("__leftBracketStart")
            .map(|start| start as u32)
            .unwrap_or(object_end);
        fixes.push(LintFix::replace_range(
            super::super::TextRange::new(bracket_start, close_bracket),
            value.to_owned(),
        ));
    } else {
        let bracket_start = node
            .field_f64("__leftBracketStart")
            .map(|start| start as u32)
            .unwrap_or(object_end);
        // A leading space avoids `1 .foo` style adjacency when the object is a
        // decimal integer literal.
        let dot = if object_needs_space_dot(object) {
            " ."
        } else {
            "."
        };
        fixes.push(LintFix::replace_range(
            super::super::TextRange::new(bracket_start, close_bracket),
            format!("{dot}{value}"),
        ));
    }

    let _ = property;
    Some(fixes)
}

/// Whether the object is a decimal integer literal, which would require a space
/// before the dot to stay parseable (`1 .foo` rather than `1.foo`).
fn object_needs_space_dot(object: &LintNode) -> bool {
    if object.kind != "Literal" {
        return false;
    }
    let Some(raw) = object.field_str("raw") else {
        return false;
    };
    if raw.is_empty() {
        return false;
    }
    if raw.contains(['.', 'e', 'E', 'x', 'X', 'o', 'O', 'b', 'B', 'n', 'N']) {
        return false;
    }
    raw.chars()
        .enumerate()
        .all(|(index, ch)| ch.is_ascii_digit() || (ch == '_' && index > 0 && index + 1 < raw.len()))
}

fn report_dot_keyword_access(ctx: &mut RuleContext<'_>, node: &LintNode) {
    if allow_keywords(node) {
        return;
    }
    let Some(property) = node.child("property") else {
        return;
    };
    if property.kind != "Identifier" {
        return;
    }
    let Some(name) = property.field_stringish("name") else {
        return;
    };
    if !is_keyword(&name) {
        return;
    }

    let suggestions = bracket_fix(node, property, &name).map(|fixes| {
        vec![LintSuggestion {
            message_id: "useBrackets".to_owned(),
            message: format!(".{name} is a syntax error."),
            fixes,
        }]
    });
    ctx.report_with_suggestions(
        "useBrackets",
        property.range,
        suggestions.unwrap_or_default(),
    );
}

/// Builds the auto-fix for keyword dot access, matching the Go fixer:
/// remove the `.` (for non-optional access) and replace the property with
/// `["name"]`. The fix is suppressed when the object is the bare identifier
/// `let`, since `let["x"]` would change parsing semantics.
fn bracket_fix(node: &LintNode, property: &LintNode, name: &str) -> Option<Vec<LintFix>> {
    let object = node.child("object")?;
    let optional = node.field_bool("optional").unwrap_or(false);

    if !optional
        && object.kind == "Identifier"
        && object.field_stringish("name").as_deref() == Some("let")
    {
        return None;
    }

    let mut fixes = Vec::with_capacity(2);
    if !optional {
        // Remove the dot between the object and the property.
        let dot_start = node
            .field_f64("__dotStart")
            .map(|start| start as u32)
            .unwrap_or(object.range.end);
        fixes.push(LintFix::remove_range(super::super::TextRange::new(
            dot_start,
            property.range.start,
        )));
    }
    fixes.push(LintFix::replace_range(
        property.range,
        format!("[\"{name}\"]"),
    ));
    Some(fixes)
}

/// Port of `shouldSkipForTypeAwareAllowances`. The checker queries (declaration
/// modifier, index-signature membership) are delivered as raw host facts; the
/// option combination and verdict stay here.
fn should_skip_for_type_aware_allowances(node: &LintNode) -> bool {
    let allow_private = allow_private_class_property_access(node);
    let allow_protected = allow_protected_class_property_access(node);
    let allow_index = allow_index_signature_property_access(node);
    if !allow_private && !allow_protected && !allow_index {
        return false;
    }

    match node.field_str("__propertyDeclarationModifier") {
        Some("private") if allow_private => return true,
        Some("protected") if allow_protected => return true,
        _ => {}
    }

    if allow_index
        && node
            .field_bool("__propertyIsIndexSignature")
            .unwrap_or(false)
    {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::super::super::{LintNode, RuleContext, RustLintRule};
    use super::DotNotationRule;

    fn run(node: &LintNode) -> Vec<super::super::super::LintDiagnostic> {
        let rule = DotNotationRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, node);
        ctx.finish()
    }

    fn member(value: Value) -> LintNode {
        serde_json::from_value(value).unwrap()
    }

    // a['b'] -> should report useDot.
    #[test]
    fn reports_string_literal_index() {
        let node = member(json!({
            "kind": "MemberExpression",
            "range": { "start": 0, "end": 6 },
            "fields": { "computed": true },
            "children": {
                "object": { "kind": "Identifier", "range": { "start": 0, "end": 1 }, "fields": { "name": "a" } },
                "property": {
                    "kind": "Literal",
                    "range": { "start": 2, "end": 5 },
                    "fields": { "value": "b", "raw": "'b'" }
                }
            }
        }));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "useDot");
    }

    // a[`time`] with allowKeywords:false -> reports useDot (not a keyword).
    #[test]
    fn reports_template_literal_index() {
        let node = member(json!({
            "kind": "MemberExpression",
            "range": { "start": 0, "end": 9 },
            "fields": { "computed": true },
            "children": {
                "object": { "kind": "Identifier", "range": { "start": 0, "end": 1 }, "fields": { "name": "a" } },
                "property": {
                    "kind": "TemplateLiteral",
                    "range": { "start": 2, "end": 8 },
                    "childLists": {
                        "expressions": [],
                        "quasis": [
                            {
                                "kind": "TemplateElement",
                                "range": { "start": 3, "end": 7 },
                                "fields": { "value": { "cooked": "time", "raw": "time" } }
                            }
                        ]
                    }
                }
            }
        }));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "useDot");
    }

    // foo.while with allowKeywords:false -> reports useBrackets.
    #[test]
    fn reports_keyword_dot_access() {
        let mut node = member(json!({
            "kind": "MemberExpression",
            "range": { "start": 0, "end": 9 },
            "fields": { "computed": false },
            "children": {
                "object": { "kind": "Identifier", "range": { "start": 0, "end": 3 }, "fields": { "name": "foo" } },
                "property": { "kind": "Identifier", "range": { "start": 4, "end": 9 }, "fields": { "name": "while" } }
            }
        }));
        node.fields.insert(
            "__ruleOptions".to_owned(),
            json!([{ "allowKeywords": false }]),
        );
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "useBrackets");
    }

    // a[b] (non-literal) -> no report.
    #[test]
    fn ignores_dynamic_index() {
        let node = member(json!({
            "kind": "MemberExpression",
            "range": { "start": 0, "end": 4 },
            "fields": { "computed": true },
            "children": {
                "object": { "kind": "Identifier", "range": { "start": 0, "end": 1 }, "fields": { "name": "a" } },
                "property": { "kind": "Identifier", "range": { "start": 2, "end": 3 }, "fields": { "name": "b" } }
            }
        }));
        assert!(run(&node).is_empty());
    }

    // a['has space'] -> not a valid identifier, no report.
    #[test]
    fn ignores_non_identifier_string() {
        let node = member(json!({
            "kind": "MemberExpression",
            "range": { "start": 0, "end": 14 },
            "fields": { "computed": true },
            "children": {
                "object": { "kind": "Identifier", "range": { "start": 0, "end": 1 }, "fields": { "name": "a" } },
                "property": {
                    "kind": "Literal",
                    "range": { "start": 2, "end": 13 },
                    "fields": { "value": "has space", "raw": "'has space'" }
                }
            }
        }));
        assert!(run(&node).is_empty());
    }

    // a['while'] with allowKeywords:false -> keyword, no report.
    #[test]
    fn ignores_keyword_when_keywords_disallowed() {
        let mut node = member(json!({
            "kind": "MemberExpression",
            "range": { "start": 0, "end": 10 },
            "fields": { "computed": true },
            "children": {
                "object": { "kind": "Identifier", "range": { "start": 0, "end": 1 }, "fields": { "name": "a" } },
                "property": {
                    "kind": "Literal",
                    "range": { "start": 2, "end": 9 },
                    "fields": { "value": "while", "raw": "'while'" }
                }
            }
        }));
        node.fields.insert(
            "__ruleOptions".to_owned(),
            json!([{ "allowKeywords": false }]),
        );
        assert!(run(&node).is_empty());
    }

    // a.true with default allowKeywords (true) -> dot keyword arm does nothing.
    #[test]
    fn ignores_keyword_dot_access_when_allowed() {
        let node = member(json!({
            "kind": "MemberExpression",
            "range": { "start": 0, "end": 6 },
            "fields": { "computed": false },
            "children": {
                "object": { "kind": "Identifier", "range": { "start": 0, "end": 1 }, "fields": { "name": "a" } },
                "property": { "kind": "Identifier", "range": { "start": 2, "end": 6 }, "fields": { "name": "true" } }
            }
        }));
        assert!(run(&node).is_empty());
    }

    // a['null'] -> string literal "null" is a valid identifier but a keyword;
    // with default allowKeywords:true it should report.
    #[test]
    fn reports_null_string_when_keywords_allowed() {
        let node = member(json!({
            "kind": "MemberExpression",
            "range": { "start": 0, "end": 9 },
            "fields": { "computed": true },
            "children": {
                "object": { "kind": "Identifier", "range": { "start": 0, "end": 1 }, "fields": { "name": "a" } },
                "property": {
                    "kind": "Literal",
                    "range": { "start": 2, "end": 8 },
                    "fields": { "value": "null", "raw": "'null'" }
                }
            }
        }));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "useDot");
    }

    // a['snake_case'] with allowPattern match -> host marks it; no report.
    #[test]
    fn ignores_allow_pattern_match() {
        let mut node = member(json!({
            "kind": "MemberExpression",
            "range": { "start": 0, "end": 14 },
            "fields": { "computed": true },
            "children": {
                "object": { "kind": "Identifier", "range": { "start": 0, "end": 1 }, "fields": { "name": "a" } },
                "property": {
                    "kind": "Literal",
                    "range": { "start": 2, "end": 13 },
                    "fields": { "value": "snake_case", "raw": "'snake_case'" }
                }
            }
        }));
        node.fields
            .insert("__propertyMatchesAllowPattern".to_owned(), json!(true));
        assert!(run(&node).is_empty());
    }

    // a[true] -> boolean literal, valid identifier, default allowKeywords:true -> reports useDot.
    #[test]
    fn reports_boolean_literal_index() {
        let node = member(json!({
            "kind": "MemberExpression",
            "range": { "start": 0, "end": 7 },
            "fields": { "computed": true },
            "children": {
                "object": { "kind": "Identifier", "range": { "start": 0, "end": 1 }, "fields": { "name": "a" } },
                "property": {
                    "kind": "Literal",
                    "range": { "start": 2, "end": 6 },
                    "fields": { "value": true, "raw": "true" }
                }
            }
        }));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "useDot");
    }

    // a[null] -> null literal, valid identifier, default allowKeywords:true -> reports useDot.
    #[test]
    fn reports_null_literal_index() {
        let node = member(json!({
            "kind": "MemberExpression",
            "range": { "start": 0, "end": 7 },
            "fields": { "computed": true },
            "children": {
                "object": { "kind": "Identifier", "range": { "start": 0, "end": 1 }, "fields": { "name": "a" } },
                "property": {
                    "kind": "Literal",
                    "range": { "start": 2, "end": 6 },
                    "fields": { "value": Value::Null, "raw": "null" }
                }
            }
        }));
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "useDot");
    }

    // a['null'] with allowKeywords:false -> string "null" is a keyword -> no report.
    #[test]
    fn ignores_null_string_when_keywords_disallowed() {
        let mut node = member(json!({
            "kind": "MemberExpression",
            "range": { "start": 0, "end": 9 },
            "fields": { "computed": true },
            "children": {
                "object": { "kind": "Identifier", "range": { "start": 0, "end": 1 }, "fields": { "name": "a" } },
                "property": {
                    "kind": "Literal",
                    "range": { "start": 2, "end": 8 },
                    "fields": { "value": "null", "raw": "'null'" }
                }
            }
        }));
        node.fields.insert(
            "__ruleOptions".to_owned(),
            json!([{ "allowKeywords": false }]),
        );
        assert!(run(&node).is_empty());
    }

    // let.if() with allowKeywords:false -> still reports useBrackets (fix suppressed, diagnostic fires).
    #[test]
    fn reports_let_keyword_access_even_when_fix_suppressed() {
        let mut node = member(json!({
            "kind": "MemberExpression",
            "range": { "start": 0, "end": 6 },
            "fields": { "computed": false },
            "children": {
                "object": { "kind": "Identifier", "range": { "start": 0, "end": 3 }, "fields": { "name": "let" } },
                "property": { "kind": "Identifier", "range": { "start": 4, "end": 6 }, "fields": { "name": "if" } }
            }
        }));
        node.fields.insert(
            "__ruleOptions".to_owned(),
            json!([{ "allowKeywords": false }]),
        );
        let diagnostics = run(&node);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message_id, "useBrackets");
        // Fix suppressed for bare `let` object: suggestion carries no fixes.
        assert!(
            diagnostics[0].suggestions.is_empty() || diagnostics[0].suggestions[0].fixes.is_empty()
        );
    }

    // x['priv_prop'] with allowPrivateClassPropertyAccess + private modifier -> no report.
    #[test]
    fn ignores_private_when_allowed() {
        let mut node = member(json!({
            "kind": "MemberExpression",
            "range": { "start": 0, "end": 14 },
            "fields": { "computed": true },
            "children": {
                "object": { "kind": "Identifier", "range": { "start": 0, "end": 1 }, "fields": { "name": "x" } },
                "property": {
                    "kind": "Literal",
                    "range": { "start": 2, "end": 13 },
                    "fields": { "value": "priv_prop", "raw": "'priv_prop'" }
                }
            }
        }));
        node.fields.insert(
            "__ruleOptions".to_owned(),
            json!([{ "allowPrivateClassPropertyAccess": true }]),
        );
        node.fields.insert(
            "__propertyDeclarationModifier".to_owned(),
            Value::String("private".to_owned()),
        );
        assert!(run(&node).is_empty());
    }
}
