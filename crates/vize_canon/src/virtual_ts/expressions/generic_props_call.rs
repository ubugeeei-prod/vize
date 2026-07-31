//! The single call into a child's generic functional prop-checker, and the
//! sub-span mapping for each entry of the props object literal it assembles.
//!
//! Split out of [`super::component_props`] so that module stays inside the
//! per-file source-length budget.

use super::super::helpers::{to_camel_case, to_safe_identifier_fragment};
use super::super::types::{VizeMapping, VizeSubSpan};
use super::component_props::{
    ComponentPropSource, collect_generated_class_bindings, is_checkable_prop,
    merged_class_binding_value,
};
use super::prop_sources::{
    append_prop_value, generated_prop_value, prop_name_source_range, prop_value_source_range,
};
use crate::virtual_ts::scope::has_checkable_props_or_spread;
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::append;
use vize_carton::cstr;
use vize_croquis::croquis::{ComponentUsage, PassedProp, SpreadProp};

/// Emit a single call into the child's generic functional prop-checker (#775),
/// assembling the dynamic props into one object literal so TypeScript can infer
/// the child's generic parameter(s) across the boundary. For a non-generic /
/// built-in / library / `any` component the checker resolves to a
/// `(props: any) => void` no-op (see `__VizePropChecker` in scope.rs), so this
/// call reports nothing and the well-tested per-prop extraction above is the
/// sole check. Each property value is mapped back to its source attribute so a
/// `TS2322` from a wrongly-typed prop points at the offending binding.
pub(super) fn generate_generic_props_call(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    usage: &ComponentUsage,
    idx: usize,
    template_prop_names: &FxHashSet<String>,
    source_context: ComponentPropSource<'_>,
    indent: &str,
) {
    let template_offset = source_context.offset;
    if !has_checkable_props_or_spread(usage) {
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
        "{expr_indent}(undefined as unknown as __{component_type_name}_Check_{idx})(",
    );
    // TypeScript anchors an *argument* assignability failure — `TS2345` — at the
    // argument expression, which here is the whole object literal and not any
    // one property. Without a mapping over that range the diagnostic has no
    // authored position and the diagnostics path drops it silently, which is
    // what made the `exactOptionalPropertyTypes` check (#3450) fire in the
    // generated program and report nothing to the user.
    //
    // The per-entry mappings pushed below stay inside this range and remain the
    // anchor for a per-property failure; this one only catches what TypeScript
    // reports about the literal as a whole.
    let literal_gen_start = ts.len();
    ts.push_str("{\n");

    let class_bindings = collect_generated_class_bindings(usage, template_prop_names);
    let merge_class_bindings = class_bindings.len() > 1;
    let mut emitted_merged_class = false;
    // `v-bind="obj"` becomes a spread in the same literal, so the child's props
    // type sees the whole bag: `({ ...bag, "label": maybe })`. vue-tsc checks the
    // spread's contribution the same way, which is what makes a wrongly typed
    // bag `TS2345` rather than silence (#3444).
    //
    // Entries follow template order, because Vue 3 resolves an overlapping key
    // by source order (last binding wins), exactly as an object literal does.
    // Spreading first unconditionally would check `1` for the `count` of
    // `:count="1" v-bind="bag"`, a key the runtime takes from `bag` instead.
    // Both lists are already sorted, so walking them together is enough.
    let mut spreads = usage.spread_props.iter().peekable();
    for prop in &usage.props {
        if !is_checkable_prop(prop) {
            continue;
        }
        while let Some(spread) = spreads.next_if(|spread| spread.start < prop.start) {
            append_spread_entry(ts, mappings, spread, template_offset, expr_indent.as_str());
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
        append!(*ts, "\"{camel_prop_name}\"");
        let key_gen_end = ts.len();
        ts.push_str(": ");
        let value_gen_range = append_prop_value(ts, generated_value.as_str());
        let entry_gen_end = ts.len();
        ts.push_str(",\n");
        // Without sub-spans the whole entry maps proportionally onto the whole
        // attribute, and `"prop": ` is one byte longer than `:prop="` — so an
        // error inside the value (TypeScript anchors a nested object-literal
        // mismatch at the offending key) landed one byte to the right of where
        // vue-tsc puts it (#3446). Mapping the key and the value separately
        // makes the value range verbatim, so offsets inside it are exact.
        let sub_spans = match merge_class_bindings && prop.name.as_str() == "class" {
            true => Vec::new(),
            false => entry_sub_spans(
                source_context,
                prop,
                entry_gen_start..key_gen_end,
                value_gen_range,
            ),
        };
        mappings.push(VizeMapping {
            gen_range: entry_gen_start..entry_gen_end,
            src_range: prop_src_start..prop_src_end,
            sub_spans,
        });
    }
    for spread in spreads {
        append_spread_entry(ts, mappings, spread, template_offset, expr_indent.as_str());
    }

    append!(*ts, "{expr_indent}}}");
    let literal_gen_end = ts.len();
    ts.push_str(");\n");
    // Anchored on the tag *name*, not on `usage.start`, which is the `<` one
    // byte to its left. `vue-tsc` puts a whole-props failure on the name — the
    // #3450 oracle reads `src/Parent.vue(6,12)` for `<template><Child …`, where
    // column 12 is the `C`. Same derivation as the navigation references in
    // `scope::component_prop_navigation`.
    let tag_src_start = (template_offset + usage.start + 1) as usize;
    mappings.push(VizeMapping {
        gen_range: literal_gen_start..literal_gen_end,
        src_range: tag_src_start..tag_src_start + usage.name.len(),
        sub_spans: Vec::new(),
    });

    if usage.vif_guard.is_some() {
        append!(*ts, "{indent}}}\n");
    }
}

/// Emit one `...expr` entry of the props literal, mapped back to its own
/// directive so an error *inside* the expression lands on the authored bytes.
fn append_spread_entry(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    spread: &SpreadProp,
    template_offset: u32,
    expr_indent: &str,
) {
    append!(*ts, "{expr_indent}  ...");
    let gen_range = append_prop_value(ts, spread.expression.as_str());
    ts.push_str(",\n");
    mappings.push(VizeMapping {
        gen_range,
        src_range: (template_offset + spread.start) as usize
            ..(template_offset + spread.end) as usize,
        sub_spans: Vec::new(),
    });
}

/// Sub-spans for one `"prop": value` entry: the key maps to the authored
/// attribute name, the value to the authored expression. Returns an empty list
/// unless both are known, since a partial list would silently drop whichever
/// half is missing.
fn entry_sub_spans(
    source_context: ComponentPropSource<'_>,
    prop: &PassedProp,
    key_gen_range: std::ops::Range<usize>,
    value_gen_range: std::ops::Range<usize>,
) -> Vec<VizeSubSpan> {
    let Some(name_src_range) = prop_name_source_range(source_context, prop) else {
        return Vec::new();
    };
    let Some(value_src_range) = prop_value_source_range(source_context, prop) else {
        return Vec::new();
    };
    vec![
        VizeSubSpan {
            gen_range: key_gen_range,
            src_range: name_src_range,
        },
        VizeSubSpan {
            gen_range: value_gen_range,
            src_range: value_src_range,
        },
    ]
}
