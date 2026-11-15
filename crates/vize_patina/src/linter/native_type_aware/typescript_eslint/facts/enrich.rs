//! Facts filled in after the checker replies.
use super::scan::{self, LocalFn};
use corsa::lint::LintNode;
use serde_json::{Value, json};
use vize_l0::{FxHashMap, FxHashSet, String};

const ENUM_LITERAL: u32 = 1 << 15;
const NUMBER_LITERAL: u32 = 1 << 11;

pub(super) fn apply(source: &str, nodes: &mut [LintNode], program: &Value) {
    let functions = scan::local_functions(program, source);
    let enums = super::shape::enum_members(program);
    let objects = super::shape::object_literal_bindings(program);
    let bindings = super::shape::annotated_bindings(program, source);
    let binding_enums = super::shape::annotated_binding_enum_parts(program, &enums, &bindings);
    for node in nodes {
        walk(
            node,
            &functions,
            &enums,
            &objects,
            &bindings,
            &binding_enums,
        );
    }
}

fn walk(
    node: &mut LintNode,
    functions: &FxHashMap<String, LocalFn>,
    enums: &FxHashMap<String, FxHashMap<String, String>>,
    objects: &FxHashSet<String>,
    bindings: &FxHashMap<String, String>,
    binding_enums: &FxHashMap<String, Vec<super::shape::AuthoredEnumPart>>,
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
    fill_authored_enum_binding(node, binding_enums);
    for child in node.children.values_mut() {
        walk(child, functions, enums, objects, bindings, binding_enums);
    }
    for children in node.child_lists.values_mut() {
        for child in children {
            walk(child, functions, enums, objects, bindings, binding_enums);
        }
    }
}

fn fill_authored_enum_binding(
    node: &mut LintNode,
    bindings: &FxHashMap<String, Vec<super::shape::AuthoredEnumPart>>,
) {
    if node.kind != "Identifier" || detail_has_enum(node) {
        return;
    }
    let Some(parts) = node.field_str("name").and_then(|name| bindings.get(name)) else {
        return;
    };
    let mut enum_ids = Vec::new();
    let mut enum_kinds = Vec::new();
    let mut part_ids = Vec::new();
    for part in parts {
        if !enum_ids.contains(&part.enum_id) {
            enum_ids.push(part.enum_id.clone());
        }
        if !enum_kinds.contains(&part.kind) {
            enum_kinds.push(part.kind.clone());
        }
        let flags = ENUM_LITERAL
            | (part.kind == "number")
                .then_some(NUMBER_LITERAL)
                .unwrap_or_default();
        let id = vize_l0::cstr!("authored-enum:{}:{}", part.enum_id, part.kind);
        part_ids.push(id.clone());
        if let Some(detail_parts) = node
            .fields
            .get_mut("__typeDetail")
            .and_then(|detail| detail.get_mut("parts"))
            .and_then(serde_json::Value::as_array_mut)
        {
            detail_parts.push(json!({
                "id": id,
                "flags": flags,
                "memberFlags": flags,
                "enumId": part.enum_id,
                "enumText": part.enum_id,
            }));
        }
    }
    merge_string_facts(node, "__enumTypeIds", enum_ids);
    merge_string_facts(node, "__unionPartTypeIds", part_ids);
    merge_string_facts(node, "__unionPartEnumValueKinds", enum_kinds);
}

fn merge_string_facts(node: &mut LintNode, key: &str, facts: Vec<String>) {
    let values = node.fields.entry(key.into()).or_insert_with(|| json!([]));
    let Some(values) = values.as_array_mut() else {
        return;
    };
    for fact in facts {
        if !values.iter().any(|value| value.as_str() == Some(&fact)) {
            values.push(Value::String(fact.as_str().to_owned()));
        }
    }
}

fn detail_has_enum(node: &LintNode) -> bool {
    node.fields
        .get("__typeDetail")
        .and_then(|detail| detail.get("parts"))
        .and_then(serde_json::Value::as_array)
        .is_some_and(|parts| {
            parts.iter().any(|part| {
                part.get("flags")
                    .and_then(serde_json::Value::as_u64)
                    .is_some_and(|flags| flags & u64::from(ENUM_LITERAL) != 0)
            })
        })
}

