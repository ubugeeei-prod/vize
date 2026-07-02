//! Component prop value type-check generation.
//!
//! For each dynamic prop bound on a child component usage, emits a typed
//! assertion plus a single call into the child's generic functional
//! prop-checker so TypeScript can validate the bindings and infer generics
//! across the component boundary.

use super::super::helpers::{to_camel_case, to_safe_identifier_fragment};
use super::super::types::VizeMapping;
use super::reserved_props::rewrite_reserved_template_prop;
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::append;
use vize_carton::cstr;
use vize_carton::profile;
use vize_croquis::croquis::{ComponentUsage, PassedProp};
use vize_croquis::drawer::strip_js_comments;

fn push_ts_string_literal(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('"');
}

fn generated_prop_value(
    prop: &PassedProp,
    template_prop_names: &FxHashSet<String>,
) -> Option<String> {
    if !prop.is_dynamic {
        let mut value = String::default();
        if let Some(static_value) = prop.value.as_ref() {
            push_ts_string_literal(&mut value, static_value.as_str());
        } else {
            value.push_str("true");
        }
        return Some(value);
    }

    let value = strip_js_comments(prop.value.as_ref()?.as_str());
    let trimmed_value = value.as_ref().trim();
    let rewritten_value = rewrite_reserved_template_prop(trimmed_value, template_prop_names);
    Some(rewritten_value.as_ref().map_or_else(
        || String::from(value.as_ref()),
        |s| String::from(s.as_str()),
    ))
}

fn has_inference_props(usage: &ComponentUsage) -> bool {
    usage
        .props
        .iter()
        .any(|prop| prop.name.as_str() != "key" && prop.name.as_str() != "ref")
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
        if prop.value.is_some() && prop.is_dynamic {
            let prop_src_start = (template_offset + prop.start) as usize;
            let prop_src_end = (template_offset + prop.end) as usize;
            let generated_value = profile!(
                "canon.virtual_ts.prop_check.value",
                generated_prop_value(prop, template_prop_names).unwrap_or_default()
            );
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
                generated_value.as_str(),
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
    if !has_inference_props(usage) {
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
        let Some(generated_value) = generated_prop_value(prop, template_prop_names) else {
            continue;
        };

        let prop_src_start = (template_offset + prop.start) as usize;
        let prop_src_end = (template_offset + prop.end) as usize;
        let camel_prop_name = to_camel_case(prop.name.as_str());

        append!(*ts, "{expr_indent}  ");
        // Map the whole `"prop": value` entry (key through value) back to the
        // source attribute. TypeScript reports an assignability error for an
        // object-literal property at the property key, not the value, so a
        // value-only mapping would miss it and the diagnostic would be dropped.
        let entry_gen_start = ts.len();
        append!(*ts, "\"{camel_prop_name}\": {}", generated_value.as_str());
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
