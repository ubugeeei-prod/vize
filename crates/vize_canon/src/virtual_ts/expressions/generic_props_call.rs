//! The single call into a child's generic functional prop-checker, and the
//! slot-resolver call that instantiates the child's generics for its slots.
//!
//! Split out of [`super::component_props`] so that module stays inside the
//! per-file source-length budget. The props object literal both calls pass is
//! assembled by [`super::props_literal`].

use super::super::helpers::to_safe_identifier_fragment;
use super::super::scope::append_ignored_vif_guard_open;
use super::super::types::VizeMapping;
use super::component_props::ComponentPropSource;
use super::props_literal::append_props_literal;
use vize_carton::FxHashSet;
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
/// receives `any` for that one entry so it keeps whole-object/required-prop
/// validation without also reporting a coarse property-key error.
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
    let component_type_name = to_safe_identifier_fragment(usage.name.as_str());
    let expr_indent = if usage.vif_guard.is_some() {
        cstr!("{indent}  ")
    } else {
        indent.into()
    };

    if let Some(ref guard) = usage.vif_guard {
        append_ignored_vif_guard_open(ts, indent, guard, "Inference-only guard");
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
    let literal_range = append_props_literal(
        ts,
        mappings,
        usage,
        template_prop_names,
        source_context,
        expr_indent.as_str(),
    );
    ts.push_str(");\n");
    // Anchored on the tag *name*, not on `usage.start`, which is the `<` one
    // byte to its left. `vue-tsc` puts a whole-props failure on the name — the
    // #3450 oracle reads `src/Parent.vue(6,12)` for `<template><Child …`, where
    // column 12 is the `C`. Same derivation as the navigation references in
    // `scope::component_prop_navigation`.
    let tag_src_start = (template_offset + usage.start + 1) as usize;
    mappings.push(VizeMapping {
        gen_range: literal_range,
        src_range: tag_src_start..tag_src_start + usage.name.len(),
        sub_spans: Vec::new(),
    });

    if usage.vif_guard.is_some() {
        append!(*ts, "{indent}}}\n");
    }
}

/// Emit the inference-only binding a `v-slot` scope reads its payload type
/// from: `const <name> = (undefined as unknown as __VizeSlotsResolver<typeof
/// C>)(<the authored props>);`.
///
/// The child's slot map is a function of its generic parameters, and the only
/// construct that instantiates those the way `vue-tsc` does is a *call* with
/// the authored props — a type-level probe against the construct signature
/// erases every parameter to its constraint even when the props determine it
/// (verified against `vue-tsc` 3.3.4 / TypeScript 6.0.3).
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
    template_prop_names: &FxHashSet<String>,
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
        "{indent}const {binding_name} = (undefined as unknown as __VizeSlotsResolver<typeof {component_ref}>)(",
    );
    if let Some(ref guard) = usage.vif_guard {
        append!(*ts, "({guard}) ? ");
    }
    append_props_literal(
        ts,
        &mut discarded,
        usage,
        template_prop_names,
        source_context,
        indent,
    );
    if usage.vif_guard.is_some() {
        ts.push_str(" : (undefined as unknown as never)");
    }
    ts.push_str(");\n");
    append!(*ts, "{indent}void {binding_name};\n");
}
