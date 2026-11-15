//! Report ranges aligned with typescript-eslint.
use std::collections::BTreeMap;

use corsa::lint::{LintNode, TextRange};
use serde_json::Value;
use vize_l0::{String, cstr};

pub(super) fn prepare(rule: &str, node: &LintNode) -> LintNode {
    let mut owned = node.clone();
    if rule == "no-base-to-string" {
        constrain_stringified_types(&mut owned);
        supply_tostring_argument(&mut owned);
    }
    owned
}

/// Mirror `getConstrainedTypeAtLocation` for the exact expression shapes this
/// rule stringifies. Corsa currently follows type-parameter constraints but
/// not indexed-access constraints such as `Record<K, V>[K]` (#13004).
fn constrain_stringified_types(node: &mut LintNode) {
    let constrain = |node: &mut LintNode| {
        if let Some(tree) = node.fields.get_mut("__typeTree") {
            constrain_tree(tree);
        }
    };
    match node.kind.as_str() {
        "BinaryExpression" | "AssignmentExpression" => {
            for key in ["left", "right"] {
                if let Some(expression) = node.children.get_mut(key) {
                    constrain(expression);
                }
            }
        }
        "CallExpression" => {
            if let Some(argument) = node
                .child_lists
                .get_mut("arguments")
                .and_then(|arguments| arguments.first_mut())
            {
                constrain(argument);
            }
            if let Some(object) = node
                .children
                .get_mut("callee")
                .and_then(|callee| callee.children.get_mut("object"))
            {
                constrain(object);
            }
        }
        "TemplateLiteral" => {
            if let Some(expressions) = node.child_lists.get_mut("expressions") {
                for expression in expressions {
                    constrain(expression);
                }
            }
        }
        _ => {}
    }
}

fn constrain_tree(tree: &mut Value) {
    let Some(object) = tree.as_object_mut() else {
        return;
    };
    if let Some(mut constraint) = object.remove("constraint") {
        constrain_tree(&mut constraint);
        *tree = constraint;
        return;
    }
    if let Some(child) = object.get_mut("arrayElement") {
        constrain_tree(child);
    }
    for key in ["types", "tupleElements"] {
        if let Some(children) = object.get_mut(key).and_then(Value::as_array_mut) {
            for child in children {
                constrain_tree(child);
            }
        }
    }
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::constrain_tree;

    #[test]
    fn constrained_indexed_accesses_are_normalized_inside_arrays() {
        let mut tree = json!({
            "flags": 1_048_576,
            "arrayElement": {
                "flags": 33_554_432,
                "text": "Record<K, Badge>[K]",
                "constraint": {
                    "flags": 1_048_576,
                    "text": "Badge",
                    "toString": true
                }
            }
        });
        constrain_tree(&mut tree);
        assert_eq!(
            tree,
            json!({
                "flags": 1_048_576,
                "arrayElement": {
                    "flags": 1_048_576,
                    "text": "Badge",
                    "toString": true
                }
            })
        );
    }
}
