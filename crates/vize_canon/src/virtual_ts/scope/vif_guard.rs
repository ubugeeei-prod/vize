use vize_carton::String;
use vize_croquis::{
    Scope, ScopeData, TemplateExpression, VForScopeData, drawer::extract_identifiers_oxc,
};

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
