//! Guard-term provenance for dynamic-component aliases.
//!
//! Joined branch guards repeat authored text across template scopes. This
//! module resolves each term back to the condition that evaluated it and asks
//! whether a descendant scope rebinds any referenced name before the alias.

use vize_carton::String;
use vize_croquis::{Croquis, ScopeId, TemplateExpression, TemplateExpressionKind};

pub(super) struct TermOrigin<'a> {
    pub(super) term: &'a str,
    pub(super) scope_id: ScopeId,
    pub(super) start: u32,
    pub(super) negated: bool,
}

/// Resolve each joined guard term to the `v-if` condition that established it.
/// A negative term belongs to the preceding branch condition, which also gives
/// `v-else` a concrete evaluation scope even though it has no expression node.
pub(super) fn term_origins<'a>(
    summary: &Croquis,
    usage: &vize_croquis::croquis::ComponentUsage,
    guard: &'a str,
) -> Option<Vec<TermOrigin<'a>>> {
    let terms = super::super::vif_guard::split_guard_terms(guard);
    let mut origins = Vec::with_capacity(terms.len());
    for (index, term) in terms.iter().copied().enumerate() {
        let prefix = join_terms(terms.get(..=index)?);
        let (expression, negated) = match guard_expression(summary, usage, prefix.as_str()) {
            Some(expression) => (expression, false),
            None => {
                let condition = term.strip_prefix("!(")?.strip_suffix(')')?;
                let positive = vize_carton::cstr!("({condition})");
                let mut candidate_terms = terms.get(..index)?.to_vec();
                candidate_terms.push(positive.as_str());
                let candidate = join_terms(candidate_terms.as_slice());
                (guard_expression(summary, usage, candidate.as_str())?, true)
            }
        };
        origins.push(TermOrigin {
            term,
            scope_id: expression.scope_id,
            start: expression.start,
            negated,
        });
    }
    Some(origins)
}

/// Capture only when a descendant scope declares a name read by this term.
/// Parser refusal stays conservative: crossing an unknown scope then captures.
pub(super) fn term_needs_capture(
    summary: &Croquis,
    origin: &TermOrigin<'_>,
    usage_scope: ScopeId,
) -> bool {
    if origin.scope_id == usage_scope {
        return false;
    }
    let Some(references) = vize_croquis::drawer::extract_identifier_refs_checked(origin.term)
    else {
        return true;
    };
    if references.is_empty() {
        return false;
    }

    let mut current = usage_scope;
    loop {
        if current == origin.scope_id {
            return false;
        }
        let Some(scope) = summary.scopes.get_scope(current) else {
            return true;
        };
        if references
            .iter()
            .any(|reference| scope.has_binding(reference.name.as_str()))
        {
            return true;
        }
        let Some(parent) = scope.parent() else {
            return true;
        };
        current = parent;
    }
}

pub(super) fn join_strings(terms: &[String]) -> String {
    let mut joined = String::default();
    for (index, term) in terms.iter().enumerate() {
        if index != 0 {
            joined.push_str(" && ");
        }
        joined.push_str(term.as_str());
    }
    joined
}

/// Find the latest authored condition that establishes an exact guard prefix.
fn guard_expression<'a>(
    summary: &'a Croquis,
    usage: &vize_croquis::croquis::ComponentUsage,
    guard: &str,
) -> Option<&'a TemplateExpression> {
    summary
        .template_expressions
        .iter()
        .filter(|expression| expression.kind == TemplateExpressionKind::VIf)
        .filter(|expression| expression.start < usage.end)
        .filter(|expression| expression.vif_guard.as_deref() == Some(guard))
        .filter(|expression| {
            is_lexical_ancestor(&summary.scopes, expression.scope_id, usage.scope_id)
        })
        .max_by_key(|expression| expression.start)
}

fn is_lexical_ancestor(
    scopes: &vize_croquis::ScopeChain,
    ancestor: ScopeId,
    mut descendant: ScopeId,
) -> bool {
    loop {
        if ancestor == descendant {
            return true;
        }
        let Some(parent) = scopes
            .get_scope(descendant)
            .and_then(|scope| scope.parent())
        else {
            return false;
        };
        descendant = parent;
    }
}

fn join_terms(terms: &[&str]) -> String {
    let mut joined = String::default();
    for (index, term) in terms.iter().enumerate() {
        if index != 0 {
            joined.push_str(" && ");
        }
        joined.push_str(term);
    }
    joined
}
