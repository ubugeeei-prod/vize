//! Report ranges aligned with typescript-eslint.
use std::collections::BTreeMap;

use corsa::lint::{LintNode, TextRange};

pub(super) fn prepare(rule: &str, node: &LintNode) -> LintNode {
    let mut owned = node.clone();
    if rule == "no-base-to-string" {
        supply_tostring_argument(&mut owned);
    }
    owned
}

fn supply_tostring_argument(node: &mut LintNode) {
    if node.kind != "CallExpression" {
        return;
    }
    if node
        .child_list("arguments")
        .is_some_and(|arguments| !arguments.is_empty())
    {
        return;
    }
    let property = node
        .child("callee")
        .and_then(|callee| callee.child("property"))
        .and_then(|property| property.field_str("name"));
    if !matches!(property, Some("toString" | "toLocaleString")) {
        return;
    }
    node.child_lists
        .entry("arguments".into())
        .or_default()
        .push(LintNode {
            kind: "Literal".into(),
            range: TextRange::new(node.range.end, node.range.end),
            text: None,
            type_texts: Vec::new(),
            property_names: Vec::new(),
            fields: BTreeMap::new(),
            children: BTreeMap::new(),
            child_lists: BTreeMap::new(),
        });
}

pub(super) fn range(rule: &str, node: &LintNode, start: u32, end: u32) -> (u32, u32) {
    match rule {
        "no-implied-eval" => implied_eval(node).unwrap_or((start, end)),
        "restrict-plus-operands" => plus_operand(node).unwrap_or((start, end)),
        "no-unsafe-return" => unsafe_return(node, start, end),
        _ => (start, end),
    }
}

fn implied_eval(node: &LintNode) -> Option<(u32, u32)> {
    let argument = node.child_list("arguments")?.first()?;
    Some((argument.range.start, argument.range.end))
}

fn plus_operand(node: &LintNode) -> Option<(u32, u32)> {
    if node.kind != "BinaryExpression" {
        return None;
    }
    let left = node.child("left")?;
    let right = node.child("right")?;
    if is_primitive(&left.type_texts) && !is_primitive(&right.type_texts) {
        return Some((right.range.start, right.range.end));
    }
    if is_primitive(&right.type_texts) && !is_primitive(&left.type_texts) {
        return Some((left.range.start, left.range.end));
    }
    None
}

fn unsafe_return(node: &LintNode, start: u32, end: u32) -> (u32, u32) {
    if node.kind == "ReturnStatement" {
        (node.range.start, node.range.end.max(end))
    } else {
        (start, end)
    }
}

fn is_primitive(texts: &[String]) -> bool {
    texts.iter().any(|text| {
        let text = text.trim();
        matches!(
            text,
            "string"
                | "number"
                | "bigint"
                | "boolean"
                | "any"
                | "never"
                | "unknown"
                | "null"
                | "undefined"
                | "void"
        ) || text.starts_with('"')
            || text.starts_with('\'')
    })
}
