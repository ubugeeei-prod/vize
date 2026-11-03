use vize_carton::{String, ToCompactString};

/// Placeholder member for an unresolvable union member kept beside `Boolean`
/// or `Function`. [`split_skip_check`] lowers it to Vue's `skipCheck: true`.
pub(crate) const UNKNOWN_MEMBER: &str = "?";

/// Combine union members like Vue's `resolveRuntimePropsFromType`: an
/// unresolvable member (`null`) erases the union to `null`, unless `Boolean`
/// or `Function` survives. Those keep casting and function defaults, so the
/// known members stay and the runtime type check is skipped.
pub(crate) fn combine_runtime_js_types(types: impl IntoIterator<Item = String>) -> String {
    let mut js_types: Vec<String> = Vec::new();
    let mut saw_unknown = false;
    for js_type in types {
        if js_type == "null" {
            saw_unknown = true;
            continue;
        }
        saw_unknown |= push_runtime_js_type(&mut js_types, &js_type);
    }

    if saw_unknown {
        if !js_types
            .iter()
            .any(|js_type| matches!(js_type.as_str(), "Boolean" | "Function"))
        {
            return "null".to_compact_string();
        }
        js_types.push(UNKNOWN_MEMBER.to_compact_string());
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

/// Lower a combined runtime type to its emitted constructor and whether the
/// runtime check is skipped: `[Boolean, ?]` becomes (`Boolean`, true).
pub(crate) fn split_skip_check(js_type: &str) -> (String, bool) {
    let Some(inner) = js_type.strip_prefix('[').and_then(|s| s.strip_suffix(']')) else {
        return (js_type.to_compact_string(), false);
    };
    let members = inner
        .split(',')
        .map(str::trim)
        .filter(|member| !member.is_empty())
        .collect::<Vec<_>>();
    if !members.contains(&UNKNOWN_MEMBER) {
        return (js_type.to_compact_string(), false);
    }
    let known = members
        .into_iter()
        .filter(|member| *member != UNKNOWN_MEMBER)
        .collect::<Vec<_>>();
    let lowered = match known.as_slice() {
        [only] => (*only).to_compact_string(),
        _ => {
            let joined = known.join(", ");
            let mut result = String::with_capacity(joined.len() + 2);
            result.push('[');
            result.push_str(&joined);
            result.push(']');
            result
        }
    };
    (lowered, true)
}

/// Re-attach a skipped check to a lowered constructor.
pub(crate) fn join_skip_check(js_type: &str, skip_check: bool) -> String {
    if skip_check {
        combine_runtime_js_types([js_type.to_compact_string(), "null".to_compact_string()])
    } else {
        js_type.to_compact_string()
    }
}

/// Push a member (flattening `[A, B]`); returns whether it carried an unknown.
fn push_runtime_js_type(js_types: &mut Vec<String>, js_type: &str) -> bool {
    if let Some(inner) = js_type.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        let mut unknown = false;
        for part in inner
            .split(',')
            .map(str::trim)
            .filter(|part| !part.is_empty())
        {
            unknown |= push_runtime_js_type(js_types, part);
        }
        return unknown;
    }

    if js_type == UNKNOWN_MEMBER {
        return true;
    }
    if !js_types.iter().any(|known| known.as_str() == js_type) {
        js_types.push(js_type.into());
    }
    false
}

pub(super) fn combine_runtime_intersection_types(
    types: impl IntoIterator<Item = String>,
) -> String {
    let mut primitives: Vec<String> = Vec::new();
    let mut concrete_objects: Vec<String> = Vec::new();
    let mut has_object_runtime = false;
    for js_type in types {
        // Like Vue, intersections drop unresolvable members.
        let (js_type, _) = split_skip_check(&js_type);
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
