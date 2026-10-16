use serde_json::Value;

use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule};
use crate::{
    lint::helpers::{
        callee_property_name, child_list, is_identifier_named, is_literal_string, member_object,
        rule_option_bool, strip_chain_expression,
    },
    utils::{
        TypeTextKind, classify_type_text, is_string_like_type_texts, split_top_level_type_text,
    },
};

/// Type-aware rule that rejects values stringified through base Object#toString.
///
/// Supports the upstream `checkUnknown` and `ignoredTypeNames` options.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoBaseToStringRule;

struct Options {
    check_unknown: bool,
    ignored_type_names: Vec<String>,
}

impl Options {
    fn from_node(node: &LintNode) -> Self {
        Self {
            check_unknown: rule_option_bool(node, "checkUnknown").unwrap_or(false),
            ignored_type_names: ignored_type_names(node),
        }
    }
}

/// Reads `ignoredTypeNames`, defaulting to the upstream list.
fn ignored_type_names(node: &LintNode) -> Vec<String> {
    let configured = node
        .fields
        .get("__ruleOptions")
        .and_then(Value::as_array)
        .and_then(|options| options.first())
        .and_then(|options| options.get("ignoredTypeNames"))
        .and_then(Value::as_array)
        .map(|names| {
            names
                .iter()
                .filter_map(Value::as_str)
                .map(String::from)
                .collect::<Vec<_>>()
        });
    configured.unwrap_or_else(|| {
        ["Error", "RegExp", "URL", "URLSearchParams"]
            .map(String::from)
            .to_vec()
    })
}

const MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "unexpected",
    description: "This value is stringified through its base Object#toString() representation.",
}];
const LISTENERS: &[&str] = &["BinaryExpression", "CallExpression", "TemplateLiteral"];

impl RustLintRule for NoBaseToStringRule {
    fn name(&self) -> &'static str {
        "no-base-to-string"
    }

    fn docs_description(&self) -> &'static str {
        "Disallow stringifying values that fall back to Object.prototype.toString()."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        let options = Options::from_node(node);
        match node.kind.as_str() {
            "BinaryExpression" if node.field_str("operator") == Some("+") => {
                let Some(left) = node.child("left") else {
                    return;
                };
                let Some(right) = node.child("right") else {
                    return;
                };
                if is_literal_string(left) || is_string_like_type_texts(&left.type_texts) {
                    report_if_unsafe(ctx, right, &options);
                }
                if is_literal_string(right) || is_string_like_type_texts(&right.type_texts) {
                    report_if_unsafe(ctx, left, &options);
                }
            }
            "CallExpression" => {
                let Some(first_argument) = child_list(node, "arguments").first() else {
                    return;
                };
                if node
                    .child("callee")
                    .is_some_and(|callee| is_identifier_named(callee, "String"))
                {
                    report_if_unsafe(ctx, first_argument, &options);
                    return;
                }
                if callee_property_name(Some(node)).as_deref() == Some("toString")
                    && let Some(object) = node.child("callee").and_then(member_object)
                {
                    report_if_unsafe(ctx, object, &options);
                }
            }
            "TemplateLiteral" => {
                for expression in child_list(node, "expressions") {
                    report_if_unsafe(ctx, expression, &options);
                }
            }
            _ => {}
        }
    }
}

fn report_if_unsafe(ctx: &mut RuleContext<'_>, node: &LintNode, options: &Options) {
    if is_possibly_base_to_string(node, options) {
        ctx.report("unexpected", node.range);
    }
}

fn is_possibly_base_to_string(node: &LintNode, options: &Options) -> bool {
    let current = strip_chain_expression(node);
    // A plain object literal stringifies through Object.prototype unless it
    // declares its own toString/toLocaleString.
    if current.kind == "ObjectExpression" {
        return !object_literal_defines_to_string(current);
    }
    !node.type_texts.is_empty()
        && node.type_texts.iter().any(|text| {
            split_top_level_type_text(text, '|')
                .iter()
                .any(|part| is_unsafe_stringified_text(part, options, &node.property_names))
        })
}

fn object_literal_defines_to_string(node: &LintNode) -> bool {
    child_list(node, "properties").iter().any(|property| {
        property
            .child("key")
            .and_then(|key| key.field_str("name").map(str::to_owned))
            .or_else(|| property.field_str("name").map(str::to_owned))
            .is_some_and(|name| name == "toString" || name == "toLocaleString")
    })
}

