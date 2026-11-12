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

/// Node kinds `no-generated-empty-object-type` listens to.
pub(super) const EMPTY_OBJECT_LISTENERS: [&str; 2] = ["TSIntersectionType", "TSTypeReference"];

pub(super) fn reports(collected: &Collected) -> Vec<Gap> {
    let enums = enum_names(&collected.program);
    let mut gaps = Vec::new();
    for node in &collected.nodes {
        if node.kind == "VariableDeclarator" {
            if let Some(gap) = unsafe_enum(node, &enums) {
                gaps.push(gap);
            }
        }
        if let Some(gap) = generated_empty(node) {
            gaps.push(gap);
        }
    }
    gaps
}

/// typescript-eslint `no-generated-empty-object-type`: an intersection, or a
/// type reference with type arguments outside an intersection, whose type is
/// (or holds) the empty object type. `__emptyObjectType` is the checker's
/// verdict on the type (see `corsa_session::type_node_facts`).
fn generated_empty(node: &LintNode) -> Option<Gap> {
    let inspected = match node.kind.as_str() {
        "TSIntersectionType" => true,
        // The intersection itself is checked, so its members are not.
        "TSTypeReference" => {
            node.child("typeArguments").is_some()
                && node.field_str("__parentKind") != Some("TSIntersectionType")
        }
        _ => false,
    };
    if !inspected || node.field_bool("__emptyObjectType") != Some(true) {
        return None;
    }
    Some(Gap {
        rule: "type/no-generated-empty-object-type",
        message: "This type resolves to `{}`, the empty object type. This was likely not intentional.",
        start: node.range.start,
        end: node.range.end,
    })
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
