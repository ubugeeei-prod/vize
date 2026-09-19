//! Authored condition emission for a nested v-if guard.

use super::{
    reserved_props::rewrite_reserved_template_prop,
    ts_suppression_comments::expression_source_for_typecheck,
};
use crate::virtual_ts::{VizeMapping, helpers::generated_text_range};
use vize_carton::{FxHashSet, String, append, profile};
use vize_croquis::TemplateExpression;

pub(super) fn generate_vif_guard_expression(
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
        expression_source_for_typecheck(expr.content.as_str())
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
