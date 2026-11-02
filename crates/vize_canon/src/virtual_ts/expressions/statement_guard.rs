//! Authored condition emission for a nested v-if guard.

use super::{
    reserved_props::{map_rewritten_template_binding, rewrite_reserved_template_binding},
    ts_suppression_comments::expression_source_for_typecheck,
};
use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
use crate::virtual_ts::{VizeMapping, helpers::generated_text_range};
use oxc_allocator::Allocator;
use oxc_ast::ast::Expression;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::{String, append, cstr, profile};
use vize_croquis::TemplateExpression;

pub(super) fn generate_vif_guard_expression(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    expr: &TemplateExpression,
    guard: &str,
    template_binding_access: &TemplateBindingAccess,
    template_offset: u32,
    indent: &str,
) {
    let src_start = (template_offset + expr.start) as usize;
    let src_end = (template_offset + expr.end) as usize;
    let expression = profile!(
        "canon.virtual_ts.expression.strip_comments",
        expression_source_for_typecheck(expr.content.as_str())
    );
    let trimmed_expression = expression.as_ref().trim();
    let rewritten_expression =
        rewrite_reserved_template_binding(trimmed_expression, template_binding_access);
    let generated_expression = rewritten_expression
        .as_ref()
        .map_or_else(|| expression.as_ref(), |s| s.as_str());
    let trimmed_guard = guard.trim();
    let rewritten_guard = rewrite_reserved_template_binding(trimmed_guard, template_binding_access);
    let generated_guard = rewritten_guard
        .as_ref()
        .map_or_else(|| guard, |s| s.as_str());
    let isolated_guard = super::incomplete::isolate_incomplete_expression(generated_guard);
    let generated_guard = isolated_guard.as_str();
    let enclosing = enclosing_guard(generated_guard, generated_expression);
    let outer_indent = indent;
    let inner_indent = enclosing.map(|parent| {
        crate::virtual_ts::scope::append_ignored_vif_guard_open(
            ts,
            indent,
            parent,
            "Repeated parent guard",
        );
        cstr!("{indent}  ")
    });
    let indent = inner_indent.as_deref().unwrap_or(indent);
    let generated_guard = if enclosing.is_some() {
        generated_expression
    } else {
        generated_guard
    };
    let mapping_needle = if isolated_guard.is_owned() {
        isolated_guard.mapped_str()
    } else if generated_guard.contains(generated_expression) {
        generated_expression
    } else {
        generated_guard
    };

    let gen_stmt_start = ts.len();
    append!(*ts, "{indent}if ({generated_guard}) {{\n");
    let gen_stmt_end = ts.len();
    mappings.push(VizeMapping {
        gen_range: generated_text_range(
            ts.get(gen_stmt_start..gen_stmt_end).unwrap_or_default(),
            mapping_needle,
            gen_stmt_start,
        ),
        src_range: src_start..src_end,
        sub_spans: Vec::new(),
    });
    map_rewritten_template_binding(
        ts,
        mappings,
        gen_stmt_start,
        src_start + expression.len() - expression.trim_start().len(),
        trimmed_expression,
        template_binding_access,
    );
    append!(
        *ts,
        "{indent}  // @vize-map: expr -> {src_start}:{src_end}\n",
    );
    append!(*ts, "{indent}}}\n");
    if enclosing.is_some() {
        append!(*ts, "{outer_indent}}}\n");
    }
}

/// A generated parent conjunction ends in the exact authored own condition.
/// Parse the logical boundary instead of splitting tokens inside strings.
fn enclosing_guard<'s>(guard: &'s str, own: &str) -> Option<&'s str> {
    if !guard.contains(" && ") || !vize_l0::expression_guard::expression_is_safe_to_parse(guard) {
        return None;
    }
    let allocator = Allocator::new();
    let expression = Parser::new(&allocator, guard, SourceType::ts())
        .parse_expression()
        .ok()?;
    let Expression::LogicalExpression(logical) = expression else {
        return None;
    };
    if !logical.operator.is_and() {
        return None;
    }
    let full = logical.span;
    if guard.get(full.start as usize..full.end as usize) != Some(guard.trim()) {
        return None;
    }
    let right = logical.right.span();
    if guard.get(right.start as usize..right.end as usize)
        != Some(cstr!("({})", own.trim()).as_str())
    {
        return None;
    }
    let left = logical.left.span();
    guard.get(left.start as usize..left.end as usize)
}
