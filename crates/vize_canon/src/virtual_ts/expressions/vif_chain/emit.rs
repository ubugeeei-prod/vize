//! Emit planned v-if chains with typed binding receivers and authored mappings.

use super::{GuardTerm, VifBranch, VifControlFlowChain};
use crate::virtual_ts::expressions::{
    map_rewritten_template_binding, rewrite_reserved_template_binding,
    statements::{ExpressionListEmitContext, generate_expression_statement},
};
use crate::virtual_ts::{VizeMapping, template_binding_access::TemplateBindingAccess};
use vize_carton::{String, append, cstr};
use vize_croquis::TemplateExpression;

struct BranchContext<'a, 'b> {
    exprs: &'a [&'b TemplateExpression],
    bindings: &'a TemplateBindingAccess,
    context: &'a ExpressionListEmitContext<'b>,
}

pub(in crate::virtual_ts::expressions) fn emit_vif_control_flow_chain(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    exprs: &[&TemplateExpression],
    chain: &VifControlFlowChain<'_>,
    template_binding_access: &TemplateBindingAccess,
    context: &ExpressionListEmitContext<'_>,
) {
    let branch_context = BranchContext {
        exprs,
        bindings: template_binding_access,
        context,
    };
    for (branch_index, branch) in chain.branches.iter().enumerate() {
        emit_vif_branch_open(
            ts,
            mappings,
            chain,
            branch,
            branch_index == 0,
            &branch_context,
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
                template_binding_access,
                context.template_offset,
                &body_indent,
                context.checks,
            );
        }
    }
    append!(*ts, "{}}}\n", context.indent);
}

fn emit_vif_branch_open(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    chain: &VifControlFlowChain<'_>,
    branch: &VifBranch<'_>,
    first: bool,
    ctx: &BranchContext<'_, '_>,
) {
    let context = ctx.context;
    let exprs = ctx.exprs;
    let prefix_is_empty = chain.prefix.is_empty();
    match (first, branch.condition) {
        (true, Some(condition)) => {
            append!(*ts, "{}if (", context.indent);
            append_guard_condition(ts, &chain.prefix, Some(condition), mappings, branch, ctx);
            ts.push_str(") {\n");
        }
        (false, Some(condition)) => {
            append!(*ts, "{}}} else if (", context.indent);
            append_guard_condition(ts, &chain.prefix, Some(condition), mappings, branch, ctx);
            ts.push_str(") {\n");
        }
        (false, None) if prefix_is_empty => {
            append!(*ts, "{}}} else {{\n", context.indent);
        }
        (false, None) => {
            append!(*ts, "{}}} else if (", context.indent);
            append_binding_expression(ts, branch.guard, ctx.bindings);
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

fn append_guard_condition(
    ts: &mut String,
    prefix: &[GuardTerm<'_>],
    condition: Option<&str>,
    mappings: &mut Vec<VizeMapping>,
    branch: &VifBranch<'_>,
    ctx: &BranchContext<'_, '_>,
) {
    let template_offset = ctx.context.template_offset;
    let exprs = ctx.exprs;
    append_guard_prefix(ts, prefix, ctx.bindings);
    if let Some(condition) = condition {
        if !prefix.is_empty() {
            ts.push_str(" && (");
        }
        let gen_start = ts.len();
        append_binding_expression(ts, condition, ctx.bindings);
        let gen_end = ts.len();
        if let Some(expr_index) = branch.condition_expr_index {
            let expr = exprs[expr_index];
            mappings.push(VizeMapping {
                gen_range: gen_start..gen_end,
                src_range: (template_offset + expr.start) as usize
                    ..(template_offset + expr.end) as usize,
                sub_spans: Vec::new(),
            });
            map_rewritten_template_binding(
                ts,
                mappings,
                gen_start,
                (template_offset + expr.start) as usize,
                condition,
                ctx.bindings,
            );
        }
        if !prefix.is_empty() {
            ts.push(')');
        }
    }
}

fn append_guard_prefix(
    ts: &mut String,
    prefix: &[GuardTerm<'_>],
    bindings: &TemplateBindingAccess,
) {
    for (index, term) in prefix.iter().enumerate() {
        if index > 0 {
            ts.push_str(" && ");
        }
        append_binding_expression(ts, term.raw, bindings);
    }
}

fn append_binding_expression(ts: &mut String, expression: &str, bindings: &TemplateBindingAccess) {
    let rewritten = rewrite_reserved_template_binding(expression, bindings);
    ts.push_str(rewritten.as_deref().unwrap_or(expression));
}
