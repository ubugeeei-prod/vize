//! Component prop value type-check generation.
//!
//! For each prop passed to a child component usage — dynamic bindings and
//! static attribute values alike — emits a typed assertion plus a single call
//! into the child's generic functional prop-checker so TypeScript can
//! validate the values and infer generics across the component boundary.

use super::super::helpers::{to_camel_case, to_safe_identifier_fragment};
use super::super::types::{VizeMapping, VizeSubSpan};
use super::prop_sources::{generated_prop_value, prop_name_source_range, prop_value_source_range};
use vize_carton::FxHashMap;
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::append;
use vize_carton::cstr;
use vize_carton::profile;
use vize_croquis::croquis::{ComponentUsage, PassedProp};

fn has_inference_props(usage: &ComponentUsage) -> bool {
    usage.props.iter().any(is_checkable_prop)
}

fn is_checkable_prop(prop: &PassedProp) -> bool {
    !prop.name_is_dynamic && prop.name.as_str() != "key" && prop.name.as_str() != "ref"
}

#[derive(Clone, Copy)]
pub(crate) struct ComponentPropSource<'a> {
    pub(crate) template: Option<&'a str>,
    pub(crate) offset: u32,
}

impl<'a> ComponentPropSource<'a> {
    pub(crate) const fn new(template: Option<&'a str>, offset: u32) -> Self {
        Self { template, offset }
    }
}

fn collect_generated_class_bindings<'a>(
    usage: &'a ComponentUsage,
    template_prop_names: &FxHashSet<String>,
) -> Vec<(&'a PassedProp, String)> {
    usage
        .props
        .iter()
        .filter(|prop| is_checkable_prop(prop) && prop.name.as_str() == "class")
        .filter_map(|prop| {
            generated_prop_value(prop, template_prop_names).map(|value| (prop, value))
        })
        .collect()
}

fn merged_class_binding_value(bindings: &[(&PassedProp, String)]) -> Option<String> {
    match bindings {
        [] => None,
        [(_, value)] => Some(value.clone()),
        _ => {
            let mut value = String::default();
            value.push('[');
            for (index, (_, binding_value)) in bindings.iter().enumerate() {
                if index > 0 {
                    value.push_str(", ");
                }
                value.push_str(binding_value.as_str());
            }
            value.push(']');
            Some(value)
        }
    }
}

/// Generate component prop value checks at the given indentation level.
pub(crate) fn generate_component_prop_checks(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    usage: &ComponentUsage,
    idx: usize,
    template_prop_names: &FxHashSet<String>,
    source_context: ComponentPropSource<'_>,
    indent: &str,
) {
    let component_type_name = to_safe_identifier_fragment(usage.name.as_str());
    let mut name_occurrences: FxHashMap<String, u32> = FxHashMap::default();
    for prop in &usage.props {
        if !is_checkable_prop(prop) {
            continue;
        }
        // Static attribute values are checked exactly like dynamic bindings:
        // a static `msg="text"` must still satisfy the child's prop type
        // (vue-tsc reports TS2322 here; skipping them was a false negative).
        if prop.value.is_some() {
            let prop_src_start = (source_context.offset + prop.start) as usize;
            let prop_src_end = (source_context.offset + prop.end) as usize;
            let value_src_range = prop_value_source_range(source_context, prop);
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

            // A repeated attribute name (static class next to :class) still
            // checks every authored value, but each check constant needs a
            // unique name or the virtual TS redeclares it (TS2451).
            let occurrence = name_occurrences
                .entry(String::from(safe_prop_name.as_str()))
                .and_modify(|count| *count += 1)
                .or_insert(1);
            let check_name = if *occurrence == 1 {
                cstr!("__vize_prop_check_{idx}_{safe_prop_name}")
            } else {
                cstr!("__vize_prop_check_{idx}_{safe_prop_name}_{occurrence}")
            };
            let gen_stmt_start = ts.len();
            append!(*ts, "{expr_indent}const ");
            let check_name_start = ts.len();
            ts.push_str(check_name.as_str());
            let check_name_end = ts.len();
            append!(
                *ts,
                ": __{component_type_name}_{idx}_prop_{safe_prop_name} = ",
            );
            let value_gen_start = ts.len();
            ts.push_str(generated_value.as_str());
            let value_gen_end = ts.len();
            ts.push_str(";\n");
            let gen_stmt_end = ts.len();
            append!(*ts, "{expr_indent}void {check_name};\n");

            // The synthetic identifier receives the child prop-type error
            // (TS2322-class), which vue-tsc anchors at the attribute name;
            // the initializer keeps the exact authored expression so errors
            // inside the value land on the authored bytes.
            let name_src_range = prop_name_source_range(source_context, prop);
            let mut sub_spans = Vec::new();
            if let Some(src_range) = name_src_range.or_else(|| value_src_range.clone()) {
                sub_spans.push(VizeSubSpan {
                    gen_range: check_name_start..check_name_end,
                    src_range,
                });
            }
            if let Some(src_range) = value_src_range {
                sub_spans.push(VizeSubSpan {
                    gen_range: value_gen_start..value_gen_end,
                    src_range,
                });
            }
            mappings.push(VizeMapping {
                gen_range: gen_stmt_start..gen_stmt_end,
                src_range: prop_src_start..prop_src_end,
                sub_spans,
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
        source_context.offset,
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

    let class_bindings = collect_generated_class_bindings(usage, template_prop_names);
    let merge_class_bindings = class_bindings.len() > 1;
    let mut emitted_merged_class = false;
    for prop in &usage.props {
        if !is_checkable_prop(prop) {
            continue;
        }

        let generated_value = if merge_class_bindings && prop.name.as_str() == "class" {
            if emitted_merged_class {
                continue;
            }
            emitted_merged_class = true;
            merged_class_binding_value(&class_bindings)
        } else {
            generated_prop_value(prop, template_prop_names)
        };
        let Some(generated_value) = generated_value else {
            continue;
        };

        let (prop_src_start, prop_src_end) =
            if merge_class_bindings && prop.name.as_str() == "class" {
                let start = class_bindings
                    .iter()
                    .map(|(binding, _)| binding.start)
                    .min()
                    .unwrap_or(prop.start);
                let end = class_bindings
                    .iter()
                    .map(|(binding, _)| binding.end)
                    .max()
                    .unwrap_or(prop.end);
                (
                    (template_offset + start) as usize,
                    (template_offset + end) as usize,
                )
            } else {
                (
                    (template_offset + prop.start) as usize,
                    (template_offset + prop.end) as usize,
                )
            };
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
