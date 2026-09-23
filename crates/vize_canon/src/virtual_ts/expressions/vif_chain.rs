//! Recognition and emission of v-if / v-else-if / v-else control-flow chains.

use vize_croquis::croquis::{TemplateExpression, TemplateExpressionKind};
use vize_croquis::drawer::strip_js_comments;

mod emit;
pub(super) use emit::emit_vif_control_flow_chain;

#[derive(Clone, Copy)]
struct GuardTerm<'a> {
    negated: bool,
    condition: &'a str,
    raw: &'a str,
}

struct VifBranch<'a> {
    condition: Option<&'a str>,
    guard: &'a str,
    start: usize,
    end: usize,
    condition_expr_index: Option<usize>,
}

pub(super) struct VifControlFlowChain<'a> {
    prefix: Vec<GuardTerm<'a>>,
    branches: Vec<VifBranch<'a>>,
    pub(super) end: usize,
}

impl<'a> VifControlFlowChain<'a> {
    pub(super) fn collect(exprs: &[&'a TemplateExpression], start: usize) -> Option<Self> {
        let first = collect_guard_group(exprs, start)?;
        let first_terms = parse_guard_terms(first.guard)?;
        let (&first_condition, prefix) = first_terms.split_last()?;
        if first_condition.negated {
            return None;
        }

        let mut previous_conditions = vec![first_condition.condition];
        let mut branches = vec![VifBranch {
            condition: Some(first_condition.condition),
            guard: first.guard,
            start: first.start,
            end: first.end,
            condition_expr_index: find_branch_condition_expr(
                exprs,
                first.start,
                first.end,
                first_condition.condition,
            ),
        }];
        let mut cursor = first.end;

        while cursor < exprs.len() {
            let Some(group) = collect_guard_group(exprs, cursor) else {
                break;
            };
            let Some(terms) = parse_guard_terms(group.guard) else {
                break;
            };
            if !prefix_matches(prefix, &terms) {
                break;
            }

            let chain_terms = terms.get(prefix.len()..).unwrap_or_default();
            if previous_negations_match(chain_terms, &previous_conditions)
                && chain_terms.len() == previous_conditions.len() + 1
                && let Some(current) = chain_terms.last()
                && !current.negated
            {
                previous_conditions.push(current.condition);
                branches.push(VifBranch {
                    condition: Some(current.condition),
                    guard: group.guard,
                    start: group.start,
                    end: group.end,
                    condition_expr_index: find_branch_condition_expr(
                        exprs,
                        group.start,
                        group.end,
                        current.condition,
                    ),
                });
                cursor = group.end;
                continue;
            }

            if previous_negations_match(chain_terms, &previous_conditions)
                && chain_terms.len() == previous_conditions.len()
            {
                branches.push(VifBranch {
                    condition: None,
                    guard: group.guard,
                    start: group.start,
                    end: group.end,
                    condition_expr_index: None,
                });
                cursor = group.end;
            }
            break;
        }

        if branches.len() < 2 {
            return None;
        }

        Some(Self {
            prefix: prefix.to_vec(),
            branches,
            end: cursor,
        })
    }
}

struct GuardGroup<'a> {
    guard: &'a str,
    start: usize,
    end: usize,
}

fn collect_guard_group<'a>(
    exprs: &[&'a TemplateExpression],
    start: usize,
) -> Option<GuardGroup<'a>> {
    let guard = exprs.get(start)?.vif_guard.as_ref()?.as_str();
    let mut end = start + 1;
    while exprs
        .get(end)
        .is_some_and(|expr| expr.vif_guard.as_ref().is_some_and(|g| g == guard))
    {
        end += 1;
    }
    Some(GuardGroup { guard, start, end })
}

fn parse_guard_terms(guard: &str) -> Option<Vec<GuardTerm<'_>>> {
    let mut terms = Vec::new();
    for term in split_top_level_and(guard) {
        let raw = term.trim();
        if raw.is_empty() {
            return None;
        }
        if let Some(condition) = strip_negated_wrapped_condition(raw) {
            terms.push(GuardTerm {
                negated: true,
                condition,
                raw,
            });
        } else {
            let condition = strip_wrapped_condition(raw)?;
            terms.push(GuardTerm {
                negated: false,
                condition,
                raw,
            });
        }
    }
    (!terms.is_empty()).then_some(terms)
}

fn split_top_level_and(input: &str) -> Vec<&str> {
    let bytes = input.as_bytes();
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    let mut index = 0usize;

    while let Some(&byte) = bytes.get(index) {
        match byte {
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            b'&' if depth == 0
                && bytes.get(index + 1) == Some(&b'&')
                && is_ascii_space(bytes.get(index.wrapping_sub(1)).copied())
                && is_ascii_space(bytes.get(index + 2).copied()) =>
            {
                parts.push(input.get(start..index - 1).unwrap_or_default());
                index += 3;
                start = index;
                continue;
            }
            _ => {}
        }
        index += 1;
    }

    parts.push(input.get(start..).unwrap_or_default());
    parts
}

fn is_ascii_space(byte: Option<u8>) -> bool {
    byte.is_some_and(|b| b.is_ascii_whitespace())
}

fn strip_negated_wrapped_condition(input: &str) -> Option<&str> {
    input
        .strip_prefix("!(")
        .and_then(|rest| rest.strip_suffix(')'))
}

fn strip_wrapped_condition(input: &str) -> Option<&str> {
    input
        .strip_prefix('(')
        .and_then(|rest| rest.strip_suffix(')'))
}

fn prefix_matches(prefix: &[GuardTerm<'_>], terms: &[GuardTerm<'_>]) -> bool {
    if terms.len() < prefix.len() {
        return false;
    }
    prefix
        .iter()
        .zip(terms.iter())
        .all(|(a, b)| a.negated == b.negated && a.condition == b.condition)
}

fn previous_negations_match(terms: &[GuardTerm<'_>], previous_conditions: &[&str]) -> bool {
    if terms.len() < previous_conditions.len() {
        return false;
    }
    terms
        .iter()
        .take(previous_conditions.len())
        .zip(previous_conditions.iter())
        .all(|(term, condition)| term.negated && term.condition == *condition)
}

fn find_branch_condition_expr(
    exprs: &[&TemplateExpression],
    start: usize,
    end: usize,
    condition: &str,
) -> Option<usize> {
    let trimmed_condition = condition.trim();
    (start..end).find(|&idx| {
        exprs.get(idx).is_some_and(|expr| {
            expr.kind == TemplateExpressionKind::VIf
                && strip_js_comments(expr.content.as_str()).as_ref().trim() == trimmed_condition
        })
    })
}
