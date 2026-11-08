use vize_carton::{FxHashMap, FxHashSet, String, ToCompactString};

use super::members::parse_members;
use super::{
    heritage_extends_clause, is_type_reference_name, resolve_type_to_object_body_inner,
    split_top_level,
};

/// Return whether `type_expr` has a top-level conditional-type branch.
pub(super) fn is_conditional_type(type_expr: &str) -> bool {
    split_top_level(type_expr, '?').len() == 2
}

/// Extract the checked type from a top-level `T extends U ? X : Y` expression.
pub(super) fn conditional_checked_type(type_expr: &str) -> Option<String> {
    let condition = split_top_level(type_expr, '?');
    let condition = condition.first()?;
    split_top_level_extends(condition).map(|(checked, _)| checked.to_compact_string())
}

/// Resolve the selected object branch of a supported conditional type.
pub(super) fn resolve_conditional_type(
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
