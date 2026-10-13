use serde_json::Value;

use super::LintNode;
use crate::utils::{
    TypeTextKind, classify_type_text, is_any_like_type_texts, is_error_like_type_texts,
    is_number_like_type_texts, is_promise_like_type_texts, is_string_like_type_texts,
    is_unknown_like_type_texts, split_type_text,
};

pub(super) fn strip_chain_expression(mut node: &LintNode) -> &LintNode {
    while node.kind == "ChainExpression" {
        let Some(expression) = node.child("expression") else {
            break;
        };
        node = expression;
    }
    node
}

pub(super) fn is_promise_like_node(node: &LintNode) -> bool {
    is_promise_like_type_texts(&node.type_texts, &node.property_names)
}

pub(super) fn is_obviously_promise_like(node: &LintNode) -> bool {
    let current = strip_chain_expression(node);
    if current.kind == "NewExpression" {
        return current
            .child("callee")
            .is_some_and(|callee| is_identifier_named(callee, "Promise"));
    }
    if current.kind != "CallExpression" {
        return false;
    }
    let Some(callee) = current.child("callee") else {
        return false;
    };
    member_property_name(callee).as_deref() == Some("resolve")
        && member_object(callee).is_some_and(|object| is_identifier_named(object, "Promise"))
}

pub(super) fn is_error_like_node(node: &LintNode) -> bool {
    let current = strip_chain_expression(node);
    if current.kind == "NewExpression"
        && current
            .child("callee")
            .and_then(|callee| identifier_name(callee).or_else(|| member_property_name(callee)))
            .is_some_and(|identifier| identifier.ends_with("Error"))
    {
        return true;
    }
    is_error_like_type_texts(&node.type_texts, &node.property_names)
}

pub(super) fn is_any_like_node(node: &LintNode) -> bool {
    let current = strip_chain_expression(node);
    if current.kind == "TSAsExpression"
        && current
            .child("typeAnnotation")
            .is_some_and(|type_annotation| type_annotation.kind == "TSAnyKeyword")
    {
        return true;
    }
    if current.kind == "TSAsExpression" && current.field_str("__typeAnnotationText") == Some("any")
    {
        return true;
    }
    is_any_like_type_texts(&node.type_texts)
}

pub(super) fn is_unknown_like_node(node: &LintNode) -> bool {
    let current = strip_chain_expression(node);
    if current.kind == "TSAsExpression"
        && current
            .child("typeAnnotation")
            .is_some_and(|type_annotation| type_annotation.kind == "TSUnknownKeyword")
    {
        return true;
    }
    if current.kind == "TSAsExpression"
        && current.field_str("__typeAnnotationText") == Some("unknown")
    {
        return true;
    }
    is_unknown_like_type_texts(&node.type_texts)
}

pub(super) fn member_property_name(node: &LintNode) -> Option<String> {
    let current = strip_chain_expression(node);
    if current.kind != "MemberExpression" {
        return None;
    }
    let property = current.child("property")?;
    if !current.field_bool("computed").unwrap_or(false) && property.kind == "Identifier" {
        return property.field_stringish("name");
    }
    if current.field_bool("computed").unwrap_or(false) && property.kind == "Literal" {
        return property.field_stringish("value");
    }
    None
}

pub(super) fn member_object(node: &LintNode) -> Option<&LintNode> {
    let current = strip_chain_expression(node);
    if current.kind == "MemberExpression" {
        current.child("object")
    } else {
        None
    }
}

pub(super) fn callee_property_name(node: Option<&LintNode>) -> Option<String> {
    let node = strip_chain_expression(node?);
    if node.kind == "CallExpression" {
        node.child("callee").and_then(member_property_name)
    } else {
        None
    }
}

pub(super) fn identifier_name(node: &LintNode) -> Option<String> {
    let current = strip_chain_expression(node);
    if current.kind == "Identifier" {
        current.field_stringish("name")
    } else {
        None
    }
}

pub(super) fn is_identifier_named(node: &LintNode, name: &str) -> bool {
    identifier_name(node).as_deref() == Some(name)
}

pub(super) fn is_literal_string(node: &LintNode) -> bool {
    let current = strip_chain_expression(node);
    current.kind == "Literal" && current.fields.get("value").is_some_and(Value::is_string)
}

pub(super) fn is_negative_one_literal(node: &LintNode) -> bool {
    let current = strip_chain_expression(node);
    if current.kind == "Literal" && current.field_f64("value") == Some(-1.0) {
        return true;
    }
    current.kind == "UnaryExpression"
        && current.field_str("operator") == Some("-")
        && current
            .child("argument")
            .is_some_and(|arg| arg.kind == "Literal" && arg.field_f64("value") == Some(1.0))
}

pub(super) fn is_zero_literal(node: &LintNode) -> bool {
    let current = strip_chain_expression(node);
    current.kind == "Literal" && current.field_f64("value") == Some(0.0)
}

