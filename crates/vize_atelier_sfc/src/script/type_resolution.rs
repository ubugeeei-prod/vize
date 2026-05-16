use vize_carton::{FxHashMap, FxHashSet, String, ToCompactString};

pub(crate) fn build_interface_type_source(
    source: &str,
    name_end: usize,
    body_start: usize,
    body_end: usize,
) -> String {
    let body = source[body_start..body_end].trim();
    let header = source[name_end..body_start].trim();

    let Some(extends_idx) = header.find("extends") else {
        return body.to_compact_string();
    };

    let extends_clause = header[extends_idx + "extends".len()..].trim();
    if extends_clause.is_empty() {
        return body.to_compact_string();
    }

    let bases = split_top_level(extends_clause, ',');
    if bases.is_empty() {
        return body.to_compact_string();
    }

    let mut merged = String::default();
    for base in bases {
        let trimmed = base.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !merged.is_empty() {
            merged.push_str(" & ");
        }
        merged.push_str(trimmed);
    }

    if !body.is_empty() {
        if !merged.is_empty() {
            merged.push_str(" & ");
        }
        merged.push_str(body);
    }

    if merged.is_empty() {
        body.to_compact_string()
    } else {
        merged
    }
}

pub(crate) fn resolve_type_args(
    type_args: &str,
    interfaces: &FxHashMap<String, String>,
    type_aliases: &FxHashMap<String, String>,
) -> String {
    let content = type_args.trim();
    if content.starts_with('{') {
        return content.to_compact_string();
    }

    let Some(body) = resolve_type_to_object_body(content, interfaces, type_aliases) else {
        return content.to_compact_string();
    };

    let trimmed = body.trim();
    if trimmed.is_empty() {
        return content.to_compact_string();
    }

    let mut result = String::with_capacity(trimmed.len() + 4);
    result.push_str("{ ");
    result.push_str(trimmed);
    result.push_str(" }");
    result
}

pub(crate) fn resolve_single_type_ref(
    name: &str,
    interfaces: &FxHashMap<String, String>,
    type_aliases: &FxHashMap<String, String>,
) -> Option<String> {
    let base_name = strip_generic_params(name);

    if let Some(body) = interfaces.get(base_name) {
        return Some(body.clone());
    }

    if let Some(body) = type_aliases.get(base_name) {
        return Some(body.clone());
    }

    None
}

pub(crate) fn resolve_type_to_object_body(
    type_expr: &str,
    interfaces: &FxHashMap<String, String>,
    type_aliases: &FxHashMap<String, String>,
) -> Option<String> {
    let mut stack = FxHashSet::default();
    resolve_type_to_object_body_inner(type_expr, interfaces, type_aliases, &mut stack)
}

fn resolve_type_to_object_body_inner(
    type_expr: &str,
    interfaces: &FxHashMap<String, String>,
    type_aliases: &FxHashMap<String, String>,
    stack: &mut FxHashSet<String>,
) -> Option<String> {
    let trimmed = type_expr.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        let inner = trimmed[1..trimmed.len() - 1].trim();
        return Some(inner.to_compact_string());
    }

    let parts = split_top_level(trimmed, '&');
    if parts.len() > 1 {
        let mut merged = String::default();
        for part in parts {
            let Some(body) =
                resolve_type_to_object_body_inner(&part, interfaces, type_aliases, stack)
            else {
                continue;
            };
            let body = body.trim();
            if body.is_empty() {
                continue;
            }
            if !merged.is_empty() {
                merged.push_str("; ");
            }
            merged.push_str(body);
        }

        if merged.is_empty() {
            return None;
        }

        return Some(merged);
    }

    let base_name = strip_generic_params(trimmed);
    if !stack.insert(base_name.to_compact_string()) {
        return None;
    }

    let resolved = resolve_single_type_ref(base_name, interfaces, type_aliases)
        .and_then(|body| resolve_type_to_object_body_inner(&body, interfaces, type_aliases, stack));

    stack.remove(base_name);
    resolved
}

fn strip_generic_params(name: &str) -> &str {
    if let Some(idx) = name.find('<') {
        name[..idx].trim()
    } else {
        name.trim()
    }
}

fn split_top_level(input: &str, delimiter: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::default();
    let mut depth = 0i32;
    let mut prev = '\0';

    for ch in input.chars() {
        match ch {
            '{' | '<' | '(' | '[' => depth += 1,
            '}' | ')' | ']' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            '>' => {
                if prev != '=' && depth > 0 {
                    depth -= 1;
                }
            }
            _ => {}
        }

        if ch == delimiter && depth == 0 {
            let trimmed = current.trim();
            if !trimmed.is_empty() {
                parts.push(trimmed.to_compact_string());
            }
            current.clear();
            prev = ch;
            continue;
        }

        current.push(ch);
        prev = ch;
    }

    let trimmed = current.trim();
    if !trimmed.is_empty() {
        parts.push(trimmed.to_compact_string());
    }

    parts
}
