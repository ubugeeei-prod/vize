//! Expression and component prop check generation for virtual TypeScript.
//!
//! Handles generating TypeScript code for template expressions (with optional
//! v-if narrowing) and component prop value type assertions.

use super::{
    helpers::{
        generated_text_range, is_reserved_identifier, to_camel_case, to_safe_identifier_fragment,
    },
    types::VizeMapping,
};
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::append;
use vize_carton::cstr;
use vize_carton::profile;
use vize_croquis::analysis::{ComponentUsage, TemplateExpression, TemplateExpressionKind};
use vize_croquis::analyzer::strip_js_comments;

/// Generate template expressions, compacting recognized v-if chains into
/// TypeScript control-flow blocks.
pub(crate) fn generate_expressions(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    exprs: &[&TemplateExpression],
    template_prop_names: &FxHashSet<String>,
    template_offset: u32,
    indent: &str,
) {
    let mut index = 0;
    while index < exprs.len() {
        if let Some(chain) = VifControlFlowChain::collect(exprs, index) {
            emit_vif_control_flow_chain(
                ts,
                mappings,
                exprs,
                &chain,
                template_prop_names,
                template_offset,
                indent,
            );
            index = chain.end;
            continue;
        }

        profile!(
            "canon.virtual_ts.generate_expression",
            generate_expression(
                ts,
                mappings,
                exprs[index],
                template_prop_names,
                template_offset,
                indent,
            )
        );
        index += 1;
    }
}

/// Generate a template expression with optional v-if narrowing.
///
/// When the expression has a `vif_guard`, wraps it in an if block to enable TypeScript type narrowing.
/// For example, `{{ todo.description }}` inside `v-if="todo.description"` generates:
/// ```typescript
/// if (todo.description) {
///   const __expr_X = todo.description;
/// }
/// ```
pub(crate) fn generate_expression(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    expr: &vize_croquis::TemplateExpression,
    template_prop_names: &FxHashSet<String>,
    template_offset: u32,
    indent: &str,
) {
    if let Some(ref guard) = expr.vif_guard {
        let trimmed_guard = guard.as_str().trim();
        let rewritten_guard = rewrite_reserved_template_prop(trimmed_guard, template_prop_names);
        let generated_guard = rewritten_guard
            .as_ref()
            .map_or_else(|| guard.as_str(), |s| s.as_str());
        // Wrap in if block for type narrowing
        append!(*ts, "{indent}if ({generated_guard}) {{\n");
        generate_expression_statement(
            ts,
            mappings,
            expr,
            template_prop_names,
            template_offset,
            &cstr!("{indent}  "),
        );
        append!(*ts, "{indent}}}\n");
    } else {
        generate_expression_statement(
            ts,
            mappings,
            expr,
            template_prop_names,
            template_offset,
            indent,
        );
    }
}

fn generate_expression_statement(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    expr: &TemplateExpression,
    template_prop_names: &FxHashSet<String>,
    template_offset: u32,
    indent: &str,
) {
    let src_start = (template_offset + expr.start) as usize;
    let src_end = (template_offset + expr.end) as usize;
    let expression = profile!(
        "canon.virtual_ts.expression.strip_comments",
        strip_js_comments(expr.content.as_str())
    );
    let trimmed_expression = expression.as_ref().trim();
    let rewritten_expression =
        rewrite_reserved_template_prop(trimmed_expression, template_prop_names);
    let generated_expression = rewritten_expression
        .as_ref()
        .map_or_else(|| expression.as_ref(), |s| s.as_str());
    let mapping_needle = if rewritten_expression.is_some() {
        generated_expression
    } else {
        expression.as_ref()
    };

    let gen_stmt_start = ts.len();
    append!(
        *ts,
        "{indent}void ({}); // {}\n",
        generated_expression,
        expr.kind.as_str()
    );
    let gen_stmt_end = ts.len();
    mappings.push(VizeMapping {
        gen_range: generated_text_range(
            &ts[gen_stmt_start..gen_stmt_end],
            mapping_needle,
            gen_stmt_start,
        ),
        src_range: src_start..src_end,
        sub_spans: Vec::new(),
    });
    append!(*ts, "{indent}// @vize-map: expr -> {src_start}:{src_end}\n",);
}

