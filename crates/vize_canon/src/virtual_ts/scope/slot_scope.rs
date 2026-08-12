//! `v-slot` scope emission and the slot-payload type the scope's parameter is
//! annotated with.
//!
//! A scoped slot's payload is a function of the child's *instantiated* generic
//! parameters, so it can only be derived from a call that passes the authored
//! props. `vue-tsc` calls the child's functional component with the props
//! object and reads `slots` off the result; probing the child's construct
//! signature at the type level instead erases every type parameter to its
//! constraint even when the props determine it. This module emits the
//! equivalent call — see [`crate::virtual_ts::expressions::generate_slot_host_binding`]
//! — and annotates the slot function from its result.

use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::append;
use vize_carton::cstr;
use vize_carton::profile;
use vize_croquis::{Croquis, Scope, VSlotScopeData, analysis::ComponentUsage};

use crate::virtual_ts::expressions::{
    ComponentPropSource, ExpressionListEmitContext, generate_expressions,
    generate_slot_host_binding,
};
use crate::virtual_ts::helpers::to_safe_identifier_fragment;
use crate::virtual_ts::types::{VirtualTsOptions, VizeMapping};

use super::children::generate_child_scopes;
use super::context::{ScopeGenContext, VForPropsContext};
use super::emit::{component_binding_reference, emit_slot_function_open, slot_props_type};

/// The component usage that hosts this `v-slot` scope.
///
/// A template can mount the same child many times, so the tag name alone does
/// not identify the usage whose props instantiate this slot. Both the scope and
/// the usage's [`vize_croquis::croquis::SlotUsage`] record the offset of the
/// authored `v-slot` / `#name` directive, so that offset links the two exactly
/// — no containment heuristic, and no ambiguity between nested usages of the
/// same tag.
fn find_slot_host<'a>(
    summary: &'a Croquis,
    scope: &Scope,
    component: &str,
) -> Option<&'a ComponentUsage> {
    let directive_offset = scope.span.start;
    summary
        .component_usages
        .iter()
        .filter(|usage| usage.name.as_str() == component)
        .find(|usage| {
            usage
                .slots
                .iter()
                .any(|slot| slot.start == directive_offset)
        })
}

/// Everything the payload type needs that differs between the two emitters.
struct SlotPayloadContext<'a> {
    summary: &'a Croquis,
    options: &'a VirtualTsOptions,
    template_prop_names: &'a FxHashSet<String>,
    source_context: ComponentPropSource<'a>,
    /// Name prefix of the emitted host binding. The two emitters share a block
    /// scope, so each needs its own or the second would redeclare the first.
    binding_prefix: &'a str,
    indent: &'a str,
}

/// Emit the host binding when the slot sits on a resolvable child, and return
/// the slot-payload type its scope parameter is annotated with.
fn slot_payload_type(
    ts: &mut String,
    ctx: &SlotPayloadContext<'_>,
    scope: &Scope,
    data: &VSlotScopeData,
) -> String {
    let summary = ctx.summary;
    let name_is_static = summary.scopes.is_v_slot_name_static(scope.id);
    let Some(component) = data.component.as_deref() else {
        return slot_props_type(
            summary,
            ctx.options,
            None,
            data.name.as_str(),
            name_is_static,
        );
    };
    let Some(usage) = find_slot_host(summary, scope, component) else {
        return slot_props_type(
            summary,
            ctx.options,
            Some(component),
            data.name.as_str(),
            name_is_static,
        );
    };
    let component_ref = component_binding_reference(summary, ctx.options, component);
    let binding = cstr!("{}{}", ctx.binding_prefix, scope.id.as_u32());
    generate_slot_host_binding(
        ts,
        usage,
        binding.as_str(),
        component_ref.as_str(),
        ctx.template_prop_names,
        ctx.source_context,
        ctx.indent,
    );
    if name_is_static {
        cstr!(
            "__VizeSlotPayload<typeof {binding}, \"{}\">",
            data.name.as_str()
        )
    } else {
        cstr!("__VizeAnySlotPayload<typeof {binding}>")
    }
}

