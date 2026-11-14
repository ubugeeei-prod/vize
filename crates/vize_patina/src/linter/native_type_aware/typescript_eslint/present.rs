//! Report ranges aligned with typescript-eslint.
use std::collections::BTreeMap;

use corsa::lint::{LintNode, TextRange};
use vize_l0::{String, cstr};

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
        "no-unsafe-return" => unsafe_return(node, start, end),
        _ => (start, end),
    }
}

pub(super) fn message(rule: &str, node: &LintNode, start: u32, fallback: String) -> String {
    if rule != "no-unnecessary-type-parameters" {
        return fallback;
    }
    let Some(param) = node.child_list("params").and_then(|params| {
        params
            .iter()
            .find(|param| param.range.start <= start && start < param.range.end)
    }) else {
        return fallback;
    };
    let name = param.field_str("__typeParameterName").unwrap_or("T");
    let uses = match param.field_f64("__typeParameterUsageCount") {
        Some(1.0) => "never used",
        Some(2.0) => "used only once",
        _ => return fallback,
    };
    let descriptor = match node.field_str("__ownerKind") {
        Some("ClassDeclaration" | "ClassExpression") => "class",
        _ => "function",
    };
    cstr!("Type parameter {name} is {uses} in the {descriptor} signature.")
}

fn unsafe_return(node: &LintNode, start: u32, end: u32) -> (u32, u32) {
    if node.kind == "ReturnStatement" {
        (node.range.start, node.range.end.max(end))
    } else {
        (start, end)
    }
}
