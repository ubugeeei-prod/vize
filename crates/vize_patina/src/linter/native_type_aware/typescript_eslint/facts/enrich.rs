//! Facts filled in after the checker replies.
use super::scan::{self, LocalFn};
use corsa::lint::LintNode;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};

pub(super) fn apply(source: &str, nodes: &mut [LintNode], program: &Value) {
    let functions = scan::local_functions(program, source);
    let enums = super::shape::enum_members(program);
    let objects = super::shape::object_literal_bindings(program);
    let bindings = super::shape::annotated_bindings(program, source);
    for node in nodes {
        walk(node, &functions, &enums, &objects, &bindings);
    }
}

fn walk(
    node: &mut LintNode,
    functions: &HashMap<String, LocalFn>,
    enums: &HashMap<String, HashMap<String, String>>,
    objects: &HashSet<String>,
    bindings: &HashMap<String, String>,
) {
    if node.kind == "CallExpression" {
        fill_call(node, functions, objects);
    }
    if node.kind == "TSAsExpression" || node.kind == "TSTypeAssertion" {
        fill_assertion(node, bindings);
    }
    if node.kind == "MemberExpression" {
        fill_enum_member(node, enums);
    }
    if !node.fields.contains_key("__enumTypeIds") {
        fill_primitive_side(node);
    }
    for child in node.children.values_mut() {
        walk(child, functions, enums, objects, bindings);
    }
    for children in node.child_lists.values_mut() {
        for child in children {
            walk(child, functions, enums, objects, bindings);
        }
    }
}

fn fill_call(node: &mut LintNode, functions: &HashMap<String, LocalFn>, objects: &HashSet<String>) {
    let Some(name) = callee_name(node) else {
        ensure_tostring_object(node, objects);
        return;
    };
    if let Some(returns) = functions
        .get(&name)
        .and_then(|function| function.returns.clone())
    {
        if (returns == "void" || returns == "undefined")
            && !node.type_texts.iter().any(|text| {
                let text = text.trim();
                text == "void" || text == "undefined"
            })
        {
            node.type_texts = vec![returns];
        }
    }
    if !node.fields.contains_key("__expectedArgumentTypeTexts") {
        if let Some(function) = functions.get(&name) {
            let texts = function
                .value_params
                .iter()
                .map(|param| param.clone().into_iter().collect::<Vec<_>>())
                .collect::<Vec<_>>();
            if texts.iter().any(|texts| !texts.is_empty()) {
                node.fields
                    .insert("__expectedArgumentTypeTexts".into(), json!(texts));
            }
        }
    }
    ensure_tostring_object(node, objects);
}

fn ensure_tostring_object(node: &mut LintNode, objects: &HashSet<String>) {
    let Some(callee) = node.children.get("callee") else {
        return;
    };
    if callee.kind != "MemberExpression" {
        return;
    }
    let property = callee
        .child("property")
        .and_then(|property| property.field_str("name"))
        .map(str::to_owned);
    let name = callee
        .child("object")
        .and_then(|object| object.field_str("name"))
        .map(str::to_owned);
    if !matches!(property.as_deref(), Some("toString" | "toLocaleString")) {
        return;
    }
    let Some(name) = name else {
        return;
    };
    if !objects.contains(&name) {
        return;
    }
    let Some(object) = node
        .children
        .get_mut("callee")
        .and_then(|callee| callee.children.get_mut("object"))
    else {
        return;
    };
    if needs_object_fact(&object.type_texts) {
        object.type_texts = vec!["object".to_owned()];
        object.property_names.clear();
    }
}

fn needs_object_fact(texts: &[String]) -> bool {
    texts.is_empty()
        || texts.iter().all(|text| {
            let trimmed = text.trim();
            !trimmed.starts_with('{') && trimmed != "object" && trimmed != "Object"
        })
}

fn fill_assertion(node: &mut LintNode, bindings: &HashMap<String, String>) {
    let Some(annotation) = node.field_str("__typeAnnotationText").map(str::trim) else {
        return;
    };
    let Some(expression) = node.child("expression") else {
        return;
    };
    let from_checker = expression
        .type_texts
        .iter()
        .any(|text| same_type(text, annotation));
    let from_binding = expression.field_str("name").is_some_and(|name| {
        bindings
            .get(name)
            .is_some_and(|bound| same_type(bound, annotation))
    });
    if from_checker || from_binding {
        node.fields.insert("__typeIsUnchanged".into(), json!(true));
    }
}

fn fill_enum_member(node: &mut LintNode, enums: &HashMap<String, HashMap<String, String>>) {
    if node.field_bool("computed").unwrap_or(false) {
        return;
    }
    let Some(enum_name) = node
        .child("object")
        .and_then(|object| object.field_str("name"))
        .map(str::to_owned)
    else {
        return;
    };
    let Some(members) = enums.get(&enum_name) else {
        return;
    };
    let Some(member) = node
        .child("property")
        .and_then(|property| property.field_str("name"))
        .map(str::to_owned)
    else {
        return;
    };
    let Some(kind) = members.get(&member) else {
        return;
    };
    node.fields
        .insert("__enumTypeIds".into(), json!([enum_name]));
    node.fields
        .insert("__unionPartEnumValueKinds".into(), json!([kind]));
}

fn fill_primitive_side(node: &mut LintNode) {
    if number_like(&node.type_texts) {
        node.fields.insert("__isNumberLike".into(), json!(true));
    }
    if string_like(&node.type_texts) {
        node.fields.insert("__isStringLike".into(), json!(true));
    }
}

fn callee_name(node: &LintNode) -> Option<String> {
    node.child("callee")
        .and_then(|callee| callee.field_str("name"))
        .map(str::to_owned)
}

fn same_type(text: &str, annotation: &str) -> bool {
    let text = text.trim();
    let annotation = annotation.trim();
    text == annotation
        || (annotation == "number" && number_like(&[text.to_owned()]))
        || (annotation == "string" && string_like(&[text.to_owned()]))
}

fn number_like(texts: &[String]) -> bool {
    texts.iter().any(|text| {
        text.split('|').any(|part| {
            let part = part.trim();
            part == "number" || part.parse::<f64>().is_ok()
        })
    })
}

fn string_like(texts: &[String]) -> bool {
    texts.iter().any(|text| {
        text.split('|').any(|part| {
            let part = part.trim();
            part == "string" || part.starts_with('"') || part.starts_with('\'')
        })
    })
}