/// Generate a `v-slot` scope's closure: the host binding, the slot function,
/// the expressions authored inside it and its nested scopes.
pub(super) fn generate_v_slot_scope(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &ScopeGenContext<'_>,
    scope: &Scope,
    data: &VSlotScopeData,
    indent: &str,
    inner_indent: &str,
) {
    let scope_id = scope.id.as_u32();
    append!(*ts, "\n{indent}// v-slot scope: #{}\n", data.name);
    let props_pattern = data.props_pattern.as_deref().unwrap_or("slotProps");
    let safe_slot_name = to_safe_identifier_fragment(data.name.as_str());
    let props_type = slot_payload_type(
        ts,
        &SlotPayloadContext {
            summary: ctx.summary,
            options: ctx.virtual_ts_options,
            template_prop_names: ctx.template_prop_names,
            source_context: ComponentPropSource::new(
                ctx.template_source,
                ctx.template_offset,
                &ctx.summary.scopes,
            ),
            binding_prefix: "__vize_slot_host_",
            indent,
        },
        scope,
        data,
    );
    emit_slot_function_open(
        ts,
        indent,
        cstr!("_slot_{safe_slot_name}_{scope_id}").as_str(),
        props_pattern,
        &props_type,
    );
    if data.prop_names.is_empty() {
        // Simple identifier (no destructuring)
        append!(*ts, "{inner_indent}void {props_pattern};\n");
    } else {
        // Destructured: void each extracted prop name
        for prop_name in data.prop_names.iter() {
            append!(*ts, "{inner_indent}void {prop_name};\n");
        }
    }

    if let Some(exprs) = ctx.expressions_by_scope.get(&scope_id)
        && ctx.check_options.check_template_bindings
    {
        generate_expressions(
            ts,
            mappings,
            exprs,
            ctx.template_prop_names,
            &ExpressionListEmitContext::new(
                ctx.skipped_expression_ranges,
                ctx.template_offset,
                inner_indent,
                ctx.checks,
            ),
        );
    }

    // Recursively generate child scopes inside this closure
    profile!(
        "canon.virtual_ts.child_scopes",
        generate_child_scopes(ts, mappings, ctx, scope_id, inner_indent)
    );

    ts.push_str(indent);
    ts.push_str("};\n");
}

/// Generate the component prop checks that live inside a `v-slot` scope.
pub(super) fn generate_v_slot_props_scope(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &VForPropsContext<'_>,
    scope: &Scope,
    data: &VSlotScopeData,
    indent: &str,
    inner_indent: &str,
) {
    let scope_id = scope.id.as_u32();
    let props_pattern = data.props_pattern.as_deref().unwrap_or("slotProps");
    let safe_slot_name = to_safe_identifier_fragment(data.name.as_str());
    append!(
        *ts,
        "\n{indent}// Component props in v-slot scope: #{}\n",
        data.name
    );
    let props_type = slot_payload_type(
        ts,
        &SlotPayloadContext {
            summary: ctx.summary,
            options: ctx.options,
            template_prop_names: ctx.template_prop_names,
            source_context: ctx.source_context,
            binding_prefix: "__vize_slot_props_host_",
            indent,
        },
        scope,
        data,
    );
    emit_slot_function_open(
        ts,
        indent,
        cstr!("_slot_props_{safe_slot_name}_{scope_id}").as_str(),
        props_pattern,
        &props_type,
    );
    // Mark slot prop variables as used
    if data.prop_names.is_empty() {
        append!(*ts, "{inner_indent}void {props_pattern};\n");
    } else {
        for prop_name in data.prop_names.iter() {
            append!(*ts, "{inner_indent}void {prop_name};\n");
        }
    }
    super::empty_component_props::generate_scope_checks(ts, mappings, ctx, scope_id, inner_indent);
    super::component_props::recurse_child_closure_scopes(ts, mappings, ctx, scope_id, inner_indent);

    ts.push_str(indent);
    ts.push_str("};\n");
}