fn is_unsafe_stringified_text(text: &str, options: &Options, property_names: &[String]) -> bool {
    let current = text.trim();
    if matches!(
        classify_type_text(Some(current)),
        TypeTextKind::String
            | TypeTextKind::Number
            | TypeTextKind::Bigint
            | TypeTextKind::Boolean
            | TypeTextKind::Nullish
            | TypeTextKind::Regexp
    ) {
        return false;
    }
    if current == "symbol" {
        return true;
    }
    if current == "unknown" {
        return options.check_unknown;
    }
    // Functions stringify through Function.prototype.toString, not
    // Object.prototype.
    if current.contains("=>") || current.starts_with("new ") {
        return false;
    }
    // Built-ins that carry their own toString implementation are never
    // stringified through Object.prototype.
    if matches!(
        current,
        "Date"
            | "Error"
            | "EvalError"
            | "RangeError"
            | "ReferenceError"
            | "RegExp"
            | "SyntaxError"
            | "TypeError"
            | "URIError"
            | "URL"
            | "URLSearchParams"
            | "Function"
    ) {
        return false;
    }
    let reference_name = &current[..current.find('<').unwrap_or(current.len())];
    if options
        .ignored_type_names
        .iter()
        .any(|ignored| ignored == reference_name)
    {
        return false;
    }
    // Arrays and tuples stringify through Array.prototype.join, so only the
    // element types decide safety (`string[]` is fine, `object[]` is not).
    if let Some(element) = array_element_text(current) {
        return split_top_level_type_text(element, '|')
            .iter()
            .any(|part| is_unsafe_stringified_text(part, options, &[]));
    }
    if current.starts_with('[') && current.ends_with(']') {
        return split_top_level_type_text(&current[1..current.len() - 1], ',')
            .iter()
            .any(|part| is_unsafe_stringified_text(part, options, &[]));
    }
    if current == "object"
        || current == "Object"
        || current.starts_with('{')
        || current.starts_with("Map<")
        || current.starts_with("ReadonlyMap<")
        || current.starts_with("Set<")
        || current.starts_with("ReadonlySet<")
        || current.starts_with("Record<")
        || current.starts_with("WeakMap<")
        || current.starts_with("WeakSet<")
        || current.starts_with("Promise<")
    {
        return true;
    }
    // A nominal reference type is unsafe when the checker-provided member
    // list proves it declares no toString/toLocaleString of its own. An empty
    // member list means the fact is unavailable, so stay silent.
    reference_name
        .chars()
        .next()
        .is_some_and(char::is_uppercase)
        && !property_names.is_empty()
        && !property_names
            .iter()
            .any(|name| name == "toString" || name == "toLocaleString")
}

/// Returns the element type text when `text` denotes an array type.
fn array_element_text(text: &str) -> Option<&str> {
    if let Some(element) = text.strip_suffix("[]") {
        let element = element.trim();
        return Some(
            element
                .strip_prefix('(')
                .and_then(|inner| inner.strip_suffix(')'))
                .unwrap_or(element),
        );
    }
    for wrapper in ["Array<", "ReadonlyArray<"] {
        if let Some(rest) = text.strip_prefix(wrapper) {
            return Some(rest.strip_suffix('>').unwrap_or(rest));
        }
    }
    text.strip_prefix("readonly ").and_then(array_element_text)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::super::{LintNode, RuleContext};
    use super::NoBaseToStringRule;
    use crate::lint::RustLintRule;

    fn run(node: &LintNode) -> Vec<String> {
        let rule = NoBaseToStringRule;
        let mut ctx = RuleContext::new(&rule);
        rule.check(&mut ctx, node);
        ctx.finish()
            .into_iter()
            .map(|diagnostic| diagnostic.message_id)
            .collect()
    }

    fn node(value: serde_json::Value) -> LintNode {
        serde_json::from_value(value).expect("valid LintNode")
    }

    fn template_with_expression(type_text: &str, options: Option<serde_json::Value>) -> LintNode {
        template_with_typed_expression(type_text, json!([]), options)
    }

    fn template_with_typed_expression(
        type_text: &str,
        property_names: serde_json::Value,
        options: Option<serde_json::Value>,
    ) -> LintNode {
        let mut fields = serde_json::Map::new();
        if let Some(options) = options {
            fields.insert("__ruleOptions".to_owned(), json!([options]));
        }
        node(json!({
            "kind": "TemplateLiteral",
            "range": { "start": 0, "end": 30 },
            "fields": fields,
            "childLists": {
                "expressions": [{
                    "kind": "Identifier",
                    "range": { "start": 3, "end": 8 },
                    "typeTexts": [type_text],
                    "propertyNames": property_names,
                    "fields": { "name": "value" }
                }]
            }
        }))
    }

    #[test]
    fn unknown_is_ignored_by_default() {
        assert!(run(&template_with_expression("unknown", None)).is_empty());
    }

    #[test]
    fn check_unknown_reports_unknown_values() {
        let diagnostics = run(&template_with_expression(
            "unknown",
            Some(json!({ "checkUnknown": true })),
        ));
        assert_eq!(diagnostics, vec!["unexpected"]);
    }

    #[test]
    fn ignored_type_names_suppress_reports() {
        let members = json!(["rows", "cols"]);
        assert_eq!(
            run(&template_with_typed_expression(
                "Matrix",
                members.clone(),
                None
            )),
            vec!["unexpected"]
        );
        assert!(
            run(&template_with_typed_expression(
                "Matrix",
                members.clone(),
                Some(json!({ "ignoredTypeNames": ["Matrix"] })),
            ))
            .is_empty()
        );
        // Generic instantiations match on the bare reference name.
        assert!(
            run(&template_with_typed_expression(
                "Matrix<number>",
                members,
                Some(json!({ "ignoredTypeNames": ["Matrix"] })),
            ))
            .is_empty()
        );
    }

    #[test]
    fn nominal_types_with_their_own_to_string_stay_silent() {
        assert!(
            run(&template_with_typed_expression(
                "Matrix",
                json!(["rows", "cols", "toString"]),
                None,
            ))
            .is_empty()
        );
    }

    #[test]
    fn functions_and_safe_arrays_are_not_reported() {
        assert!(run(&template_with_expression("() => void", None)).is_empty());
        assert!(run(&template_with_expression("string[]", None)).is_empty());
        assert!(run(&template_with_expression("Array<number>", None)).is_empty());
    }

    #[test]
    fn arrays_of_base_objects_are_reported() {
        assert_eq!(
            run(&template_with_expression("object[]", None)),
            vec!["unexpected"]
        );
        assert_eq!(
            run(&template_with_expression("Array<{ id: number }>", None)),
            vec!["unexpected"]
        );
    }

    #[test]
    fn own_tostring_builtins_stay_silent_with_custom_ignore_list() {
        assert!(
            run(&template_with_expression(
                "Date",
                Some(json!({ "ignoredTypeNames": ["Matrix"] })),
            ))
            .is_empty()
        );
    }
}
