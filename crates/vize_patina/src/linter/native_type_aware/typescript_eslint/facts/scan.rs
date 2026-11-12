//! Same-file names the host can resolve without the checker.
use serde_json::Value;
use vize_l0::{FxHashMap, FxHashSet, String};

pub(super) struct LocalFn {
    pub(super) params: Vec<(String, Option<String>)>,
    pub(super) value_params: Vec<Option<String>>,
    pub(super) returns: Option<String>,
}

pub(super) fn deprecated_names(source: &str) -> FxHashSet<String> {
    let mut names = FxHashSet::default();
    let mut rest = source;
    while let Some(at) = rest.find("@deprecated") {
        rest = rest.get(at + "@deprecated".len()..).unwrap_or_default();
        if let Some(name) = rest.split("function").nth(1).and_then(|tail| {
            tail.trim_start()
                .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_' && ch != '$')
                .next()
        }) && !name.is_empty()
        {
            names.insert(String::from(name));
        }
    }
    names
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
                params: type_params(value, source),
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

fn type_params(value: &Value, source: &str) -> Vec<(String, Option<String>)> {
    let list = value
        .get("typeParameters")
        .or_else(|| value.get("typeArguments"));
    let Some(params) = list
        .and_then(|list| list.get("params"))
        .and_then(Value::as_array)
    else {
        return Vec::new();
    };
    params
        .iter()
        .filter_map(|param| {
            let name = param.get("name").and_then(|name| {
                name.as_str()
                    .or_else(|| name.get("name").and_then(Value::as_str))
            })?;
            let default = param
                .get("default")
                .and_then(|default| slice_node(source, default));
            Some((String::from(name), default))
        })
        .collect()
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

pub(super) fn class_methods(program: &Value) -> FxHashMap<String, FxHashSet<String>> {
    let mut methods = FxHashMap::default();
    walk_classes(program, &mut methods);
    methods
}

fn walk_classes(value: &Value, methods: &mut FxHashMap<String, FxHashSet<String>>) {
    if value.get("type").and_then(Value::as_str) == Some("ClassDeclaration")
        && let Some(name) = value
            .get("id")
            .and_then(|id| id.get("name"))
            .and_then(Value::as_str)
    {
        let mut names = FxHashSet::default();
        if let Some(body) = value
            .get("body")
            .and_then(|body| body.get("body"))
            .and_then(Value::as_array)
        {
            for member in body {
                if member.get("type").and_then(Value::as_str) == Some("MethodDefinition")
                    && let Some(method) = member
                        .get("key")
                        .and_then(|key| key.get("name"))
                        .and_then(Value::as_str)
                {
                    names.insert(String::from(method));
                }
            }
        }
        methods.insert(String::from(name), names);
    }
    match value {
        Value::Object(map) => {
            for child in map.values() {
                walk_classes(child, methods);
            }
        }
        Value::Array(items) => {
            for item in items {
                walk_classes(item, methods);
            }
        }
        _ => {}
    }
}

pub(super) fn class_instances(program: &Value) -> FxHashMap<String, String> {
    let mut instances = FxHashMap::default();
    walk_instances(program, &mut instances);
    instances
}

fn walk_instances(value: &Value, instances: &mut FxHashMap<String, String>) {
    if value.get("type").and_then(Value::as_str) == Some("VariableDeclarator") {
        let name = value
            .get("id")
            .and_then(|id| id.get("name"))
            .and_then(Value::as_str);
        let class_name = value
            .get("init")
            .filter(|init| init.get("type").and_then(Value::as_str) == Some("NewExpression"))
            .and_then(|init| init.get("callee"))
            .and_then(|callee| callee.get("name"))
            .and_then(Value::as_str);
        if let (Some(name), Some(class_name)) = (name, class_name) {
            instances.insert(String::from(name), String::from(class_name));
        }
    }
    match value {
        Value::Object(map) => {
            for child in map.values() {
                walk_instances(child, instances);
            }
        }
        Value::Array(items) => {
            for item in items {
                walk_instances(item, instances);
            }
        }
        _ => {}
    }
}
