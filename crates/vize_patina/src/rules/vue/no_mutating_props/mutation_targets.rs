//! Text-level matching of a mutation target against the component's props.
//!
//! Callers pass the *source slice of an assignment target* (`props.msg`,
//! `user.name`, `count`), never a whole expression, so matching a prefix here
//! cannot pick up an unrelated occurrence elsewhere in the expression.

use vize_s0::FxHashSet;

pub(super) fn is_prop_mutation_target(
    content: &str,
    prop_names: &FxHashSet<&str>,
    has_props_object_binding: bool,
) -> bool {
    let content = content.trim();
    if prop_names.contains(content) {
        return true;
    }

    if has_props_object_binding
        && content
            .strip_prefix("props")
            .is_some_and(|rest| is_props_object_member_mutation(rest, prop_names))
    {
        return true;
    }

    prop_names.iter().any(|name| {
        content
            .strip_prefix(*name)
            .is_some_and(is_member_access_suffix)
    })
}

fn is_member_access_suffix(rest: &str) -> bool {
    rest.starts_with('.') || rest.starts_with('[') || rest.starts_with("?.")
}

fn is_props_object_member_mutation(rest: &str, prop_names: &FxHashSet<&str>) -> bool {
    if let Some(name) = props_member_root(rest) {
        return prop_names.is_empty() || prop_names.contains(name);
    }

    is_dynamic_props_member_access(rest)
}

fn is_dynamic_props_member_access(rest: &str) -> bool {
    let mut rest = rest.trim_start();
    if let Some(after_optional) = rest.strip_prefix("?.") {
        rest = after_optional.trim_start();
    }

    let Some(after_bracket) = rest.strip_prefix('[') else {
        return false;
    };
    let after_bracket = after_bracket.trim_start();
    !after_bracket.starts_with('\'') && !after_bracket.starts_with('"')
}

fn props_member_root(rest: &str) -> Option<&str> {
    let mut rest = rest.trim_start();
    let mut consumed_optional = false;
    if let Some(after_optional) = rest.strip_prefix("?.") {
        rest = after_optional.trim_start();
        consumed_optional = true;
    }

    if let Some(after_dot) = rest.strip_prefix('.') {
        return identifier_root(after_dot);
    }

    if consumed_optional && let Some(name) = identifier_root(rest) {
        return Some(name);
    }

    let after_bracket = rest.strip_prefix('[')?.trim_start();
    let quote = after_bracket.chars().next()?;
    if quote != '\'' && quote != '"' {
        return None;
    }
    let name_start = quote.len_utf8();
    let name_end = after_bracket[name_start..].find(quote)? + name_start;
    (name_end > name_start).then_some(&after_bracket[name_start..name_end])
}

fn identifier_root(source: &str) -> Option<&str> {
    let end = source
        .find(|ch: char| !(ch == '_' || ch == '$' || ch.is_ascii_alphanumeric()))
        .unwrap_or(source.len());
    (end > 0).then_some(&source[..end])
}

#[cfg(test)]
mod tests {
    use super::is_prop_mutation_target;
    use vize_s0::FxHashSet;

    #[test]
    fn prop_mutation_target_matches_member_roots() {
        let prop_names = FxHashSet::from_iter(["count", "user"]);

        assert!(is_prop_mutation_target("count", &prop_names, false));
        assert!(is_prop_mutation_target("user.name", &prop_names, false));
        assert!(is_prop_mutation_target("user?.name", &prop_names, false));
        assert!(is_prop_mutation_target("props.count", &prop_names, true));
        assert!(is_prop_mutation_target(
            "props.user.name",
            &prop_names,
            true
        ));
        assert!(is_prop_mutation_target("props['count']", &prop_names, true));
        assert!(is_prop_mutation_target("props[key]", &prop_names, true));
        assert!(is_prop_mutation_target(
            "props[key].name",
            &prop_names,
            true
        ));
        assert!(is_prop_mutation_target(
            "props?.user.name",
            &prop_names,
            true
        ));
        assert!(!is_prop_mutation_target("props.extra", &prop_names, true));
        assert!(!is_prop_mutation_target(
            "props['extra']",
            &prop_names,
            true
        ));
        assert!(!is_prop_mutation_target(
            "props.user.name",
            &prop_names,
            false
        ));
        assert!(!is_prop_mutation_target(
            "counter.value",
            &prop_names,
            false
        ));
        assert!(!is_prop_mutation_target(
            "propsState.count",
            &prop_names,
            true
        ));

        let unknown_prop_names = FxHashSet::default();
        assert!(is_prop_mutation_target(
            "props.title",
            &unknown_prop_names,
            true
        ));
        assert!(is_prop_mutation_target(
            "props[field]",
            &unknown_prop_names,
            true
        ));
    }
}
