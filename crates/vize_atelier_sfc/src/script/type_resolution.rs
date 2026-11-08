use vize_carton::{FxHashMap, FxHashSet, String, ToCompactString};

mod conditional;
mod members;
#[cfg(test)]
use members::strip_comments;
use members::{Member, parse_members, render_members};
mod union;
mod utility;

use conditional::{conditional_checked_type, is_conditional_type, resolve_conditional_type};
use utility::{is_supported_utility_type, resolve_utility_type};

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
        if collect_missing
            && type_args
                .first()
                .is_some_and(|checked| is_type_reference_name(strip_generic_params(checked)))
        {
            missing(trimmed);
        }
        return None;
    }

    let base_name = strip_generic_params(trimmed);
    if !is_type_reference_name(base_name) {
        if collect_missing {
            missing(trimmed);
        }
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

/// Split a generic type reference `Name<args>` into `("Name", "args")`.
fn split_generic_call(type_expr: &str) -> Option<(&str, &str)> {
    let (name, rest) = type_expr.split_once('<')?;
    if !type_expr.trim_end().ends_with('>') {
        return None;
    }
    let (args, _) = rest.rsplit_once('>')?;
    Some((name.trim(), args.trim()))
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
