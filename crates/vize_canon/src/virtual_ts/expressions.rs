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

    if let Some(ref guard) = expr.vif_guard {
        let trimmed_guard = guard.as_str().trim();
        let rewritten_guard = rewrite_reserved_template_prop(trimmed_guard, template_prop_names);
        let generated_guard = rewritten_guard
            .as_ref()
            .map_or_else(|| guard.as_str(), |s| s.as_str());
        // Wrap in if block for type narrowing
        append!(*ts, "{indent}if ({generated_guard}) {{\n");
        let gen_stmt_start = ts.len();
        append!(
            *ts,
            "{indent}  void ({}); // {}\n",
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