fn rewrite_reserved_template_prop(
    expression: &str,
    template_prop_names: &FxHashSet<String>,
) -> Option<String> {
    if !is_reserved_identifier(expression) || !template_prop_names.contains(expression) {
        return None;
    }
    Some(cstr!("props[\"{expression}\"]"))
}

#[derive(Clone, Copy)]
struct GuardTerm<'a> {
    negated: bool,
    condition: &'a str,
    raw: &'a str,
}

struct VifBranch<'a> {
    condition: Option<&'a str>,
    start: usize,
    end: usize,
    condition_expr_index: Option<usize>,
}

struct VifControlFlowChain<'a> {
    prefix: Vec<GuardTerm<'a>>,
    branches: Vec<VifBranch<'a>>,
    end: usize,
}

impl<'a> VifControlFlowChain<'a> {
    fn collect(exprs: &[&'a TemplateExpression], start: usize) -> Option<Self> {
        let first = collect_guard_group(exprs, start)?;
        let first_terms = parse_guard_terms(first.guard)?;
        let (&first_condition, prefix) = first_terms.split_last()?;
        if first_condition.negated {
            return None;
        }

        let mut previous_conditions = vec![first_condition.condition];
        let mut branches = vec![VifBranch {
            condition: Some(first_condition.condition),
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

fn emit_vif_control_flow_chain(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    exprs: &[&TemplateExpression],
    chain: &VifControlFlowChain<'_>,
    template_prop_names: &FxHashSet<String>,
    template_offset: u32,
    indent: &str,
) {
    let context = VifBranchEmitContext {
        template_offset,
        indent,
    };
    for (branch_index, branch) in chain.branches.iter().enumerate() {
        emit_vif_branch_open(
            ts,
            mappings,
            exprs,
            chain,
            branch,
            branch_index == 0,
            &context,
        );

        let body_indent = cstr!("{indent}  ");
        for (expr_index, expr) in exprs.iter().enumerate().take(branch.end).skip(branch.start) {
            if branch.condition_expr_index == Some(expr_index) {
                continue;
            }
            generate_expression_statement(
                ts,
                mappings,
                expr,
                template_prop_names,
                template_offset,
                &body_indent,
            );
        }
    }
    append!(*ts, "{indent}}}\n");
}

fn emit_vif_branch_open(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    exprs: &[&TemplateExpression],
    chain: &VifControlFlowChain<'_>,
    branch: &VifBranch<'_>,
    first: bool,
    context: &VifBranchEmitContext<'_>,
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
            append_guard_condition(
                ts,
                &chain.prefix,
                None,
                mappings,
                exprs,
                branch,
                context.template_offset,
            );
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

struct VifBranchEmitContext<'a> {
    template_offset: u32,
    indent: &'a str,
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

/// Generate component prop value checks at the given indentation level.
pub(crate) fn generate_component_prop_checks(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    usage: &ComponentUsage,
    idx: usize,
    template_prop_names: &FxHashSet<String>,
    template_offset: u32,
    indent: &str,
) {
    let component_type_name = to_safe_identifier_fragment(usage.name.as_str());
    for prop in &usage.props {
        if prop.name.as_str() == "key" || prop.name.as_str() == "ref" {
            continue;
        }
        if let Some(ref value) = prop.value
            && prop.is_dynamic
        {
            let prop_src_start = (template_offset + prop.start) as usize;
            let prop_src_end = (template_offset + prop.end) as usize;
            let value = profile!(
                "canon.virtual_ts.prop_check.strip_comments",
                strip_js_comments(value.as_str())
            );
            let trimmed_value = value.as_ref().trim();
            let rewritten_value =
                rewrite_reserved_template_prop(trimmed_value, template_prop_names);
            let generated_value = rewritten_value
                .as_ref()
                .map_or_else(|| value.as_ref(), |s| s.as_str());
            append!(
                *ts,
                "{indent}// @vize-map: prop -> {prop_src_start}:{prop_src_end}\n",
            );

            let safe_prop_name = to_safe_identifier_fragment(prop.name.as_str());
            let expr_indent = if usage.vif_guard.is_some() {
                cstr!("{indent}  ")
            } else {
                indent.into()
            };

            if let Some(ref guard) = usage.vif_guard {
                append!(*ts, "{indent}if ({guard}) {{\n");
            }

            let gen_stmt_start = ts.len();
            let check_name = cstr!("__vize_prop_check_{idx}_{safe_prop_name}");
            append!(
                *ts,
                "{expr_indent}const {check_name}: __{component_type_name}_{idx}_prop_{safe_prop_name} = {};\n",
                generated_value,
            );
            let gen_stmt_end = ts.len();
            append!(*ts, "{expr_indent}void {check_name};\n");
            mappings.push(VizeMapping {
                gen_range: gen_stmt_start..gen_stmt_end,
                src_range: prop_src_start..prop_src_end,
                sub_spans: Vec::new(),
            });

            if usage.vif_guard.is_some() {
                append!(*ts, "{indent}}}\n");
            }
        }
    }

    generate_generic_props_call(
        ts,
        mappings,
        usage,
        idx,
        template_prop_names,
        template_offset,
        indent,
    );
}

/// Emit a single call into the child's generic functional prop-checker (#775),
/// assembling the dynamic props into one object literal so TypeScript can infer
/// the child's generic parameter(s) across the boundary. For a non-generic /
/// built-in / library / `any` component the checker resolves to a
/// `(props: any) => void` no-op (see `__VizePropChecker` in scope.rs), so this
/// call reports nothing and the well-tested per-prop extraction above is the
/// sole check. Each property value is mapped back to its source attribute so a
/// `TS2322` from a wrongly-typed prop points at the offending binding.
fn generate_generic_props_call(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    usage: &ComponentUsage,
    idx: usize,
    template_prop_names: &FxHashSet<String>,
    template_offset: u32,
    indent: &str,
) {
    let has_dynamic_props = usage.props.iter().any(|p| {
        p.name.as_str() != "key" && p.name.as_str() != "ref" && p.value.is_some() && p.is_dynamic
    });
    if !has_dynamic_props {
        return;
    }

    let component_type_name = to_safe_identifier_fragment(usage.name.as_str());
    let expr_indent = if usage.vif_guard.is_some() {
        cstr!("{indent}  ")
    } else {
        indent.into()
    };

    if let Some(ref guard) = usage.vif_guard {
        append!(*ts, "{indent}if ({guard}) {{\n");
    }

    append!(
        *ts,
        "{expr_indent}(undefined as unknown as __{component_type_name}_Check_{idx})({{\n",
    );

    for prop in &usage.props {
        if prop.name.as_str() == "key" || prop.name.as_str() == "ref" {
            continue;
        }
        let Some(ref value) = prop.value else {
            continue;
        };
        if !prop.is_dynamic {
            continue;
        }

        let prop_src_start = (template_offset + prop.start) as usize;
        let prop_src_end = (template_offset + prop.end) as usize;
        let value = strip_js_comments(value.as_str());
        let trimmed_value = value.as_ref().trim();
        let rewritten_value = rewrite_reserved_template_prop(trimmed_value, template_prop_names);
        let generated_value = rewritten_value
            .as_ref()
            .map_or_else(|| value.as_ref(), |s| s.as_str());
        let camel_prop_name = to_camel_case(prop.name.as_str());

        append!(*ts, "{expr_indent}  ");
        // Map the whole `"prop": value` entry (key through value) back to the
        // source attribute. TypeScript reports an assignability error for an
        // object-literal property at the property key, not the value, so a
        // value-only mapping would miss it and the diagnostic would be dropped.
        let entry_gen_start = ts.len();
        append!(*ts, "\"{camel_prop_name}\": {generated_value}");
        let entry_gen_end = ts.len();
        ts.push_str(",\n");
        mappings.push(VizeMapping {
            gen_range: entry_gen_start..entry_gen_end,
            src_range: prop_src_start..prop_src_end,
            sub_spans: Vec::new(),
        });
    }

    append!(*ts, "{expr_indent}}});\n");

    if usage.vif_guard.is_some() {
        append!(*ts, "{indent}}}\n");
    }
}
