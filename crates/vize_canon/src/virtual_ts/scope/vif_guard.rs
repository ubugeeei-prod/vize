use vize_carton::{String, append};
use vize_croquis::{
    Scope, ScopeData, TemplateExpression, VForScopeData, drawer::extract_identifiers_oxc,
};

pub(crate) fn append_ignored_vif_guard_open(
    ts: &mut String,
    indent: &str,
    guard: &str,
    purpose: &str,
) {
    append!(
        *ts,
        "{indent}// @ts-ignore {purpose}; authored v-if checks own diagnostics.\n{indent}if ({guard}) {{\n"
    );
}

/// Compute the enclosing v-if guard shared by all expressions in a v-for scope.
pub(super) fn common_vif_guard_prefix(exprs: &[&TemplateExpression]) -> Option<String> {
    let mut iter = exprs.iter();
    let first = iter.next()?.vif_guard.as_ref()?;
    let mut rest = Vec::new();
    for expr in iter {
        rest.push(expr.vif_guard.as_ref()?.as_str());
    }
    common_guard_prefix_from_terms(first.as_str(), rest.into_iter())
}

pub(super) fn common_vif_guard_prefix_for_guards(guards: &[&str]) -> Option<String> {
    let (first, rest) = guards.split_first()?;
    common_guard_prefix_from_terms(first, rest.iter().copied())
}

/// Return the positive guard terms that must be repeated inside a generated
/// callback to restore narrowing for captured object properties.
///
/// Negated branch terms are intentionally omitted. Rechecking an exclusion
/// after the surrounding control flow has already narrowed the discriminant
/// can itself produce an impossible-comparison diagnostic.
pub(super) fn callback_vif_guard(guard: &str) -> Option<String> {
    let positive_terms: Vec<_> = split_guard_terms(guard)
        .into_iter()
        .filter(|term| !term.starts_with('!'))
        .collect();

    (!positive_terms.is_empty()).then(|| String::from(positive_terms.join(" && ").as_str()))
}

/// Remove a guard prefix that is already active in the surrounding generated
/// control flow.
///
/// Guards are compared term-by-term so conditions containing nested `&&`
/// expressions are not truncated accidentally. If `prefix` is not an exact
/// top-level prefix, the original guard is preserved.
pub(crate) fn remove_enclosing_vif_guard_prefix(guard: &str, prefix: &str) -> Option<String> {
    let guard_terms = split_guard_terms(guard);
    let prefix_terms = split_guard_terms(prefix);
    if prefix_terms.len() > guard_terms.len()
        || !prefix_terms
            .iter()
            .zip(guard_terms.iter())
            .all(|(prefix_term, guard_term)| prefix_term == guard_term)
    {
        return Some(String::from(guard));
    }

    let remaining = &guard_terms[prefix_terms.len()..];
    (!remaining.is_empty()).then(|| String::from(remaining.join(" && ").as_str()))
}

fn common_guard_prefix_from_terms<'a>(
    first: &'a str,
    rest: impl Iterator<Item = &'a str>,
) -> Option<String> {
    let mut common: Vec<&str> = split_guard_terms(first);

    for guard in rest {
        let terms = split_guard_terms(guard);
        let shared = common
            .iter()
            .zip(terms.iter())
            .take_while(|(a, b)| a == b)
            .count();
        common.truncate(shared);
        if common.is_empty() {
            return None;
        }
    }

    (!common.is_empty()).then(|| String::from(common.join(" && ").as_str()))
}

pub(super) fn common_vif_guard_prefix_outside_v_for(
    exprs: &[&TemplateExpression],
    data: &VForScopeData,
) -> Option<String> {
    let guard = common_vif_guard_prefix(exprs)?;
    trim_v_for_alias_guard_prefix(guard, data)
}

pub(super) fn common_vif_guard_prefix_outside_v_for_scope(
    exprs: &[&TemplateExpression],
    scope: &Scope,
) -> Option<String> {
    let ScopeData::VFor(data) = scope.data() else {
        return None;
    };
    common_vif_guard_prefix_outside_v_for(exprs, data)
}

pub(super) fn common_vif_guard_prefix_for_guards_outside_v_for(
    guards: &[&str],
    data: &VForScopeData,
) -> Option<String> {
    let guard = common_vif_guard_prefix_for_guards(guards)?;
    trim_v_for_alias_guard_prefix(guard, data)
}

fn trim_v_for_alias_guard_prefix(guard: String, data: &VForScopeData) -> Option<String> {
    let own_aliases = v_for_aliases(data);
    if own_aliases.is_empty() {
        return Some(guard);
    }

    let terms = split_guard_terms(guard.as_str());
    let outside_terms: Vec<&str> = terms
        .into_iter()
        .take_while(|term| !references_any_alias(term, own_aliases.as_slice()))
        .collect();

    (!outside_terms.is_empty()).then(|| String::from(outside_terms.join(" && ").as_str()))
}

fn split_guard_terms(guard: &str) -> Vec<&str> {
    let bytes = guard.as_bytes();
    let mut terms = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    let mut index = 0usize;

    while index < bytes.len() {
        match bytes[index] {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            b'&' if depth == 0
                && bytes.get(index + 1) == Some(&b'&')
                && index >= 1
                && bytes[index - 1] == b' '
                && bytes.get(index + 2) == Some(&b' ') =>
            {
                terms.push(guard[start..index - 1].trim());
                index += 3;
                start = index;
                continue;
            }
            _ => {}
        }
        index += 1;
    }

    terms.push(guard[start..].trim());
    terms
}

fn v_for_aliases(data: &VForScopeData) -> Vec<&str> {
    let mut aliases: Vec<&str> = data
        .value_bindings
        .iter()
        .map(|alias| alias.as_str())
        .collect();
    if let Some(alias) = data.key_alias.as_ref() {
        aliases.push(alias.as_str());
    }
    if let Some(alias) = data.index_alias.as_ref() {
        aliases.push(alias.as_str());
    }
    aliases
}

fn references_any_alias(term: &str, aliases: &[&str]) -> bool {
    extract_identifiers_oxc(term)
        .iter()
        .any(|identifier| aliases.iter().any(|alias| identifier.as_str() == *alias))
}

#[cfg(test)]
mod tests {
    use super::{callback_vif_guard, remove_enclosing_vif_guard_prefix};

    #[test]
    fn callback_guard_keeps_positive_narrowing_terms() {
        assert_eq!(
            callback_vif_guard("!(item.type === 'text') && ready && (item.type === 'container')")
                .as_deref(),
            Some("ready && (item.type === 'container')")
        );
    }

    #[test]
    fn callback_guard_omits_negated_branch_terms() {
        assert_eq!(callback_vif_guard("!(item.type === 'text')"), None);
    }

    #[test]
    fn removes_an_exact_enclosing_guard() {
        assert_eq!(
            remove_enclosing_vif_guard_prefix(
                "!(item.type === 'hunk-bar')",
                "!(item.type === 'hunk-bar')"
            ),
            None
        );
    }

    #[test]
    fn removes_only_complete_top_level_guard_terms() {
        assert_eq!(
            remove_enclosing_vif_guard_prefix(
                "(ready && enabled) && !(item.type === 'hunk-bar')",
                "(ready && enabled)",
            )
            .as_deref(),
            Some("!(item.type === 'hunk-bar')")
        );
    }

    #[test]
    fn preserves_a_guard_when_the_prefix_does_not_match() {
        assert_eq!(
            remove_enclosing_vif_guard_prefix("(ready) && (visible)", "(enabled)").as_deref(),
            Some("(ready) && (visible)")
        );
    }
}
