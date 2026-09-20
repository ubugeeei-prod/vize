use vize_carton::String;
use vize_croquis::{Croquis, Scope, ScopeData, analysis::ComponentUsage};

/// The resolver lives at module scope so the same host type is used for
/// named and dynamic slot scopes, including functional components.
const SLOT_RESOLVER_HELPERS: &str = r#"type __VizeSlotHostSlots<C> = C extends { readonly __vizeSlots?: infer __S } ? NonNullable<__S> : C extends { new (): { $slots: infer __S } } ? __S : C extends (props: any, context: infer __Ctx, ...args: any[]) => any ? NonNullable<__Ctx> extends { slots: infer __S } ? __S : any : any;
declare function __vizeConstructorSlots<P, S>(component: new (props: P, ...args: any[]) => { $slots: S }): (props: P) => S;
declare function __vizeFunctionalSlots<P, C, R>(component: (props: P, context: C, ...args: any[]) => R): (props: P) => "__ctx" extends keyof R ? NonNullable<R["__ctx"]> extends { slots: infer S } ? S : any : NonNullable<C> extends { slots: infer S } ? S : any;
type __VizeNativeSlotsFactory<C> = C extends new (...args: any[]) => any ? typeof __vizeConstructorSlots : C extends (...args: any[]) => any ? typeof __vizeFunctionalSlots : (component: C) => (props: any) => __VizeSlotHostSlots<C>;
type __VizeSlotsFactory<C> = __VizeIsAny<C> extends true ? (component: C) => (props: any) => any : C extends { __vizeResolveSlots?: infer F } ? F extends (...args: any[]) => any ? (component: C) => F : __VizeNativeSlotsFactory<C> : __VizeNativeSlotsFactory<C>;
"#;

/// Only names admitted by the authored key contribute payloads. A broad string
/// can select any string slot; a literal union cannot acquire unrelated slots.
/// Optional slot provisioning must not add `undefined` to the payload itself.
const SLOT_PAYLOAD_HELPER: &str = "type __VizeSlotPayload<__S, __N> = __VizeIsAny<__S> extends true ? any : { [__K in keyof __S & __N]-?: NonNullable<__S[__K]> extends (props: infer __P, ...args: any[]) => any ? __P : never }[keyof __S & __N] extends infer __P ? ([__P] extends [never] ? any : __P) : any;\n";

/// Emit helpers only when a component owns an authored slot scope. Keeping
/// unused aliases out also preserves `noUnusedLocals` declaration consumers.
pub(crate) fn emit_slot_payload_helpers(
    ts: &mut String,
    summary: &Croquis,
    embedded_preamble: bool,
) {
    if embedded_preamble {
        ts.push('\n');
    }
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
