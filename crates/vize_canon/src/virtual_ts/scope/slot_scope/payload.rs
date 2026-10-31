use vize_carton::String;
use vize_croquis::{Croquis, Scope, ScopeData, analysis::ComponentUsage};

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
pub(crate) fn emit_slot_payload_helpers(ts: &mut String, summary: &Croquis, emit_per_file: bool) {
    if !emit_per_file {
        return;
    }
    ts.push('\n');
    if summary
        .scopes
        .iter()
        .any(|scope| matches!(scope.data(), ScopeData::VSlot(data) if data.component.is_some()))
    {
        ts.push_str(SLOT_RESOLVER_HELPERS);
        ts.push_str(SLOT_PAYLOAD_HELPER);
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
    vize_croquis::facts::component_usage_list(summary)
        .into_iter()
        .filter(|usage| usage.name.as_str() == component)
        .find(|usage| {
            usage
                .slots
                .iter()
                .any(|slot| slot.start == directive_offset)
        })
}
