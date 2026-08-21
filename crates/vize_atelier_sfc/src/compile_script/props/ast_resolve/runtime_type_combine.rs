use vize_carton::{String, ToCompactString};

pub(super) fn combine_runtime_js_types(types: impl IntoIterator<Item = String>) -> String {
    let mut js_types: Vec<String> = Vec::new();
    let mut saw_unknown = false;
    for js_type in types {
        if js_type == "null" {
            saw_unknown = true;
            continue;
        }
        if !js_types.contains(&js_type) {
            js_types.push(js_type);
        }
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

pub(super) fn combine_runtime_intersection_types(
    types: impl IntoIterator<Item = String>,
) -> String {
    let mut primitives: Vec<String> = Vec::new();
    let mut has_object_runtime = false;
    for js_type in types {
        match js_type.as_str() {
            "String" | "Number" | "Boolean" | "Symbol" | "BigInt" => {
                if !primitives.contains(&js_type) {
                    primitives.push(js_type);
                }
            }
            "Object" | "Array" | "Function" => has_object_runtime = true,
            _ => {}
        }
    }

    if primitives.len() == 1 {
        return primitives
            .pop()
            .unwrap_or_else(|| "null".to_compact_string());
    }

    if has_object_runtime {
        "Object".to_compact_string()
    } else {
        "null".to_compact_string()
    }
}
