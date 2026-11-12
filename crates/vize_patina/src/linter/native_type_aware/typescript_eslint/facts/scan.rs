//! Same-file names the host can resolve without the checker.
use serde_json::Value;
use vize_l0::{FxHashMap, String};

pub(super) struct LocalFn {
    pub(super) value_params: Vec<Option<String>>,
    pub(super) returns: Option<String>,
}

pub(super) fn local_functions(program: &Value, source: &str) -> FxHashMap<String, LocalFn> {
    let mut functions = FxHashMap::default();
    walk_functions(program, source, &mut functions);
    functions
}

fn walk_functions(value: &Value, source: &str, functions: &mut FxHashMap<String, LocalFn>) {
    if value.get("type").and_then(Value::as_str) == Some("FunctionDeclaration")
        && let Some(name) = value
            .get("id")
            .and_then(|id| id.get("name"))
            .and_then(Value::as_str)
    {
        functions.insert(
            String::from(name),
            LocalFn {
                value_params: value_param_texts(value, source),
                returns: return_annotation(value, source),
            },
        );
    }
    match value {
        Value::Object(map) => {
            for child in map.values() {
                walk_functions(child, source, functions);
            }
        }
        Value::Array(items) => {
            for item in items {
                walk_functions(item, source, functions);
            }
        }
        _ => {}
    }
}

fn slice_node(source: &str, value: &Value) -> Option<String> {
    let start = value.get("start").and_then(Value::as_u64)? as usize;
    let end = value.get("end").and_then(Value::as_u64)? as usize;
    source.get(start..end).map(String::from)
}

fn value_param_texts(value: &Value, source: &str) -> Vec<Option<String>> {
    let Some(params) = value.get("params").and_then(Value::as_array) else {
        return Vec::new();
    };
    params
        .iter()
        .map(|param| annotation_text(param, source))
        .collect()
}

fn return_annotation(value: &Value, source: &str) -> Option<String> {
    value
        .get("returnType")
        .and_then(|node| annotation_text(node, source))
}

pub(super) fn annotation_text(value: &Value, source: &str) -> Option<String> {
    match value.get("type").and_then(Value::as_str) {
        Some("AssignmentPattern") => annotation_text(value.get("left")?, source),
        Some("RestElement" | "TSParameterProperty") => annotation_text(
            value.get("argument").or_else(|| value.get("parameter"))?,
            source,
        ),
        Some("TSTypeAnnotation") => {
            let inner = value.get("typeAnnotation").unwrap_or(value);
            slice_node(source, inner)
        }
        _ => {
            let annotation = value.get("typeAnnotation")?;
            let inner = annotation.get("typeAnnotation").unwrap_or(annotation);
            slice_node(source, inner)
        }
    }
}
