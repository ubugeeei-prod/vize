//! Template expression statement generation.
//!
//! Emits TypeScript `void (...)` statements for template expressions, with
//! optional v-if narrowing, and delegates recognized v-if chains to the
//! control-flow emitter in [`super::vif_chain`].

use super::super::helpers::generated_text_range;
use super::super::types::VizeMapping;
use super::component_ref_callbacks::generate_component_ref_callback_statement;
use super::directive_values::generate_directive_value_statement;
use super::native_props::generate_native_prop_statement;
use super::reserved_props::{map_rewritten_template_binding, rewrite_reserved_template_binding};
use super::statement_guard::generate_vif_guard_expression;
use super::ts_suppression_comments::expression_source_for_typecheck;
use super::value_checks::TemplateValueChecks;
use super::vif_chain::{VifControlFlowChain, emit_vif_control_flow_chain};
use crate::virtual_ts::scope::{append_ignored_vif_guard_open, remove_enclosing_vif_guard_prefix};
use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
use std::borrow::Cow;
use vize_carton::{CompactString, FxHashSet, String, append, cstr, profile};
use vize_croquis::croquis::{TemplateExpression, TemplateExpressionKind};

/// Generate template expressions, compacting recognized v-if chains into
/// TypeScript control-flow blocks.
pub(crate) fn generate_expressions(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    exprs: &[&TemplateExpression],
    template_binding_access: &TemplateBindingAccess,
    context: &ExpressionListEmitContext<'_>,
) {
    let mut index = 0;
    while index < exprs.len() {
        if context
            .skipped_expression_ranges
            .contains(&(exprs[index].start, exprs[index].end))
        {
            index += 1;
            continue;
        }
        if let Some(chain) = VifControlFlowChain::collect(exprs, index) {
            emit_vif_control_flow_chain(
                ts,
                mappings,
                exprs,
                &chain,
                template_binding_access,
                context,
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
                template_binding_access,
                context.template_offset,
                context.indent,
                context.checks,
            )
        );
        index += 1;
    }
}

/// Generate expressions inside control flow that already enforces a common
/// v-if guard, removing only that exact top-level guard prefix first.
pub(crate) fn generate_expressions_in_enclosing_guard(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    exprs: &[&TemplateExpression],
    template_binding_access: &TemplateBindingAccess,
    context: &ExpressionListEmitContext<'_>,
    enclosing_guard: Option<&str>,
) {
    let Some(enclosing_guard) = enclosing_guard else {
        generate_expressions(ts, mappings, exprs, template_binding_access, context);
        return;
    };

    let adjusted_expressions: Vec<_> = exprs
        .iter()
        .map(|expr| {
            let mut adjusted = (**expr).clone();
            adjusted.vif_guard = adjusted.vif_guard.as_ref().and_then(|guard| {
                remove_enclosing_vif_guard_prefix(guard.as_str(), enclosing_guard)
                    .map(|guard| CompactString::new(guard.as_str()))
            });
            adjusted
        })
        .collect();
    let adjusted_expression_refs: Vec<_> = adjusted_expressions.iter().collect();
    generate_expressions(
        ts,
        mappings,
        &adjusted_expression_refs,
        template_binding_access,
        context,
    );
}

pub(crate) struct ExpressionListEmitContext<'a> {
    pub(crate) skipped_expression_ranges: &'a FxHashSet<(u32, u32)>,
    pub(crate) template_offset: u32,
    pub(crate) indent: &'a str,
    pub(crate) checks: TemplateValueChecks<'a>,
}

impl<'a> ExpressionListEmitContext<'a> {
    pub(crate) fn new(
        skipped_expression_ranges: &'a FxHashSet<(u32, u32)>,
        template_offset: u32,
        indent: &'a str,
        checks: TemplateValueChecks<'a>,
    ) -> Self {
        Self {
            skipped_expression_ranges,
            template_offset,
            indent,
            checks,
        }
    }
}

