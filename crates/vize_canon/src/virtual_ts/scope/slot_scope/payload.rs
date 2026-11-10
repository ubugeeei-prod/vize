use vize_croquis::{Croquis, Scope, ScopeData, analysis::ComponentUsage};

use super::{SlotPayloadContext, SlotTypes};
use crate::virtual_ts::expressions::{constructor_instance, generate_slot_constructor_capture};

/// The resolver lives at module scope so the same host type is used for
/// named and dynamic slot scopes, including functional components.
/// An unknown host must retain a regular payload fallback: inventing `any`
/// here would select the silent error type and hide authored diagnostics.
const SLOT_RESOLVER_HELPERS: &str =
    include_str!("../../helpers/declarations/slot-resolver.d.ts.txt");

/// Only names admitted by the authored key contribute payloads. A literal union
/// cannot acquire unrelated slots. Optional slot provisioning must not add
/// `undefined` to the payload itself.
///
/// A broad `string` key on a host without a string index signature leaves the
/// payload unchecked, as `vue-tsc` does. A forwarding wrapper
/// (`v-for="(_, name) in $slots"` with `#[name]="props"`) would otherwise hand
/// every consumer the union of all host payloads and reject each named slot.
///
/// A declared `any` provider retains regular `any`, so authored conditional
/// type witnesses still report errors. A compiler error-any provider retains
/// its own intrinsic identity instead of receiving a fabricated replacement.
/// Missing registry providers keep their framework-compatible silent fallback.
const SLOT_PAYLOAD_HELPER: &str = include_str!("../../helpers/declarations/slot-payload.d.ts.txt");

/// Emit helpers only when a component owns an authored slot scope. Keeping
/// unused aliases out also preserves `noUnusedLocals` declaration consumers.
pub(crate) fn emit_slot_payload_helpers(
    ts: &mut crate::virtual_ts::TextBuf,
    summary: &Croquis,
    emit_per_file: bool,
) {
    if !emit_per_file {
        return;
    }
    ts.push('\n');
    if summary.scopes.iter().any(|scope| match scope.data() {
        ScopeData::VSlot(data) => data.component.as_deref().is_some_and(|component| {
            data.props_pattern.is_some() || captures_root_constructor(summary, scope, component)
        }),
        _ => false,
    }) {
        ts.push_lib_text(SLOT_RESOLVER_HELPERS);
        ts.push_lib_text(SLOT_PAYLOAD_HELPER);
    }
}

/// Whether this unbound slot still writes its usage's constructor-instance
/// pair, which the usage's props type reads.
fn captures_root_constructor(summary: &Croquis, scope: &Scope, component: &str) -> bool {
    find_slot_host(summary, scope, component)
        .is_some_and(|usage| is_root_constructor_slot(summary, scope, &usage))
}

/// Whether `scope` is the slot whose host writes `usage`'s constructor-instance
/// pair: the one the usage's props type reads.
fn is_root_constructor_slot(summary: &Croquis, scope: &Scope, usage: &ComponentUsage) -> bool {
    constructor_instance::root_slot_scope(&summary.scopes, usage, false)
        .is_some_and(|root| root.id == scope.id)
}

/// The payload of an unbound slot (`<template #label>`): nothing. Its
/// parameter is the synthetic `__vize_slot_props_N`, which nothing reads, so
/// the host inference over the child's constructor and slots factory would only
/// cost time. The props pass already skips it (#11432). The one statement
/// another check reads is the root usage's constructor-instance pair, which its
/// props type takes the actual constructor result from; keep exactly that.
pub(super) fn unread_slot_types(
    ts: &mut crate::virtual_ts::TextBuf,
    ctx: &SlotPayloadContext<'_>,
    scope: &Scope,
    host: Option<&ComponentUsage>,
    binding: &str,
    component_ref: &str,
) -> SlotTypes {
    if let Some(usage) = host
        && is_root_constructor_slot(ctx.summary, scope, usage)
    {
        generate_slot_constructor_capture(
            ts,
            usage,
            binding,
            component_ref,
            ctx.template_binding_access,
            ctx.indent,
            (ctx.source_context, ctx.hoist_shared_preamble),
        );
    }
    SlotTypes {
        payload: "unknown".into(),
        member: "unknown".into(),
    }
}

/// The component usage that hosts this `v-slot` scope.
///
/// A template can mount the same child many times, so the tag name alone does
/// not identify the usage whose props instantiate this slot. Both the scope and
/// the usage's [`vize_croquis::croquis::SlotUsage`] record the offset of the
/// authored `v-slot` / `#name` directive, so that offset links the two exactly
/// — no containment heuristic, and no ambiguity between nested usages of the
/// same tag.
pub(super) fn find_slot_host(
    summary: &Croquis,
    scope: &Scope,
    component: &str,
) -> Option<ComponentUsage> {
    let directive_offset = scope.span.start;
    vize_croquis::facts::component_usage_slice(summary)
        .iter()
        .filter(|usage| usage.name.as_str() == component)
        .find(|usage| {
            usage
                .slots
                .iter()
                .any(|slot| slot.start == directive_offset)
        })
        .cloned()
}
