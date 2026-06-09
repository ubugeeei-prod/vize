//! Template expression statement generation.
//!
//! Emits TypeScript `void (...)` statements for template expressions, with
//! optional v-if narrowing, and delegates recognized v-if chains to the
//! control-flow emitter in [`super::vif_chain`].

use super::super::helpers::generated_text_range;
use super::super::types::VizeMapping;
use super::reserved_props::rewrite_reserved_template_prop;
use super::vif_chain::{VifControlFlowChain, emit_vif_control_flow_chain};
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::append;
use vize_carton::cstr;
use vize_carton::profile;
use vize_croquis::analysis::{TemplateExpression, TemplateExpressionKind};
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
        if expr.kind == TemplateExpressionKind::VIf {
            generate_vif_guard_expression(
                ts,
                mappings,
                expr,
                guard.as_str(),
                template_prop_names,
                template_offset,
                indent,
            );
            return;
        }

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

fn generate_vif_guard_expression(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    expr: &TemplateExpression,
    guard: &str,
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
    let trimmed_guard = guard.trim();
    let rewritten_guard = rewrite_reserved_template_prop(trimmed_guard, template_prop_names);
    let generated_guard = rewritten_guard
        .as_ref()
        .map_or_else(|| guard, |s| s.as_str());
    let mapping_needle = if generated_guard.contains(generated_expression) {
        generated_expression
    } else {
        generated_guard
    };

    let gen_stmt_start = ts.len();
    append!(*ts, "{indent}if ({generated_guard}) {{\n");
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
    append!(
        *ts,
        "{indent}  // @vize-map: expr -> {src_start}:{src_end}\n",
    );
    append!(*ts, "{indent}}}\n");
}

pub(super) fn generate_expression_statement(
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