/// Generate a template expression, wrapping guarded expressions so TypeScript
/// can narrow them.
pub(crate) fn generate_expression(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    expr: &vize_croquis::TemplateExpression,
    template_binding_access: &TemplateBindingAccess,
    template_offset: u32,
    indent: &str,
    checks: TemplateValueChecks<'_>,
) {
    if let Some(ref guard) = expr.vif_guard {
        if expr.kind == TemplateExpressionKind::VIf {
            generate_vif_guard_expression(
                ts,
                mappings,
                expr,
                guard.as_str(),
                template_binding_access,
                template_offset,
                indent,
            );
            return;
        }

        let trimmed_guard = guard.as_str().trim();
        let rewritten_guard =
            rewrite_reserved_template_binding(trimmed_guard, template_binding_access);
        let generated_guard = rewritten_guard
            .as_ref()
            .map_or_else(|| guard.as_str(), |s| s.as_str());
        // The authored condition has its own projection. This duplicate guard
        // only carries narrowing; mapping it onto the body would rename body
        // tokens when the condition's symbol is renamed.
        append_ignored_vif_guard_open(ts, indent, generated_guard, "Narrowing-only guard");
        generate_expression_statement(
            ts,
            mappings,
            expr,
            template_binding_access,
            template_offset,
            &cstr!("{indent}  "),
            checks,
        );
        append!(*ts, "{indent}}}\n");
    } else {
        generate_expression_statement(
            ts,
            mappings,
            expr,
            template_binding_access,
            template_offset,
            indent,
            checks,
        );
    }
}

pub(super) fn generate_expression_statement(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    expr: &TemplateExpression,
    template_binding_access: &TemplateBindingAccess,
    template_offset: u32,
    indent: &str,
    checks: TemplateValueChecks<'_>,
) {
    let generated_start = ts.len();
    emit_expression_statement(
        ts,
        mappings,
        expr,
        template_binding_access,
        template_offset,
        indent,
        checks,
    );
    if !template_binding_access.is_empty() {
        let expression = expression_source_for_typecheck(expr.content.as_str());
        let trimmed = expression.trim();
        let leading = expression.len() - expression.trim_start().len();
        map_rewritten_template_binding(
            ts,
            mappings,
            generated_start,
            (template_offset + expr.start) as usize + leading,
            trimmed,
            template_binding_access,
        );
    }
}

fn emit_expression_statement(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    expr: &TemplateExpression,
    template_binding_access: &TemplateBindingAccess,
    template_offset: u32,
    indent: &str,
    checks: TemplateValueChecks<'_>,
) {
    let src_start = (template_offset + expr.start) as usize;
    let src_end = (template_offset + expr.end) as usize;
    let is_component_ref_callback = checks
        .component_ref_callbacks
        .contains_key(&(expr.start, expr.end));
    let expression = if is_component_ref_callback {
        Cow::Borrowed(expr.content.as_str())
    } else {
        profile!(
            "canon.virtual_ts.expression.strip_comments",
            expression_source_for_typecheck(expr.content.as_str())
        )
    };
    let trimmed_expression = expression.as_ref().trim();
    if trimmed_expression.is_empty() {
        return;
    }
    let statement_expression =
        if expr.kind == TemplateExpressionKind::VOn && trimmed_expression.starts_with(';') {
            trimmed_expression
                .trim_start_matches(|character: char| character == ';' || character.is_whitespace())
        } else {
            expression.as_ref()
        };
    if statement_expression.is_empty() {
        return;
    }
    let rewritten_expression =
        rewrite_reserved_template_binding(statement_expression.trim(), template_binding_access);
    let generated_expression = rewritten_expression
        .as_ref()
        .map_or_else(|| statement_expression, |s| s.as_str());
    let mapping_needle = if rewritten_expression.is_some() {
        generated_expression
    } else {
        statement_expression
    };

    if let Some(native_prop) = checks.native_props.get(&(expr.start, expr.end)) {
        generate_native_prop_statement(
            ts,
            mappings,
            expr,
            native_prop,
            generated_expression,
            template_offset,
            indent,
        );
        return;
    }

    if is_component_ref_callback {
        generate_component_ref_callback_statement(
            ts,
            mappings,
            expr,
            generated_expression,
            template_offset,
            indent,
        );
        return;
    }

    if expr.kind == TemplateExpressionKind::CustomDirective
        && let Some(directive_value) = checks.directive_values.get(&(expr.start, expr.end))
    {
        generate_directive_value_statement(
            ts,
            mappings,
            expr,
            directive_value,
            generated_expression,
            template_offset,
            indent,
        );
        return;
    }

    if expr.kind == TemplateExpressionKind::DynamicDirectiveArgument {
        super::dynamic_arguments::generate(
            ts,
            mappings,
            expr,
            generated_expression,
            template_offset,
            indent,
            checks.template_source,
        );
        return;
    }
    let gen_stmt_start = ts.len();
    append!(
        *ts,
        "{indent}void ({generated_expression}); // {}\n",
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
