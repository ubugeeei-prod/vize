use vize_carton::{String, ToCompactString};

pub(super) fn combine_runtime_js_types(types: impl IntoIterator<Item = String>) -> String {
    let mut js_types: Vec<String> = Vec::new();
    let mut saw_unknown = false;
    for js_type in types {
        if js_type == "null" {
            saw_unknown = true;
            continue;
        }
        push_runtime_js_type(&mut js_types, &js_type);
    }

    if saw_unknown && !js_types.iter().any(|js_type| js_type.as_str() == "Boolean") {
        return "null".to_compact_string();
    }

    match js_types.len() {
        0 => "null".to_compact_string(),
        1 => js_types.pop().unwrap_or_else(|| "null".to_compact_string()),
        _ => {
            let joined = js_types.join(", ");
            let mut result = String::with_capacity(joined.len() + 2);
            result.push('[');
            result.push_str(&joined);
            result.push(']');
            result
        }
    }
}

fn push_runtime_js_type(js_types: &mut Vec<String>, js_type: &str) {
    if js_type.starts_with('[') && js_type.ends_with(']') {
        let inner = &js_type[1..js_type.len() - 1];
        for part in inner
            .split(',')
            .map(str::trim)
            .filter(|part| !part.is_empty())
        {
            push_runtime_js_type(js_types, part);
        }
        return;
    }

    if !js_types.iter().any(|known| known.as_str() == js_type) {
        js_types.push(js_type.into());
    }
}

pub(super) fn combine_runtime_intersection_types(
    types: impl IntoIterator<Item = String>,
) -> String {
    let mut primitives: Vec<String> = Vec::new();
    let mut concrete_objects: Vec<String> = Vec::new();
    let mut has_object_runtime = false;
    for js_type in types {
        match js_type.as_str() {
            "String" | "Number" | "Boolean" | "Symbol" | "BigInt" => {
                push_unique_type(&mut primitives, &js_type);
            }
            "Object" => has_object_runtime = true,
            "Array" | "Function" => push_unique_type(&mut concrete_objects, &js_type),
            _ if js_type != "null" => push_unique_type(&mut concrete_objects, &js_type),
            _ => {}
        }
    }

    if primitives.len() == 1 {
        let primitive = primitives
            .pop()
            .unwrap_or_else(|| "null".to_compact_string());
        return primitive;
    }

    if primitives.is_empty() && !concrete_objects.is_empty() {
        return combine_runtime_js_types(concrete_objects);
    }

    if has_object_runtime {
        "Object".to_compact_string()
    } else {
        "null".to_compact_string()
    }
}

fn push_unique_type(types: &mut Vec<String>, js_type: &str) {
    if !types.iter().any(|known| known.as_str() == js_type) {
        types.push(js_type.into());
    }
}
