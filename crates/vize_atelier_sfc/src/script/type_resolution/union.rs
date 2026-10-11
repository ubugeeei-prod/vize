//! Preserve branch members before the existing runtime-prop AST inference.

use super::{
    FxHashMap, FxHashSet, Member, String, parse_members, render_members,
    resolve_type_to_object_body_inner,
};
use vize_carton::cstr;

pub(super) fn resolve_members(
    parts: &[String],
    interfaces: &FxHashMap<String, String>,
    type_aliases: &FxHashMap<String, String>,
    stack: &mut FxHashSet<String>,
) -> Option<String> {
    let mut members: Vec<Member> = Vec::new();
    for part in parts {
        // Keep the original expression when any branch cannot be expanded.
        // A partial literal would erase the unresolved branch before the
        // existing AST resolver has a chance to inspect it.
        let body = resolve_type_to_object_body_inner(part, interfaces, type_aliases, stack)?;
        for member in parse_members(&body) {
            if let Some(previous) = members.iter_mut().find(|old| old.key == member.key) {
                previous.optional |= member.optional;
                previous.readonly |= member.readonly;
                if previous.ty != member.ty {
                    previous.ty = cstr!("({}) | ({})", previous.ty, member.ty);
                }
            } else {
                members.push(member);
            }
        }
    }
    (!members.is_empty()).then(|| render_members(&members))
}

#[cfg(test)]
mod tests {
    use super::super::resolve_type_args;
    use super::*;

    #[test]
    fn nested_union_intersection_retains_branch_keys_and_common_constructor_types() {
        let aliases = FxHashMap::from_iter([
            (cstr!("Base"), String::from("{ id?: string }")),
            (
                cstr!("Props"),
                String::from(
                    "Base & ({ decorative?: true; title?: never } | { decorative?: false; title: string })",
                ),
            ),
        ]);
        assert_eq!(
            resolve_type_args("Props", &FxHashMap::default(), &aliases),
            "{ id?: string; decorative?: (true) | (false); title?: (never) | (string) }"
        );
    }

    #[test]
    fn direct_union_preserves_branch_local_optionality_and_object_member_unions() {
        assert_eq!(
            resolve_type_args(
                "{ title: string; value: string; choice: 'a' | 'b' } | { label?: string; value?: number }",
                &FxHashMap::default(),
                &FxHashMap::default()
            ),
            "{ title: string; value?: (string) | (number); choice: 'a' | 'b'; label?: string }"
        );
    }

    #[test]
    fn unknown_and_recursive_branches_do_not_become_partial_object_literals() {
        let aliases = FxHashMap::from_iter([
            (cstr!("Cycle"), String::from("Cycle | { known: string }")),
            (
                cstr!("Nested"),
                String::from("{ known: string } | (Missing | { enabled?: boolean })"),
            ),
        ]);
        for expression in [
            "Missing | { known: string }",
            "{ known: string } | Missing",
            "Cycle",
            "Nested",
        ] {
            assert_eq!(
                resolve_type_args(expression, &FxHashMap::default(), &aliases),
                expression,
            );
        }
    }
}
