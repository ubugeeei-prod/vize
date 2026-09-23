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
///
/// An untyped host (an unresolved tag, an `any` component) yields an *error
/// type* rather than a declared `any`. Both leave the payload unchecked, but
/// TypeScript never reports on values derived from an error type, where a
/// declared `any` still fails a `never` parameter. `vue-tsc` resolves an
/// unknown tag through a missing registry key, so its payloads are silent in
/// exactly this way. The component itself stays `any`: listener and prop
/// checks rely on conditional types an error type would collapse.
const SLOT_PAYLOAD_HELPER: &str = "// @ts-ignore The unchecked payload of an untyped host: an error type, never reported.\ntype __VizeSilentAny = {}[\"__vizeSilentAny\"];\ntype __VizeSlotPayload<__S, __N> = __VizeIsAny<__S> extends true ? __VizeSilentAny : { [__K in keyof __S & __N]-?: NonNullable<__S[__K]> extends (props: infer __P, ...args: any[]) => any ? __P : never }[keyof __S & __N] extends infer __P ? ([__P] extends [never] ? any : __P) : any;\n";

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
