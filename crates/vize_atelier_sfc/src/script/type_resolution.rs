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

    // Built-in TS utility types (`Partial<T>`, `Pick<T, K>`, ...) are not stored
    // in the interface/alias maps, so resolve them structurally by transforming
    // the member set of their inner type argument.
    if let Some(body) = resolve_utility_type(trimmed, interfaces, type_aliases, stack) {
        return Some(body);
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

/// A single object member parsed out of a resolved type-literal body, e.g. the
/// `a?: string` in `{ a?: string; b: number }`.
struct Member {
    key: String,
    optional: bool,
    readonly: bool,
    ty: String,
}

/// Resolve a TS built-in utility type (`Partial`/`Required`/`Readonly`/`Pick`/
/// `Omit`/`Record`) to an object-literal member body, or `None` when `type_expr`
/// is not one of those (or its argument can't be resolved structurally).
fn resolve_utility_type(
    type_expr: &str,
    interfaces: &FxHashMap<String, String>,
    type_aliases: &FxHashMap<String, String>,
    stack: &mut FxHashSet<String>,
) -> Option<String> {
    let (name, args) = split_generic_call(type_expr)?;
    let type_args = split_top_level(args, ',');

    match name {
        "Partial" | "Required" | "Readonly" => {
            let inner = type_args.first()?;
            let body = resolve_type_to_object_body_inner(inner, interfaces, type_aliases, stack)?;
            let mut members = parse_members(&body);
            for member in &mut members {
                match name {
                    "Partial" => member.optional = true,
                    "Required" => member.optional = false,
                    "Readonly" => member.readonly = true,
                    _ => unreachable!(),
                }
            }
            Some(render_members(&members))
        }
        "Pick" | "Omit" => {
            let inner = type_args.first()?;
            let keys_arg = type_args.get(1)?;
            let body = resolve_type_to_object_body_inner(inner, interfaces, type_aliases, stack)?;
            let keys = parse_string_literal_union(keys_arg);
            let members: Vec<Member> = parse_members(&body)
                .into_iter()
                .filter(|m| {
                    let contains = keys.iter().any(|k| k == m.key.as_str());
                    if name == "Pick" { contains } else { !contains }
                })
                .collect();
            Some(render_members(&members))
        }
        "Record" => {
            let keys_arg = type_args.first()?;
            let value_arg = type_args.get(1)?;
            let keys = parse_string_literal_union(keys_arg);
            if keys.is_empty() {
                return None;
            }
            let value = value_arg.trim();
            let members: Vec<Member> = keys
                .into_iter()
                .map(|key| Member {
                    key,
                    optional: false,
                    readonly: false,
                    ty: value.to_compact_string(),
                })
                .collect();
            Some(render_members(&members))
        }
        _ => None,
    }
}

/// Split a generic type reference `Name<args>` into `("Name", "args")`.
fn split_generic_call(type_expr: &str) -> Option<(&str, &str)> {
    let open = type_expr.find('<')?;
    if !type_expr.trim_end().ends_with('>') {
        return None;
    }
    let close = type_expr.rfind('>')?;
    if close <= open {
        return None;
    }
    let name = type_expr[..open].trim();
    let args = type_expr[open + 1..close].trim();
    Some((name, args))
}

/// Parse a resolved object-literal member body (`a?: string; b: number`) into
/// individual members.
fn parse_members(body: &str) -> Vec<Member> {
    let mut members = Vec::new();
    for part in split_top_level(body, ';') {
        for sub in split_top_level(&part, ',') {
            if let Some(member) = parse_member(&sub) {
                members.push(member);
            }
        }
    }
    members
}

fn parse_member(part: &str) -> Option<Member> {
    let mut text = part.trim();
    if text.is_empty() {
        return None;
    }

    let mut readonly = false;
    if let Some(rest) = text.strip_prefix("readonly ") {
        readonly = true;
        text = rest.trim();
    }

    let colon = find_top_level_colon(text)?;
    let mut key = text[..colon].trim();
    let ty = text[colon + 1..].trim();

    let optional = key.ends_with('?');
    if optional {
        key = key[..key.len() - 1].trim();
    }
    if key.is_empty() || ty.is_empty() {
        return None;
    }

    Some(Member {
        key: key.to_compact_string(),
        optional,
        readonly,
        ty: ty.to_compact_string(),
    })
}

fn find_top_level_colon(text: &str) -> Option<usize> {
    let mut depth = 0i32;
    let mut prev = '\0';
    for (idx, ch) in text.char_indices() {
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
            ':' if depth == 0 => return Some(idx),
            _ => {}
        }
        prev = ch;
    }
    None
}

/// Render members back into a `key: type; ...` body matching the resolver's
/// existing output shape.
fn render_members(members: &[Member]) -> String {
    let mut out = String::default();
    for member in members {
        if !out.is_empty() {
            out.push_str("; ");
        }
        if member.readonly {
            out.push_str("readonly ");
        }
        out.push_str(&member.key);
        if member.optional {
            out.push('?');
        }
        out.push_str(": ");
        out.push_str(&member.ty);
    }
    out
}

/// Parse a string-literal key argument (`'a'` or `'a' | 'b'`) into the set of
/// bare key names. Returns empty when the argument isn't a string-literal union.
fn parse_string_literal_union(arg: &str) -> Vec<String> {
    let mut keys = Vec::new();
    for part in split_top_level(arg, '|') {
        let trimmed = part.trim();
        let unquoted = trimmed
            .strip_prefix('\'')
            .and_then(|s| s.strip_suffix('\''))
            .or_else(|| trimmed.strip_prefix('"').and_then(|s| s.strip_suffix('"')));
        match unquoted {
            Some(key) => keys.push(key.to_compact_string()),
            None => return Vec::new(),
        }
    }
    keys
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
