use vize_carton::{FxHashMap, FxHashSet, String, ToCompactString};

use super::members::{Member, parse_members, render_members};
use super::{
    resolve_single_type_ref, resolve_type_to_object_body_inner, split_generic_call,
    split_top_level, strip_generic_params,
};

/// Resolve a TS built-in utility type (`Partial`/`Required`/`Readonly`/`Pick`/
/// `Omit`/`Record`) to an object-literal member body, or `None` when `type_expr`
/// is not one of those (or its argument can't be resolved structurally).
pub(super) fn resolve_utility_type(
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
                .filter(|member| {
                    let contains = keys.iter().any(|key| key == member.key.as_str());
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

pub(super) fn is_supported_utility_type(name: &str) -> bool {
    matches!(
        name,
        "Partial" | "Required" | "Readonly" | "Pick" | "Omit" | "Record"
    )
}

/// Parse a string-literal key argument (`'a'` or `'a' | 'b'`) into the set of
/// bare key names. Returns empty when the argument isn't a string-literal union.
fn parse_string_literal_union(arg: &str) -> Vec<String> {
    let mut keys = Vec::new();
    for part in split_top_level(arg, '|') {
        let trimmed = part.trim();
        let unquoted = trimmed
            .strip_prefix('\'')
            .and_then(|value| value.strip_suffix('\''))
            .or_else(|| {
                trimmed
                    .strip_prefix('"')
                    .and_then(|value| value.strip_suffix('"'))
            });
        match unquoted {
            Some(key) => keys.push(key.to_compact_string()),
            None => return Vec::new(),
        }
    }
    keys
}
