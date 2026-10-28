use vize_carton::String;
use vize_croquis::{Croquis, Scope, ScopeData, analysis::ComponentUsage};

/// The resolver lives at module scope so the same host type is used for
/// named and dynamic slot scopes, including functional components.
/// An unknown host must retain a regular payload fallback: inventing `any`
/// here would select the silent error type and hide authored diagnostics.
const SLOT_RESOLVER_HELPERS: &str = r#"type $$SlotHostSlots<C> = C extends { readonly __vizeSlots?: infer __S } ? NonNullable<__S> : C extends { new (): { $slots: infer __S } } ? __S : C extends (props: any, context: infer __Ctx, ...args: any[]) => any ? NonNullable<__Ctx> extends { slots: infer __S } ? __S : any : unknown;
declare function __vizeFunctionalSlots<P, C, R>(component: (props: P, context: C, ...args: any[]) => R): (props: P) => "__ctx" extends keyof R ? NonNullable<R["__ctx"]> extends { slots: infer S } ? S : any : NonNullable<C> extends { slots: infer S } ? S : any;
type $$NativeSlotsFactory<C> = C extends new (...args: any[]) => any ? (component: C) => (props: any) => $$SlotHostSlots<C> : C extends (...args: any[]) => any ? typeof __vizeFunctionalSlots : (component: C) => (props: any) => $$SlotHostSlots<C>;
type $$SlotsFactory<C> = $$IsAny<C> extends true ? (component: C) => (props: any) => any : C extends { __vizeResolveSlots?: infer F } ? F extends (...args: any[]) => any ? (component: C) => F : $$NativeSlotsFactory<C> : C extends { readonly __vizeComponentMarker: true } ? (component: C) => (props: any) => $$SlotHostSlots<C> : $$NativeSlotsFactory<C>;
type $$HasSlotResolver<C> = C extends { __vizeResolveSlots?: infer F } ? F extends (...args: any[]) => any ? true : false : false;
type $$DirectSlotConstructor<C> = $$IsAny<C> extends true ? never : $$HasSlotResolver<C> extends true ? never : C extends { readonly __vizeComponentMarker: true } ? never : C extends new (...args: any[]) => { $slots: any } ? C : never;
type $$SlotConstructor<C> = [$$DirectSlotConstructor<C>] extends [never] ? new (props: any, ...args: any[]) => { $slots: never } : $$DirectSlotConstructor<C>;
type $$SlotConstructorArgs<C> = C extends new (props: any, ...args: infer A) => any ? { [K in keyof A]: any } : [];
type $$SlotHostSelector<C> = [$$DirectSlotConstructor<C>] extends [never] ? <S>(instance: unknown, slots: S) => S : <I extends { $slots: any }>(instance: I, slots: unknown) => I["$slots"];
"#;

/// Only names admitted by the authored key contribute payloads. A literal union
/// cannot acquire unrelated slots. Optional slot provisioning must not add
/// `undefined` to the payload itself.
///
/// A broad `string` key on a host without a string index signature leaves the
/// payload unchecked, as `vue-tsc` does. A forwarding wrapper
/// (`v-for="(_, name) in $slots"` with `#[name]="props"`) would otherwise hand
/// every consumer the union of all host payloads and reject each named slot.
///
/// An untyped host (an unresolved tag, an `any` component) yields an *error
/// type* rather than a declared `any`. Both leave the payload unchecked, but
/// TypeScript never reports on values derived from an error type, where a
/// declared `any` still fails a `never` parameter. `vue-tsc` resolves an
/// unknown tag through a missing registry key, so its payloads are silent in
/// exactly this way. The component itself stays `any`: listener and prop
/// checks rely on conditional types an error type would collapse.
const SLOT_PAYLOAD_HELPER: &str = "// @ts-ignore The unchecked payload of an untyped host: an error type, never reported.\ntype $$SilentAny = {}[\"__vizeSilentAny\"];\ntype $$NamedSlotPayload<__S, __N> = { [__K in keyof __S & __N]-?: NonNullable<__S[__K]> extends (props: infer __P, ...args: any[]) => any ? __P : never }[keyof __S & __N] extends infer __P ? ([__P] extends [never] ? any : __P) : any;\ntype $$SlotPayload<__S, __N> = $$IsAny<__S> extends true ? $$SilentAny : string extends __N ? string extends keyof __S ? $$NamedSlotPayload<__S, __N> : any : $$NamedSlotPayload<__S, __N>;\n";

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
