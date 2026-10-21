//! The single call into a child's generic functional prop-checker, and the
//! slot-resolver call that instantiates the child's generics for its slots.
//!
//! Split out of [`super::component_props`] so that module stays inside the
//! per-file source-length budget. The props object literal both calls pass is
//! assembled by [`super::props_literal`].

use super::super::helpers::to_safe_identifier_fragment;
use super::super::scope::{append_chained_vif_guard_close, append_chained_vif_guard_open};
use super::super::types::VizeMapping;
use super::component_props::ComponentPropSource;
use super::props_literal::{append_inference_props_literal, append_props_literal};
use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
use vize_carton::String;
use vize_carton::append;
use vize_carton::cstr;
use vize_croquis::croquis::ComponentUsage;

/// Emit a single call into the child's generic functional prop-checker (#775),
/// assembling the dynamic props into one object literal so TypeScript can infer
/// the child's generic parameter(s) across the boundary. For a non-generic /
/// built-in / library / `any` component the checker resolves to a
/// `(props: any) => void` no-op (see `__VizePropChecker` in scope.rs), so this
/// call reports nothing and the well-tested per-prop extraction above is the
/// sole check. Each property value is mapped back to its source attribute so a
/// `TS2322` from a wrongly-typed prop points at the offending binding.
///
/// Direct inline callbacks are the exception: their authored expression is
/// checked separately against the resolver's instantiated prop type. This call
/// receives `never` for that one entry so it keeps whole-object/required-prop
/// validation without also reporting a coarse property-key error.
#[expect(
    clippy::too_many_arguments,
    reason = "generic prop checks share the component source context"
)]
pub(super) fn generate_generic_props_call(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    usage: &ComponentUsage,
    idx: usize,
    template_binding_access: &TemplateBindingAccess,
    source_context: ComponentPropSource<'_>,
    indent: &str,
    typed_fallthrough_values: bool,
) {
    let component_type_name = to_safe_identifier_fragment(usage.name.as_str());
    let expr_indent = if usage.vif_guard.is_some() {
        cstr!("{indent}  ")
    } else {
        indent.into()
    };

    if let Some(ref guard) = usage.vif_guard {
        append_chained_vif_guard_open(ts, indent, guard, "Inference-only guard");
    }

    append!(
        *ts,
        "{expr_indent}(undefined as unknown as __{component_type_name}_Check_{idx})(",
    );
    let check_spread_values = typed_fallthrough_values && !usage.spread_props.is_empty();
    // TypeScript anchors an *argument* assignability failure — `TS2345` — at the
    // opening brace of the object literal. Mapping only that byte gives the
    // broad failure an authored owner without overlapping the per-property
    // rows nested inside the literal. Content Mapper forbids overlapping
    // virtual spans, so mapping the complete literal made its normalizer drop
    // this tag-name owner and the host fell through to the end of the tag.
    //
    // The per-entry mappings pushed below stay inside this range and remain the
    // anchor for a per-property failure; this one only catches what TypeScript
    // reports about the literal as a whole.
    let literal_range = append_props_literal(
        ts,
        mappings,
        usage,
        template_binding_access,
        source_context,
        expr_indent.as_str(),
    );
    ts.push_str(");\n");
    let fallthrough_argument_range = check_spread_values.then(|| {
        append!(
            *ts,
            "{expr_indent}const __VizeFallthroughProps_{idx} = ",
        );
        append_inference_props_literal(
            ts,
            usage,
            template_binding_access,
            source_context,
            expr_indent.as_str(),
        );
        ts.push_str(";\n");
        append!(
            *ts,
            "{expr_indent}(undefined as unknown as __VizeFallthroughSpreadChecker<__{component_type_name}_Component_{idx}, __{component_type_name}_Props_{idx}>)(",
        );
        let start = ts.len();
        append!(*ts, "__VizeFallthroughProps_{idx}");
        let range = start..ts.len();
        ts.push_str(");\n");
        range
    });
    // Anchored on the tag *name*, not on `usage.start`, which is the `<` one
    // byte to its left. `vue-tsc` puts a whole-props failure on the name — the
    // #3450 oracle reads `src/Parent.vue(6,12)` for `<template><Child …`, where
    // column 12 is the `C`. Same derivation as the navigation references in
    // `scope::component_prop_navigation`.
    if let Some(src_range) = crate::virtual_ts::component_tag::source_range(
        source_context.template,
        source_context.offset,
        usage,
    ) {
        mappings.push(VizeMapping {
            gen_range: literal_range.start..literal_range.start + 1,
            src_range: src_range.clone(),
            sub_spans: Vec::new(),
        });
        if let Some(gen_range) = fallthrough_argument_range {
            mappings.push(VizeMapping {
                gen_range,
                src_range,
                sub_spans: Vec::new(),
            });
        }
    }

    if usage.vif_guard.is_some() {
        append_chained_vif_guard_close(ts, indent);
    }
}

/// Emit the inference-only binding a `v-slot` scope reads its payload type
/// from: `const <name> = factory(C)(<the authored props>);`.
///
/// The child's slot map is a function of its generic parameters, and the only
/// construct that instantiates those the way `vue-tsc` does is a *call* with
/// the authored props — a type-level probe against the construct signature
/// erases every parameter to its constraint even when the props determine it.
/// Higher-order inference preserves native functional/constructor generics;
/// Vize components retain their authored resolver signature directly.
/// Select one factory signature before inference: overloading the factory
/// erases generics in library declarations whose parameters reference their
/// own setup context (for example vue-virtual-scroller).
///
/// Every mapping this literal would produce is discarded: the identical literal
/// is already emitted, mapped, against the child's prop checker, so an authored
/// prop error is reported exactly once from there. A generated position with no
/// mapping has no authored position and is dropped by the diagnostics path, so
/// this construct is diagnostically inert by construction.
///
/// The `v-if` guard is reproduced as a conditional *expression* rather than a
/// statement so the narrowing that guard performs still applies to the props
/// while the binding stays in the enclosing block scope.
pub(crate) fn generate_slot_host_binding(
    ts: &mut String,
    usage: &ComponentUsage,
    binding_name: &str,
    component_ref: &str,
    template_binding_access: &TemplateBindingAccess,
    source_context: ComponentPropSource<'_>,
    indent: &str,
) {
    let mut discarded = Vec::new();
    if usage.vif_guard.is_some() {
        append!(
            *ts,
            "{indent}// @ts-ignore Inference-only guard; authored v-if checks own diagnostics.\n"
        );
    }
    append!(
        *ts,
        "{indent}const {binding_name} = (undefined as unknown as __VizeSlotsFactory<typeof {component_ref}>)({component_ref})(",
    );
    let isolated_guard = usage
        .vif_guard
        .as_ref()
        .map(|guard| super::incomplete::isolate_incomplete_expression(guard.as_str()));
    if let Some(guard) = isolated_guard.as_ref() {
        let guard = guard.as_str();
        append!(*ts, "({guard}) ? ");
    }
    append_props_literal(
        ts,
        &mut discarded,
        usage,
        template_binding_access,
        source_context,
        indent,
    );
    if usage.vif_guard.is_some() {
        ts.push_str(" : (undefined as unknown as never)");
    }
    ts.push_str(");\n");
    append!(*ts, "{indent}void {binding_name};\n");
}
