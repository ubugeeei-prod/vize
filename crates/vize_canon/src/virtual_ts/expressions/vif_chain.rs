//! Recognition and emission of v-if / v-else-if / v-else control-flow chains.

use super::super::types::VizeMapping;
use super::statements::generate_expression_statement;
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::append;
use vize_carton::cstr;
use vize_croquis::analysis::{TemplateExpression, TemplateExpressionKind};
use vize_croquis::analyzer::strip_js_comments;

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

            let chain_terms = &terms[prefix.len()..];
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
    while end < exprs.len() && exprs[end].vif_guard.as_ref().is_some_and(|g| g == guard) {
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
        } else if let Some(condition) = strip_wrapped_condition(raw) {
            terms.push(GuardTerm {
                negated: false,
                condition,
                raw,
            });
        } else {
            return None;
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

    while index < bytes.len() {
        match bytes[index] {
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            b'&' if depth == 0
                && bytes.get(index + 1) == Some(&b'&')
                && is_ascii_space(bytes.get(index.wrapping_sub(1)).copied())
                && is_ascii_space(bytes.get(index + 2).copied()) =>
            {
                parts.push(&input[start..index - 1]);
                index += 3;
                start = index;
                continue;
            }
            _ => {}
        }
        index += 1;
    }

    parts.push(&input[start..]);
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
        exprs[idx].kind == TemplateExpressionKind::VIf
            && strip_js_comments(exprs[idx].content.as_str())
                .as_ref()
                .trim()
                == trimmed_condition
    })
}

pub(super) fn emit_vif_control_flow_chain(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    exprs: &[&TemplateExpression],
    chain: &VifControlFlowChain<'_>,
    template_prop_names: &FxHashSet<String>,
    context: &VifControlFlowEmitContext<'_>,
) {
    for (branch_index, branch) in chain.branches.iter().enumerate() {
        emit_vif_branch_open(
            ts,
            mappings,
            exprs,
            chain,
            branch,
            branch_index == 0,
            context,
        );

        let body_indent = cstr!("{}  ", context.indent);
        for (expr_index, expr) in exprs.iter().enumerate().take(branch.end).skip(branch.start) {
            if branch.condition_expr_index == Some(expr_index) {
                continue;
            }
            if context
                .skipped_expression_ranges
                .contains(&(expr.start, expr.end))
            {
                continue;
            }
            generate_expression_statement(
                ts,
                mappings,
                expr,
                template_prop_names,
                context.template_offset,
                &body_indent,
            );
        }
    }
    append!(*ts, "{}}}\n", context.indent);
}

fn emit_vif_branch_open(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    exprs: &[&TemplateExpression],
    chain: &VifControlFlowChain<'_>,
    branch: &VifBranch<'_>,
    first: bool,
    context: &VifControlFlowEmitContext<'_>,
) {
    let prefix_is_empty = chain.prefix.is_empty();
    match (first, branch.condition) {
        (true, Some(condition)) => {
            append!(*ts, "{}if (", context.indent);
            append_guard_condition(
                ts,
                &chain.prefix,
                Some(condition),
                mappings,
                exprs,
                branch,
                context.template_offset,
            );
            ts.push_str(") {\n");
        }
        (false, Some(condition)) => {
            append!(*ts, "{}}} else if (", context.indent);
            append_guard_condition(
                ts,
                &chain.prefix,
                Some(condition),
                mappings,
                exprs,
                branch,
                context.template_offset,
            );
            ts.push_str(") {\n");
        }
        (false, None) if prefix_is_empty => {
            append!(*ts, "{}}} else {{\n", context.indent);
        }
        (false, None) => {
            append!(*ts, "{}}} else if (", context.indent);
            ts.push_str(branch.guard);
            ts.push_str(") {\n");
        }
        (true, None) => {}
    }

    if let Some(expr_index) = branch.condition_expr_index {
        let expr = exprs[expr_index];
        let src_start = (context.template_offset + expr.start) as usize;
        let src_end = (context.template_offset + expr.end) as usize;
        append!(
            *ts,
            "{}  // @vize-map: expr -> {src_start}:{src_end}\n",
            context.indent,
        );
    }
}

pub(super) struct VifControlFlowEmitContext<'a> {
    pub(super) skipped_expression_ranges: &'a FxHashSet<(u32, u32)>,
    pub(super) template_offset: u32,
    pub(super) indent: &'a str,
}

fn append_guard_condition(
    ts: &mut String,
    prefix: &[GuardTerm<'_>],
    condition: Option<&str>,
    mappings: &mut Vec<VizeMapping>,
    exprs: &[&TemplateExpression],
    branch: &VifBranch<'_>,
    template_offset: u32,
) {
    append_guard_prefix(ts, prefix);
    if let Some(condition) = condition {
        if !prefix.is_empty() {
            ts.push_str(" && (");
        }
        let gen_start = ts.len();
        ts.push_str(condition);
        let gen_end = ts.len();
        if let Some(expr_index) = branch.condition_expr_index {
            let expr = exprs[expr_index];
            mappings.push(VizeMapping {
                gen_range: gen_start..gen_end,
                src_range: (template_offset + expr.start) as usize
                    ..(template_offset + expr.end) as usize,
                sub_spans: Vec::new(),
            });
        }
        if !prefix.is_empty() {
            ts.push(')');
        }
    }
}

fn append_guard_prefix(ts: &mut String, prefix: &[GuardTerm<'_>]) {
    for (index, term) in prefix.iter().enumerate() {
        if index > 0 {
            ts.push_str(" && ");
        }
        ts.push_str(term.raw);
    }
}
