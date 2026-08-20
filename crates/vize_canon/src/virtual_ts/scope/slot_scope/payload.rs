use vize_carton::String;
use vize_croquis::{Croquis, Scope, ScopeData, analysis::ComponentUsage};

/// The slot-payload aliases, emitted per file rather than hoisted into the
/// shared preamble.
///
/// They deliberately stay out of `VUE_TYPE_HELPERS`: every other alias there is
/// transitively referenced by the always-emitted default-export types, while
/// these three are roots that only a resolvable `v-slot` scope reaches. A
/// module-scope copy in a component that has no such scope is dead code, which
/// a `noUnusedLocals` consumer reports as `TS6196` — the same reason
/// `__VizeWidenTemplateRef` and `__EmitProps` are emitted conditionally. A
/// component without a resolvable `v-slot` scope therefore gains nothing.
const SLOT_RESOLVER_HELPERS: &str = "type __VizeStructuralSlots<C> = C extends { readonly __vizeSlots?: infer __S } ? NonNullable<__S> : C extends { new (): { $slots: infer __S } } ? __S : any;\ntype __VizeSlotsResolver<C> = C extends { __vizeResolveSlots?: infer __F } ? (__F extends (...args: any[]) => any ? __F : (props: any) => __VizeStructuralSlots<C>) : C extends { readonly __vizeSlots?: any } ? (props: any) => __VizeStructuralSlots<C> : __VizeIsAny<C> extends true ? (props: any) => any : (props: any) => __VizeStructuralSlots<C>;\n";

/// Payload of one statically named slot.
const STATIC_SLOT_PAYLOAD_HELPER: &str = "type __VizeSlotPayload<__S, __K extends PropertyKey> = __K extends keyof __S ? (NonNullable<__S[__K]> extends (props: infer __P, ...args: any[]) => any ? __P : any) : any;\n";

/// Union of every declared payload, for `v-slot:[name]`.
/// `-?` removes the marker's optional provisioning modifier from the mapped
/// result; otherwise indexing `Partial<Slots>` adds `undefined` to the payload.
const DYNAMIC_SLOT_PAYLOAD_HELPER: &str = "type __VizeAnySlotPayload<__S> = { [__K in keyof __S]-?: NonNullable<__S[__K]> extends (props: infer __P, ...args: any[]) => any ? __P : never }[keyof __S] extends infer __P ? ([__P] extends [never] ? any : __P) : any;\n";

/// Whether any `v-slot` scope in this document resolves a host component, split
/// by how the slot is named — the static and dynamic payload aliases have
/// disjoint call sites, so a document that only has one kind must only declare
/// that one.
fn slot_helper_usage(summary: &Croquis) -> (bool, bool) {
    let mut used = (false, false);
    for scope in summary.scopes.iter() {
        let Some(data) = (match scope.data() {
            ScopeData::VSlot(data) => Some(data),
            _ => None,
        }) else {
            continue;
        };
        let Some(component) = data.component.as_deref() else {
            continue;
        };
        if find_slot_host(summary, scope, component).is_none() {
            continue;
        }
        if summary.scopes.is_v_slot_name_static(scope.id) {
            used.0 = true;
        } else {
            used.1 = true;
        }
    }
    used
}

/// Emit the per-file slot-payload aliases this document actually references,
/// plus the blank line that closed the embedded preamble before them.
pub(crate) fn emit_slot_payload_helpers(
    ts: &mut String,
    summary: &Croquis,
    embedded_preamble: bool,
) {
    if embedded_preamble {
        ts.push('\n');
    }
    let (static_names, dynamic_names) = slot_helper_usage(summary);
    if !static_names && !dynamic_names {
        return;
    }
    ts.push_str(SLOT_RESOLVER_HELPERS);
    if static_names {
        ts.push_str(STATIC_SLOT_PAYLOAD_HELPER);
    }
    if dynamic_names {
        ts.push_str(DYNAMIC_SLOT_PAYLOAD_HELPER);
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
pub(super) fn find_slot_host<'a>(
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
