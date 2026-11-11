//! Same-file enums and bindings the checker does not have to prove.
use super::scan::annotation_text;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

pub(super) fn enum_members(program: &Value) -> HashMap<String, HashMap<String, String>> {
    let mut enums = HashMap::new();
    walk_enums(program, &mut enums);
    enums
}

fn walk_enums(value: &Value, enums: &mut HashMap<String, HashMap<String, String>>) {
    if value.get("type").and_then(Value::as_str) == Some("TSEnumDeclaration") {
        if let Some(name) = value
            .get("id")
            .and_then(|id| id.get("name"))
            .and_then(Value::as_str)
        {
            let mut members = HashMap::new();
            if let Some(body) = value
                .get("body")
                .and_then(|body| body.get("members"))
                .or_else(|| value.get("members"))
                .and_then(Value::as_array)
            {
                for member in body {
                    if let Some(member_name) = member
                        .get("id")
                        .and_then(|id| id.get("name"))
                        .and_then(Value::as_str)
                    {
                        members.insert(member_name.to_owned(), member_kind(member));
                    }
                }
            }
            enums.insert(name.to_owned(), members);
        }
    }
    walk_children(value, |child| walk_enums(child, enums));
}

fn member_kind(member: &Value) -> String {
    let Some(init) = member.get("initializer") else {
        return "number".to_owned();
    };
    if init.get("value").and_then(Value::as_str).is_some() {
        "string".to_owned()
    } else {
        "number".to_owned()
    }
}

pub(super) fn object_literal_bindings(program: &Value) -> HashSet<String> {
    let mut names = HashSet::new();
    walk_object_bindings(program, &mut names);
    names
}

fn walk_object_bindings(value: &Value, names: &mut HashSet<String>) {
    if value.get("type").and_then(Value::as_str) == Some("VariableDeclarator") {
        let init_is_object = value
            .get("init")
            .and_then(|init| init.get("type"))
            .and_then(Value::as_str)
            == Some("ObjectExpression");
        if init_is_object {
            if let Some(name) = value
                .get("id")
                .and_then(|id| id.get("name"))
                .and_then(Value::as_str)
            {
                names.insert(name.to_owned());
            }
        }
    }
    walk_children(value, |child| walk_object_bindings(child, names));
}

pub(super) fn annotated_bindings(program: &Value, source: &str) -> HashMap<String, String> {
    let mut bindings = HashMap::new();
    walk_annotated_bindings(program, source, &mut bindings);
    bindings
}

fn walk_annotated_bindings(value: &Value, source: &str, bindings: &mut HashMap<String, String>) {
    if value.get("type").and_then(Value::as_str) == Some("VariableDeclarator") {
        if let Some(name) = value
            .get("id")
            .and_then(|id| id.get("name"))
            .and_then(Value::as_str)
        {
            if let Some(text) = value.get("id").and_then(|id| annotation_text(id, source)) {
                bindings.insert(name.to_owned(), text);
            }
        }
    }
    walk_children(value, |child| {
        walk_annotated_bindings(child, source, bindings)
    });
}

fn walk_children(value: &Value, mut visit: impl FnMut(&Value)) {
    match value {
        Value::Object(map) => {
            for child in map.values() {
                visit(child);
            }
        }
        Value::Array(items) => {
            for item in items {
                visit(item);
            }
        }
        _ => {}
    }
}
