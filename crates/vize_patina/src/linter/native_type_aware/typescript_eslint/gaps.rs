//! The two typescript-eslint checks corsa does not ship.
use super::tree::Collected;
use corsa::lint::LintNode;
use serde_json::Value;
use std::collections::HashSet;

pub(super) struct Gap {
    pub rule: &'static str,
    pub message: &'static str,
    pub start: u32,
    pub end: u32,
}

pub(super) fn reports(collected: &Collected) -> Vec<Gap> {
    let enums = enum_names(&collected.program);
    let mut gaps = Vec::new();
    for node in &collected.nodes {
        if node.kind == "VariableDeclarator" {
            if let Some(gap) = unsafe_enum(node, &enums) {
                gaps.push(gap);
            }
        }
        if node.kind == "TSTypeAliasDeclaration" {
            if let Some(gap) = generated_empty(node) {
                gaps.push(gap);
            }
        }
    }
    gaps
}

fn unsafe_enum(node: &LintNode, enums: &HashSet<String>) -> Option<Gap> {
    let id = node.child("id")?;
    let name = referenced_name(id.child("typeAnnotation")?)?;
    if !enums.contains(&name) {
        return None;
    }
    let init = node.child("init")?;
    if init.kind != "Literal" {
        return None;
    }
    let literal = init.fields.get("value")?;
    if !literal.is_number() && !literal.is_string() {
        return None;
    }
    Some(Gap {
        rule: "type/no-unsafe-enum-assignment",
        message: "Unsafe assignment of a non-enum value.",
        start: id.range.start,
        end: id.range.end,
    })
}

fn generated_empty(node: &LintNode) -> Option<Gap> {
    let annotation = node.child("typeAnnotation")?;
    let inner = annotation.child("typeAnnotation").unwrap_or(annotation);
    if inner.kind == "TSTypeLiteral" {
        return None;
    }
    let empty = inner
        .type_texts
        .iter()
        .chain(
            node.child("id")
                .map(|id| id.type_texts.as_slice())
                .into_iter()
                .flatten(),
        )
        .any(|text| empty_object_text(text))
        || omit_drops_every_key(inner);
    if !empty {
        return None;
    }
    Some(Gap {
        rule: "type/no-generated-empty-object-type",
        message: "Do not generate an empty object type.",
        start: inner.range.start,
        end: inner.range.end,
    })
}

fn empty_object_text(text: &str) -> bool {
    let compact: String = text.chars().filter(|ch| !ch.is_whitespace()).collect();
    compact == "{}" || compact == "Record<string,never>" || compact == "Record<never,never>"
}

fn omit_drops_every_key(node: &LintNode) -> bool {
    if node.kind != "TSTypeReference" {
        return false;
    }
    let name = node
        .child("typeName")
        .and_then(|type_name| type_name.field_str("name"));
    if name != Some("Omit") {
        return false;
    }
    let Some(params) = node
        .child("typeArguments")
        .or_else(|| node.child("typeParameters"))
        .and_then(|list| list.child_list("params"))
    else {
        return false;
    };
    if params.len() < 2 {
        return false;
    }
    let keys = object_keys(&params[0]);
    if keys.is_empty() {
        return false;
    }
    let dropped = literal_keys(&params[1]);
    keys.iter()
        .all(|key| dropped.iter().any(|item| item == key))
}

fn object_keys(node: &LintNode) -> Vec<String> {
    if node.kind != "TSTypeLiteral" {
        return Vec::new();
    }
    node.child_list("members")
        .unwrap_or(&[])
        .iter()
        .filter_map(|member| {
            member
                .child("key")
                .and_then(|key| key.field_str("name"))
                .map(str::to_owned)
        })
        .collect()
}

fn literal_keys(node: &LintNode) -> Vec<String> {
    match node.kind.as_str() {
        "TSUnionType" => node
            .child_list("types")
            .unwrap_or(&[])
            .iter()
            .flat_map(literal_keys)
            .collect(),
        "TSLiteralType" => node
            .child("literal")
            .and_then(|literal| literal.field_str("value"))
            .map(|value| vec![value.to_owned()])
            .unwrap_or_default(),
        "Literal" => node
            .field_str("value")
            .map(|value| vec![value.to_owned()])
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn referenced_name(annotation: &LintNode) -> Option<String> {
    let inner = annotation.child("typeAnnotation").unwrap_or(annotation);
    if inner.kind != "TSTypeReference" {
        return None;
    }
    let name = inner.child("typeName").unwrap_or(inner);
    name.field_str("name")
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
}

fn enum_names(value: &Value) -> HashSet<String> {
    let mut names = HashSet::new();
    walk_enums(value, &mut names);
    names
}

fn walk_enums(value: &Value, names: &mut HashSet<String>) {
    if value.get("type").and_then(Value::as_str) == Some("TSEnumDeclaration") {
        if let Some(name) = value
            .get("id")
            .and_then(|id| id.get("name"))
            .and_then(Value::as_str)
        {
            names.insert(name.to_owned());
        }
    }
    match value {
        Value::Object(map) => {
            for child in map.values() {
                walk_enums(child, names);
            }
        }
        Value::Array(items) => {
            for item in items {
                walk_enums(item, names);
            }
        }
        _ => {}
    }
}