fn fill_call(
    node: &mut LintNode,
    functions: &FxHashMap<String, LocalFn>,
    objects: &FxHashSet<String>,
) {
    let Some(name) = callee_name(node) else {
        ensure_tostring_object(node, objects);
        return;
    };
    if let Some(returns) = functions
        .get(&name)
        .and_then(|function| function.returns.clone())
        && (returns == "void" || returns == "undefined")
        && !node.type_texts.iter().any(|text| {
            let text = text.trim();
            text == "void" || text == "undefined"
        })
    {
        node.type_texts = vec![returns.into()];
    }
    if !node.fields.contains_key("__expectedArgumentTypeTexts")
        && let Some(function) = functions.get(&name)
    {
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
    ensure_tostring_object(node, objects);
}

fn ensure_tostring_object(node: &mut LintNode, objects: &FxHashSet<String>) {
    let Some(callee) = node.children.get("callee") else {
        return;
    };
    if callee.kind != "MemberExpression" {
        return;
    }
    let property = callee
        .child("property")
        .and_then(|property| property.field_str("name"))
        .map(String::from);
    let name = callee
        .child("object")
        .and_then(|object| object.field_str("name"))
        .map(String::from);
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

fn needs_object_fact(texts: &[impl AsRef<str>]) -> bool {
    texts.is_empty()
        || texts.iter().all(|text| {
            let trimmed = text.as_ref().trim();
            !trimmed.starts_with('{') && trimmed != "object" && trimmed != "Object"
        })
}

fn fill_assertion(node: &mut LintNode, bindings: &FxHashMap<String, String>) {
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

fn fill_enum_member(node: &mut LintNode, enums: &FxHashMap<String, FxHashMap<String, String>>) {
    if node.field_bool("computed").unwrap_or(false) {
        return;
    }
    let Some(enum_name) = node
        .child("object")
        .and_then(|object| object.field_str("name"))
        .map(String::from)
    else {
        return;
    };
    let Some(members) = enums.get(&enum_name) else {
        return;
    };
    let Some(member) = node
        .child("property")
        .and_then(|property| property.field_str("name"))
        .map(String::from)
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
        .map(String::from)
}

fn same_type(text: &str, annotation: &str) -> bool {
    let text = text.trim();
    let annotation = annotation.trim();
    text == annotation
        || (annotation == "number" && number_like(&[text]))
        || (annotation == "string" && string_like(&[text]))
}

fn number_like(texts: &[impl AsRef<str>]) -> bool {
    texts.iter().any(|text| {
        text.as_ref().split('|').any(|part| {
            let part = part.trim();
            part == "number" || part.parse::<f64>().is_ok()
        })
    })
}

fn string_like(texts: &[impl AsRef<str>]) -> bool {
    texts.iter().any(|text| {
        text.as_ref().split('|').any(|part| {
            let part = part.trim();
            part == "string" || part.starts_with('"') || part.starts_with('\'')
        })
    })
}

#[cfg(test)]
mod tests {
    use super::super::super::tree;
    use super::apply;
    use serde_json::{Value, json};

    fn enum_ids(source: &str) -> Vec<String> {
        let mut collected = tree::collect(source, false, &["BinaryExpression"]);
        let Some(left) = collected
            .nodes
            .first_mut()
            .and_then(|node| node.children.get_mut("left"))
        else {
            return Vec::new();
        };
        left.fields.insert(
            "__typeDetail".into(),
            json!({
                "id": "projected",
                "flags": 1,
                "parts": [{ "id": "projected", "flags": 1 }],
            }),
        );
        let program = collected.program.clone();
        apply(source, &mut collected.nodes, &program);
        collected
            .nodes
            .first()
            .and_then(|node| node.child("left"))
            .and_then(|left| left.fields.get("__typeDetail"))
            .and_then(|detail| detail.get("parts"))
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|part| part.get("enumId").and_then(Value::as_str))
            .map(String::from)
            .collect()
    }

    #[test]
    fn direct_union_enum_constituent_repairs_a_projected_binding() {
        let source = "enum Fruit { Apple = 'apple' }\ntype Branded = (string & { brand: void }) | Fruit;\ndeclare const value: Branded;\nvalue === 'other';";
        assert_eq!(enum_ids(source), ["Fruit"]);
    }

    #[test]
    fn branded_string_without_an_enum_is_not_promoted() {
        let source = "type Branded = string & { brand: void };\ndeclare const value: Branded;\nvalue === 'other';";
        assert!(enum_ids(source).is_empty());
    }
}
