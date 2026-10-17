use oxc_allocator::Allocator;
use oxc_ast::ast::Expression;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
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
    let isolated = crate::virtual_ts::expressions::isolate_incomplete_expression(guard);
    let guard = isolated.as_str();
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

    let remaining = guard_terms.get(prefix_terms.len()..).unwrap_or_default();
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

pub(super) fn split_guard_terms(guard: &str) -> Vec<&str> {
    // Generated guards wrap each authored condition in parentheses. Preserve
    // those nodes: only the outer generated conjunction joins branch terms.
    // A byte-depth scan can split inside strings, comments, regexes or templates.
    let allocator = Allocator::default();
    let Ok(expression) = Parser::new(&allocator, guard, SourceType::ts()).parse_expression() else {
        return vec![guard];
    };
    // This parser API can accept a valid prefix without consuming trailing
    // tokens. Never discard them (or outside comments) when opening guards.
    let span = expression.span();
    if guard.get(span.start as usize..span.end as usize) != Some(guard.trim()) {
        return vec![guard];
    }
    let mut terms = Vec::new();
    collect_guard_terms(&expression, guard, &mut terms);
    terms
}

fn collect_guard_terms<'s>(expression: &Expression<'_>, source: &'s str, terms: &mut Vec<&'s str>) {
    if let Expression::LogicalExpression(logical) = expression
        && logical.operator.is_and()
    {
        collect_guard_terms(&logical.left, source, terms);
        collect_guard_terms(&logical.right, source, terms);
    } else {
        let span = expression.span();
        terms.push(
            source
                .get(span.start as usize..span.end as usize)
                .unwrap_or(source),
        );
    }
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
    use super::{callback_vif_guard, remove_enclosing_vif_guard_prefix, split_guard_terms};

    #[test]
    fn branch_boundaries_come_from_parsed_expressions() {
        for condition in [
            "(item.kind === ') && (')",
            r#"(item.kind === ") && (")"#,
            "(item.kind === `) && ${'('}`)",
            "(/\\) && \\(/.test(item.kind))",
            "(item.kind /* ) && ( */ === 'a')",
            "(item.kind // ) && (\n === 'a')",
            "(ready && enabled)",
            "((item as { kind: string }).kind === 'a')",
            "((item satisfies { kind: string }).kind === 'a')",
        ] {
            let mut guard = vize_carton::String::from("(parent) && ");
            guard.push_str(condition);
            assert_eq!(split_guard_terms(guard.as_str()), ["(parent)", condition]);
        }
        for guard in [
            "(parent) && (",
            "(parent) && (ready) trailing",
            "(parent) && (ready) // retained outside comment",
            "/* retained outside comment */ (parent) && (ready)",
        ] {
            assert_eq!(split_guard_terms(guard), [guard]);
        }
    }

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
