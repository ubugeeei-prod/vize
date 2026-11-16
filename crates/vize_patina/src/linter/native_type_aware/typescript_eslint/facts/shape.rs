//! Same-file enums and bindings the checker does not have to prove.
use super::scan::annotation_text;
use serde_json::Value;
use vize_l0::{FxHashMap, FxHashSet, String};

/// One same-file enum constituent stated directly by an authored type alias.
#[derive(Clone, Debug)]
pub(super) struct AuthoredEnumPart {
    /// Stable same-file enum name.
    pub enum_id: String,
    /// Primitive value kind carried by the enum constituent.
    pub kind: String,
}

pub(super) fn enum_members(program: &Value) -> FxHashMap<String, FxHashMap<String, String>> {
    let mut enums = FxHashMap::default();
    walk_enums(program, &mut enums);
    enums
}

fn walk_enums(value: &Value, enums: &mut FxHashMap<String, FxHashMap<String, String>>) {
    if value.get("type").and_then(Value::as_str) == Some("TSEnumDeclaration")
        && let Some(name) = value
            .get("id")
            .and_then(|id| id.get("name"))
            .and_then(Value::as_str)
    {
        let mut members = FxHashMap::default();
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
                    members.insert(String::from(member_name), member_kind(member));
                }
            }
        }
        enums.insert(String::from(name), members);
    }
    walk_children(value, |child| walk_enums(child, enums));
}

fn member_kind(member: &Value) -> String {
    let Some(init) = member.get("initializer") else {
        return String::from("number");
    };
    if init.get("value").and_then(Value::as_str).is_some() {
        String::from("string")
    } else {
        String::from("number")
    }
}

pub(super) fn object_literal_bindings(program: &Value) -> FxHashSet<String> {
    let mut names = FxHashSet::default();
    walk_object_bindings(program, &mut names);
    names
}

fn walk_object_bindings(value: &Value, names: &mut FxHashSet<String>) {
    if value.get("type").and_then(Value::as_str) == Some("VariableDeclarator") {
        let init_is_object = value
            .get("init")
            .and_then(|init| init.get("type"))
            .and_then(Value::as_str)
            == Some("ObjectExpression");
        if init_is_object
            && let Some(name) = value
                .get("id")
                .and_then(|id| id.get("name"))
                .and_then(Value::as_str)
        {
            names.insert(String::from(name));
        }
    }
    walk_children(value, |child| walk_object_bindings(child, names));
}

pub(super) fn annotated_bindings(program: &Value, source: &str) -> FxHashMap<String, String> {
    let mut bindings = FxHashMap::default();
    walk_annotated_bindings(program, source, &mut bindings);
    bindings
}

/// Enum constituents stated directly by a local alias on an annotated binding.
///
/// The checker is authoritative whenever it retains those constituents. This
/// source fact only repairs a projection that loses them: aliases must be a
/// direct enum reference or a union with direct enum-reference constituents.
pub(super) fn annotated_binding_enum_parts(
    program: &Value,
    enums: &FxHashMap<String, FxHashMap<String, String>>,
    bindings: &FxHashMap<String, String>,
) -> FxHashMap<String, Vec<AuthoredEnumPart>> {
    if enums.is_empty() || bindings.is_empty() {
        return FxHashMap::default();
    }
    let mut aliases = FxHashMap::default();
    walk_enum_aliases(program, enums, &mut aliases);
    bindings
        .iter()
        .filter_map(|(binding, annotation)| {
            aliases
                .get(annotation.trim())
                .cloned()
                .map(|parts| (binding.clone(), parts))
        })
        .collect()
}

fn walk_enum_aliases(
    value: &Value,
    enums: &FxHashMap<String, FxHashMap<String, String>>,
    aliases: &mut FxHashMap<String, Vec<AuthoredEnumPart>>,
) {
    if value.get("type").and_then(Value::as_str) == Some("TSTypeAliasDeclaration")
        && let Some(name) = value
            .get("id")
            .and_then(|id| id.get("name"))
            .and_then(Value::as_str)
        && let Some(annotation) = value.get("typeAnnotation")
    {
        let constituents = if annotation.get("type").and_then(Value::as_str) == Some("TSUnionType")
        {
            annotation
                .get("types")
                .and_then(Value::as_array)
                .map(|types| types.as_slice())
                .unwrap_or_default()
        } else {
            core::slice::from_ref(annotation)
        };
        let mut parts = Vec::new();
        for constituent in constituents {
            let Some(enum_id) = direct_type_reference(constituent) else {
                continue;
            };
            let Some(members) = enums.get(enum_id) else {
                continue;
            };
            for kind in members.values() {
                if !parts
                    .iter()
                    .any(|part: &AuthoredEnumPart| part.enum_id == enum_id && part.kind == *kind)
                {
                    parts.push(AuthoredEnumPart {
                        enum_id: String::from(enum_id),
                        kind: kind.clone(),
                    });
                }
            }
        }
        if !parts.is_empty() {
            aliases.insert(String::from(name), parts);
        }
    }
    walk_children(value, |child| walk_enum_aliases(child, enums, aliases));
}

fn direct_type_reference(value: &Value) -> Option<&str> {
    if value.get("type").and_then(Value::as_str) != Some("TSTypeReference") {
        return None;
    }
    value
        .get("typeName")
        .filter(|name| name.get("type").and_then(Value::as_str) == Some("Identifier"))?
        .get("name")?
        .as_str()
}

fn walk_annotated_bindings(value: &Value, source: &str, bindings: &mut FxHashMap<String, String>) {
    if value.get("type").and_then(Value::as_str) == Some("VariableDeclarator")
        && let Some(name) = value
            .get("id")
            .and_then(|id| id.get("name"))
            .and_then(Value::as_str)
        && let Some(text) = value.get("id").and_then(|id| annotation_text(id, source))
    {
        bindings.insert(String::from(name), text);
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
