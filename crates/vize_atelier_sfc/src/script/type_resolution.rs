use vize_carton::{FxHashMap, FxHashSet, String, ToCompactString};

mod members;
#[cfg(test)]
use members::strip_comments;
use members::{Member, parse_members, render_members};
mod union;

pub(crate) fn build_interface_type_source(
    source: &str,
    name_end: usize,
    body_start: usize,
    body_end: usize,
) -> String {
    let body = source.get(body_start..body_end).unwrap_or_default().trim();
    let header = source.get(name_end..body_start).unwrap_or_default().trim();

    let Some(extends_clause) = heritage_extends_clause(header) else {
        return body.to_compact_string();
    };

    let extends_clause = extends_clause.trim();
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

/// Return the text after the heritage-clause `extends` in an interface header,
/// skipping the generic parameter list. For
/// `interface Foo<T extends Bar = Bar> extends Pick<Baz, 'x'>` the header is
/// `<T extends Bar = Bar> extends Pick<Baz, 'x'>`; a naive `find("extends")`
/// would hit the type-parameter constraint and mangle the whole clause.
fn heritage_extends_clause(header: &str) -> Option<&str> {
    let bytes = header.as_bytes();
    let mut depth = 0usize;
    let mut i = 0;
    while let Some(rest @ [byte, ..]) = bytes.get(i..) {
        match *byte {
            b'<' => depth += 1,
            b'>' => depth = depth.saturating_sub(1),
            // `=>` inside a generic default (e.g. `<T = () => void>`) must
            // not close an angle-bracket level.
            b'=' if bytes.get(i + 1) == Some(&b'>') => {
                i += 2;
                continue;
            }
            b'e' if depth == 0 && rest.starts_with(b"extends") => {
                let before_ok = i
                    .checked_sub(1)
                    .and_then(|prev| bytes.get(prev))
                    .is_none_or(|&b| !is_identifier_byte(b));
                let after_ok = bytes
                    .get(i + "extends".len())
                    .is_none_or(|&b| !is_identifier_byte(b));
                if before_ok && after_ok {
                    return header.get(i + "extends".len()..);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

pub(crate) fn resolve_type_args(
    type_args: &str,
    interfaces: &FxHashMap<String, String>,
    type_aliases: &FxHashMap<String, String>,
) -> String {
    let content = type_args.trim();
    if content.starts_with('{')
        && (content.contains(" in ") || split_top_level(content, '&').len() == 1)
        && split_top_level(content, '|').len() == 1
    {
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
    resolve_type_to_object_body_inner(
        type_expr,
        interfaces,
        type_aliases,
        &mut stack,
        &mut |_| {},
        false,
    )
}

/// Find unresolved references that determine the property shape, excluding value types.
pub(crate) fn unresolved_property_shape_references(
    type_expr: &str,
    interfaces: &FxHashMap<String, String>,
    type_aliases: &FxHashMap<String, String>,
) -> Vec<String> {
    let mut missing = Vec::new();
    let _ = resolve_type_to_object_body_inner(
        type_expr,
        interfaces,
        type_aliases,
        &mut FxHashSet::default(),
        &mut |name| missing.push(name.to_compact_string()),
        true,
    );
    missing
}

/// Resolve an object body while tracking cycles and optionally reporting missing shapes.
fn resolve_type_to_object_body_inner(
    type_expr: &str,
    interfaces: &FxHashMap<String, String>,
    type_aliases: &FxHashMap<String, String>,
    stack: &mut FxHashSet<String>,
    missing: &mut dyn FnMut(&str),
    collect_missing: bool,
) -> Option<String> {
    let trimmed = type_expr.trim();
    if trimmed.is_empty() {
        return None;
    }

    let union_parts = split_top_level(trimmed, '|');
    if union_parts.len() > 1 {
        return union::resolve_members(
            &union_parts,
            interfaces,
            type_aliases,
            stack,
            missing,
            collect_missing,
        );
    }

    let parts = split_top_level(trimmed, '&');
    if parts.len() > 1 {
        let mut merged = String::default();
        for part in parts {
            let Some(body) = resolve_type_to_object_body_inner(
                &part,
                interfaces,
                type_aliases,
                stack,
                missing,
                collect_missing,
            ) else {
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

    if let Some(inner) = trimmed
        .strip_prefix('(')
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return resolve_type_to_object_body_inner(
            inner,
            interfaces,
            type_aliases,
            stack,
            missing,
            collect_missing,
        );
    }

    if let Some(inner) = trimmed
        .strip_prefix('{')
        .and_then(|rest| rest.strip_suffix('}'))
    {
        return Some(inner.trim().to_compact_string());
    }

    if is_conditional_type(trimmed) {
        let resolved = resolve_conditional_type(
            trimmed,
            None,
            interfaces,
            type_aliases,
            stack,
            missing,
            collect_missing,
        );
        if resolved.is_none() && collect_missing {
            missing(trimmed);
        }
        return resolved;
    }

    // Resolve supported utility types structurally. When their inner shape is
    // missing, the recursive call already records that authored reference;
    // do not report the built-in wrapper itself as unresolved.
    if split_generic_call(trimmed).is_some_and(|(name, _)| is_supported_utility_type(name)) {
        return resolve_utility_type(
            trimmed,
            interfaces,
            type_aliases,
            stack,
            missing,
            collect_missing,
        );
    }

    if let Some((name, args)) = split_generic_call(trimmed)
        && let Some(alias) = type_aliases.get(name)
        && is_conditional_type(alias)
    {
        let type_args = split_top_level(args, ',');
        let checked_parameter = conditional_checked_type(alias);
        if type_args.len() == 1
            && checked_parameter
                .as_deref()
                .is_some_and(is_type_reference_name)
            && stack.insert(trimmed.to_compact_string())
        {
            let resolved = resolve_conditional_type(
                alias,
                type_args.first().map(String::as_str),
                interfaces,
                type_aliases,
                stack,
                missing,
                collect_missing,
            );
            stack.remove(trimmed);
            if resolved.is_some() {
                return resolved;
            }
        }
        if collect_missing {
            missing(trimmed);
        }
        return None;
    }

    let base_name = strip_generic_params(trimmed);
    if !is_type_reference_name(base_name) {
        return None;
    }
    if !stack.insert(base_name.to_compact_string()) {
        return None;
    }

    let resolved = match resolve_single_type_ref(base_name, interfaces, type_aliases) {
        Some(body) => resolve_type_to_object_body_inner(
            &body,
            interfaces,
            type_aliases,
            stack,
            missing,
            collect_missing,
        ),
        None => {
            missing(base_name);
            None
        }
    };

    stack.remove(base_name);
    resolved
}

/// Return whether `type_expr` has a top-level conditional-type branch.
fn is_conditional_type(type_expr: &str) -> bool {
    split_top_level(type_expr, '?').len() == 2
}

/// Extract the checked type from a top-level `T extends U ? X : Y` expression.
fn conditional_checked_type(type_expr: &str) -> Option<String> {
    let condition = split_top_level(type_expr, '?');
    let condition = condition.first()?;
    split_top_level_extends(condition).map(|(checked, _)| checked.to_compact_string())
}

/// Resolve the selected object branch of a supported conditional type.
fn resolve_conditional_type(
    type_expr: &str,
    checked_override: Option<&str>,
    interfaces: &FxHashMap<String, String>,
    type_aliases: &FxHashMap<String, String>,
    stack: &mut FxHashSet<String>,
    missing: &mut dyn FnMut(&str),
    collect_missing: bool,
) -> Option<String> {
    let condition = split_top_level(type_expr, '?');
    let (checked, constraint) = split_top_level_extends(condition.first()?)?;
    let branches = split_top_level(condition.get(1)?, ':');
    if branches.len() != 2 {
        return None;
    }

    let checked = checked_override.unwrap_or(checked);
    let mut ignored_missing = |_: &str| {};
    let checked_body = resolve_type_to_object_body_inner(
        checked,
        interfaces,
        type_aliases,
        stack,
        &mut ignored_missing,
        false,
    )?;
    let mut inferred = FxHashMap::default();
    let condition_matches = if constraint.trim() == "object" {
        true
    } else {
        let pattern = constraint.trim().strip_prefix('{')?.strip_suffix('}')?;
        let actual_members = parse_members(&checked_body);
        let mut matches = true;
        for expected in parse_members(pattern) {
            let Some(actual) = actual_members
                .iter()
                .find(|member| member.key == expected.key)
            else {
                matches = false;
                break;
            };
            if let Some(name) = expected.ty.trim().strip_prefix("infer ") {
                if !is_type_reference_name(name) {
                    return None;
                }
                inferred.insert(name.to_compact_string(), actual.ty.clone());
            } else if expected.ty != actual.ty {
                return None;
            }
        }
        matches
    };

    let selected = if condition_matches {
        branches.first()?
    } else {
        branches.get(1)?
    };
    let selected = inferred
        .get(selected.trim())
        .map_or(selected.as_str(), String::as_str);
    resolve_type_to_object_body_inner(
        selected,
        interfaces,
        type_aliases,
        stack,
        missing,
        collect_missing,
    )
}

/// Split a top-level `extends` condition into its checked and constraint types.
fn split_top_level_extends(type_expr: &str) -> Option<(&str, &str)> {
    let constraint = heritage_extends_clause(type_expr)?;
    let right_offset = constraint.as_ptr() as usize - type_expr.as_ptr() as usize;
    let left_end = right_offset.checked_sub("extends".len())?;
    Some((type_expr.get(..left_end)?.trim(), constraint.trim()))
}

/// Resolve a TS built-in utility type (`Partial`/`Required`/`Readonly`/`Pick`/
/// `Omit`/`Record`) to an object-literal member body, or `None` when `type_expr`
/// is not one of those (or its argument can't be resolved structurally).
fn resolve_utility_type(
    type_expr: &str,
    interfaces: &FxHashMap<String, String>,
    type_aliases: &FxHashMap<String, String>,
    stack: &mut FxHashSet<String>,
    missing: &mut dyn FnMut(&str),
    collect_missing: bool,
) -> Option<String> {
    let (name, args) = split_generic_call(type_expr)?;
    let type_args = split_top_level(args, ',');

    match name {
        "Partial" | "Required" | "Readonly" => {
            let inner = type_args.first()?;
            let body = resolve_type_to_object_body_inner(
                inner,
                interfaces,
                type_aliases,
                stack,
                missing,
                collect_missing,
            )?;
            let mut members = parse_members(&body);
            let (optional, readonly) = match name {
                "Partial" => (Some(true), false),
                "Required" => (Some(false), false),
                _ => (None, true),
            };
            for member in &mut members {
                member.optional = optional.unwrap_or(member.optional);
                member.readonly |= readonly;
            }
            Some(render_members(&members))
        }
        "Pick" | "Omit" => {
            let inner = type_args.first()?;
            let keys_arg = type_args.get(1)?;
            let body = resolve_type_to_object_body_inner(
                inner,
                interfaces,
                type_aliases,
                stack,
                missing,
                collect_missing,
            )?;
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
            let keys = match resolve_record_keys(keys_arg, interfaces, type_aliases, stack) {
                Ok(keys) => keys,
                Err(reference) => {
                    if collect_missing {
                        missing(&reference);
                    }
                    return None;
                }
            };
            if keys.is_empty() {
                if collect_missing {
                    missing(strip_generic_params(keys_arg));
                }
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

fn resolve_record_keys(
    expression: &str,
    interfaces: &FxHashMap<String, String>,
    type_aliases: &FxHashMap<String, String>,
    stack: &mut FxHashSet<String>,
) -> Result<Vec<String>, String> {
    let keys = parse_string_literal_union(expression);
    if !keys.is_empty() {
        return Ok(keys);
    }

    let name = strip_generic_params(expression);
    if !stack.insert(name.to_compact_string()) {
        return Err(name.to_compact_string());
    }
    let resolved = match resolve_single_type_ref(name, interfaces, type_aliases) {
        Some(body) => resolve_record_keys(&body, interfaces, type_aliases, stack),
        None => Err(name.to_compact_string()),
    };
    stack.remove(name);
    resolved
}

fn is_supported_utility_type(name: &str) -> bool {
    matches!(
        name,
        "Partial" | "Required" | "Readonly" | "Pick" | "Omit" | "Record"
    )
}

/// Split a generic type reference `Name<args>` into `("Name", "args")`.
fn split_generic_call(type_expr: &str) -> Option<(&str, &str)> {
    let (name, rest) = type_expr.split_once('<')?;
    if !type_expr.trim_end().ends_with('>') {
        return None;
    }
    let (args, _) = rest.rsplit_once('>')?;
    Some((name.trim(), args.trim()))
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
    name.split_once('<').map_or(name, |(base, _)| base).trim()
}

/// Return whether `name` is a dotted TypeScript identifier path.
fn is_type_reference_name(name: &str) -> bool {
    name.split('.').all(|part| {
        let mut characters = part.chars();
        characters.next().is_some_and(|character| {
            character == '_' || character == '$' || character.is_alphabetic()
        }) && characters
            .all(|character| character == '_' || character == '$' || character.is_alphanumeric())
    })
}

fn split_top_level(input: &str, delimiter: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::default();
    let mut depth = 0i32;
    let mut prev = '\0';

    for ch in input.chars() {
        match ch {
            '{' | '<' | '(' | '[' => depth += 1,
            '}' | ')' | ']' if depth > 0 => depth -= 1,
            '>' if prev != '=' && depth > 0 => depth -= 1,
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

#[cfg(test)]
mod tests;
