//! Expression and component prop check generation for virtual TypeScript.
//!
//! Handles generating TypeScript code for template expressions (with optional
//! v-if narrowing) and component prop value type assertions.

use super::{helpers::generated_text_range, types::VizeMapping};
use vize_carton::append;
use vize_carton::cstr;
use vize_carton::String;
use vize_croquis::analysis::ComponentUsage;
use vize_croquis::analyzer::strip_js_comments;

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
    template_offset: u32,
    indent: &str,
) {
    let src_start = (template_offset + expr.start) as usize;
    let src_end = (template_offset + expr.end) as usize;
    let expression = strip_js_comments(expr.content.as_str());

    if let Some(ref guard) = expr.vif_guard {
        // Wrap in if block for type narrowing
        append!(*ts, "{indent}if ({guard}) {{\n");
        let gen_stmt_start = ts.len();
        append!(
            *ts,
            "{indent}  void ({}); // {}\n",
            expression.as_ref(),
            expr.kind.as_str()
        );
        let gen_stmt_end = ts.len();
        mappings.push(VizeMapping {
            gen_range: generated_text_range(
                &ts[gen_stmt_start..gen_stmt_end],
                expression.as_ref(),
                gen_stmt_start,
            ),
            src_range: src_start..src_end,
        });
        append!(
            *ts,
            "{indent}  // @vize-map: expr -> {src_start}:{src_end}\n",
        );
        append!(*ts, "{indent}}}\n");
    } else {
        let gen_stmt_start = ts.len();
        append!(
            *ts,
            "{indent}void ({}); // {}\n",
            expression.as_ref(),
            expr.kind.as_str()
        );
        let gen_stmt_end = ts.len();
        mappings.push(VizeMapping {
            gen_range: generated_text_range(
                &ts[gen_stmt_start..gen_stmt_end],
                expression.as_ref(),
                gen_stmt_start,
            ),
            src_range: src_start..src_end,
        });
        append!(*ts, "{indent}// @vize-map: expr -> {src_start}:{src_end}\n",);
    }
}

/// Generate component prop value checks at the given indentation level.
pub(crate) fn generate_component_prop_checks(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    usage: &ComponentUsage,
    idx: usize,
    template_offset: u32,
    indent: &str,
) {
    let component_name = &usage.name;
    for prop in &usage.props {
        if prop.name.as_str() == "key" || prop.name.as_str() == "ref" {
            continue;
        }
        if let Some(ref value) = prop.value {
            if prop.is_dynamic {
                let prop_src_start = (template_offset + prop.start) as usize;
                let prop_src_end = (template_offset + prop.end) as usize;
                let value = strip_js_comments(value.as_str());
                append!(
                    *ts,
                    "{indent}// @vize-map: prop -> {prop_src_start}:{prop_src_end}\n",
                );

                let safe_prop_name = prop.name.replace('-', "_");
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
                    "{expr_indent}const {check_name}: __{component_name}_{idx}_prop_{safe_prop_name} = {};\n",
                    value.as_ref(),
                );
                let gen_stmt_end = ts.len();
                append!(*ts, "{expr_indent}void {check_name};\n");
                mappings.push(VizeMapping {
                    gen_range: gen_stmt_start..gen_stmt_end,
                    src_range: prop_src_start..prop_src_end,
                });

                if usage.vif_guard.is_some() {
                    append!(*ts, "{indent}}}\n");
                }
            }
        }
    }
}