pub(super) fn is_number_or_bigint_literal(node: &LintNode) -> bool {
    let current = strip_chain_expression(node);
    current.kind == "Literal"
        && (current.fields.get("value").is_some_and(Value::is_number)
            || current.fields.get("bigint").is_some_and(Value::is_string))
}

pub(super) fn is_comparable_index_search(node: &LintNode) -> bool {
    matches!(
        callee_property_name(Some(node)).as_deref(),
        Some("indexOf" | "lastIndexOf")
    )
}

pub(super) fn first_child_list_item<'a>(node: &'a LintNode, key: &str) -> Option<&'a LintNode> {
    node.child_list(key).and_then(|items| items.first())
}

pub(super) fn child_list<'a>(node: &'a LintNode, key: &str) -> &'a [LintNode] {
    node.child_list(key).unwrap_or(&[])
}

pub(super) fn rule_option_bool(node: &LintNode, key: &str) -> Option<bool> {
    node.fields
        .get("__ruleOptions")?
        .as_array()?
        .first()?
        .get(key)?
        .as_bool()
}

pub(super) fn regex_flags(node: &LintNode) -> Option<&str> {
    node.fields
        .get("regex")
        .and_then(|regex| regex.get("flags"))
        .and_then(Value::as_str)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum EnumMemberKind {
    Number,
    String,
    Unknown,
}

pub(super) fn enum_members_of(node: &LintNode) -> &[LintNode] {
    node.child("body")
        .and_then(|body| body.child_list("members"))
        .or_else(|| node.child_list("members"))
        .unwrap_or(&[])
}

pub(super) fn enum_member_kind(member: &LintNode) -> EnumMemberKind {
    let Some(initializer) = member.child("initializer") else {
        return EnumMemberKind::Number;
    };
    if initializer.kind == "Literal" {
        if initializer
            .fields
            .get("value")
            .is_some_and(Value::is_number)
        {
            return EnumMemberKind::Number;
        }
        if initializer
            .fields
            .get("value")
            .is_some_and(Value::is_string)
        {
            return EnumMemberKind::String;
        }
    }
    if is_string_like_type_texts(&initializer.type_texts) {
        return EnumMemberKind::String;
    }
    if is_number_like_type_texts(&initializer.type_texts) {
        return EnumMemberKind::Number;
    }
    EnumMemberKind::Unknown
}

pub(super) fn has_unknown_type_annotation(node: &LintNode) -> bool {
    node.child("typeAnnotation")
        .and_then(|type_annotation| {
            type_annotation
                .child("typeAnnotation")
                .or(Some(type_annotation))
        })
        .is_some_and(|type_annotation| type_annotation.kind == "TSUnknownKeyword")
}

pub(super) fn is_unary_minus_type_safe<T: AsRef<str>>(type_texts: &[T]) -> bool {
    !type_texts.is_empty()
        && type_texts.iter().all(|text| {
            split_type_text(text.as_ref()).iter().all(|part| {
                matches!(
                    classify_type_text(Some(part.as_str())),
                    TypeTextKind::Any | TypeTextKind::Number | TypeTextKind::Bigint
                )
            })
        })
}

/// Reads the names out of a `TypeOrValueSpecifier` list option
/// (string entries and object entries with a `name` string or string array).
pub(super) fn rule_allow_list_names(node: &LintNode, key: &str) -> Vec<String> {
    let Some(entries) = node
        .fields
        .get("__ruleOptions")
        .and_then(serde_json::Value::as_array)
        .and_then(|options| options.first())
        .and_then(|options| options.get(key))
        .and_then(serde_json::Value::as_array)
    else {
        return Vec::new();
    };
    let mut names = Vec::new();
    for entry in entries {
        match entry {
            serde_json::Value::String(name) => names.push(name.clone()),
            serde_json::Value::Object(spec) => match spec.get("name") {
                Some(serde_json::Value::String(name)) => names.push(name.clone()),
                Some(serde_json::Value::Array(name_list)) => {
                    names.extend(
                        name_list
                            .iter()
                            .filter_map(serde_json::Value::as_str)
                            .map(String::from),
                    );
                }
                _ => {}
            },
            _ => {}
        }
    }
    names
}

/// Whether any rendered type text's bare reference name matches one of the
/// allow-listed names.
pub(super) fn type_texts_match_names<T: AsRef<str>>(type_texts: &[T], names: &[String]) -> bool {
    if names.is_empty() {
        return false;
    }
    type_texts.iter().any(|text| {
        crate::utils::split_top_level_type_text(text.as_ref(), '|')
            .iter()
            .any(|part| {
                let part = part.trim();
                let name = if part.ends_with("[]") {
                    if part.starts_with("readonly ") {
                        "ReadonlyArray"
                    } else {
                        "Array"
                    }
                } else {
                    &part[..part.find('<').unwrap_or(part.len())]
                };
                names
                    .iter()
                    .any(|allowed| allowed == name || allowed == part)
            })
    })
}
