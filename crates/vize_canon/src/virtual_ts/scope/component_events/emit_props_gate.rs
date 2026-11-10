//! Props-independent inference-only emit-props calls.
//!
//! `$EmitPropsFactory` resolves each component member through a generic
//! resolver (a `__vizeResolveEmitProps`/`__vizeResolveProps` function, or a
//! vue-tsc library export) or through a route whose result is fixed by the
//! component alone: `__EmitProps<E>`, with `E` inferred from the component
//! argument, or `{}`. Generic resolvers and unmarked constructors can retain
//! type parameters until the props call. Those routes keep the original factory.
//! For the remaining components the second call's result is the factory's return type, so
//! `__vize_emit_input` checks the literal against `any`. The authored literal is
//! still emitted because the type level, not the generator, picks the route.
//!
//! The result is equal only where the component type is closed: a deferred
//! conditional would make the call read both branches. Generic SFCs and
//! `@vue-generic` bindings keep the direct call. So do literals that span
//! lines, whose later lines no `@ts-ignore` covers, and spread literals.
//!
//! Static tags and `<component :is>` targets both take the gate: a direct call
//! instantiates the factory's props input once per use, which the cheap call
//! skips. Under `v-if`, the factory is called before the guard, so a guard that
//! names the component's binding (and could narrow it) keeps the direct call.

use vize_carton::append;
use vize_croquis::{Croquis, ScopeData};

/// `{emit_props_factory}` is the shared `$EmitPropsFactory` reference and
/// `{emit_props}` the shared `__EmitProps`.
/// `__vize_emit_input` mirrors that factory's branches: `true` marks a route
/// that reads the props, `false` one that does not. A union call combines its
/// members' returns with subtype reduction, which no type alias can spell, so
/// the cheap call also requires every member to return the identical type.
///
/// `__vize_emit_closed` selects the commonest route, a single non-generic Vize
/// SFC, before the factory is built. That route is the factory's marked
/// constructor member, `(component: unknown) => <A>(props: $EmitInput<…>) =>
/// __EmitProps<$UnionEmit<C>>`, and its return never reads `A`. For one
/// member (identical to the whole type, so no union), `$UnionEmit<C>` is the
/// `$emit` that `__vize_emit_member` infers, and `(component: unknown) =>
/// (props: any) => __EmitProps<E>` is the same factory minus its props input.
/// `__vize_emit_input` then returns that call unchanged: its own branches would
/// make the same `(props: any) => R` of it, and the full factory's single
/// signature returns the same `R` where they would not. Unions, `any`,
/// `never`, generic resolvers, unmarked constructors and an `$emit` of `never`
/// keep the full factory and the full gate.
const GATE_ALIASES: &str = "type __vize_emit_native_input<C> = C extends new (...args: any[]) => any ? C extends { readonly __vizeComponentMarker: true } ? false : true : false;\ntype __vize_emit_member<C> = C extends new (...args: any[]) => { $emit: infer E } ? E : never;\ntype __vize_emit_closed<C, A = C> = [C] extends [never] ? false : 0 extends (1 & C) ? false : [C extends { readonly __vizeComponentMarker: true } ? (<G>() => G extends C ? 1 : 2) extends <G>() => G extends A ? 1 : 2 ? C extends { __vizeResolveEmitProps?: infer X } ? X extends (...args: any[]) => any ? false : true : C extends { __vizeResolveProps?: infer X } ? X extends (...args: any[]) => any ? false : true : true : false : false] extends [true] ? [__vize_emit_member<C>] extends [never] ? false : true : false;\ntype __vize_emit_resolver<L, C> = [L] extends [true] ? (component: C) => C : __vize_emit_closed<C> extends true ? (component: unknown) => (props: any) => {emit_props}<__vize_emit_member<C>> : {emit_props_factory}<C>;\ntype __vize_emit_input<L, C, F, R = F extends (...args: any) => infer T ? T : any, U = R> = [L] extends [true] ? F : __vize_emit_closed<C> extends true ? F : true extends (0 extends (1 & C) ? false : C extends { __vizeResolveEmitProps?: infer X } ? X extends (...args: any[]) => any ? true : __vize_emit_native_input<C> : C extends { __vizeResolveProps?: infer X } ? X extends (...args: any[]) => any ? true : __vize_emit_native_input<C> : __vize_emit_native_input<C>) ? F : [R] extends [never] ? F : [R extends unknown ? (<G>() => G extends R ? 1 : 2) extends <G>() => G extends U ? 1 : 2 ? true : false : false] extends [true] ? (props: any) => R : F;\n";

/// Declares the gate aliases once per module that can reach a gated call.
pub(crate) fn emit_gate_aliases(
    ts: &mut crate::virtual_ts::TextBuf,
    summary: &Croquis,
    generic_component: bool,
    legacy_vue2: bool,
    hoist_shared_preamble: bool,
) {
    if !gates_inference(generic_component, legacy_vue2) || !has_gated_candidate(summary) {
        return;
    }
    let emit_props_factory =
        crate::virtual_ts::helpers::shared_types::EMIT_PROPS_FACTORY.select(hoist_shared_preamble);
    let emit_props =
        crate::virtual_ts::helpers::shared_types::EMIT_PROPS.select(hoist_shared_preamble);
    ts.push_str(
        &GATE_ALIASES
            .replace("{emit_props_factory}", emit_props_factory)
            .replace("{emit_props}", emit_props),
    );
}

/// The template's component types are closed and Vue 3 owns the factory.
pub(crate) const fn gates_inference(generic_component: bool, legacy_vue2: bool) -> bool {
    !generic_component && !legacy_vue2
}

/// A usage whose listener can reach the gated call: it has a static event, at
/// least one valued static prop, and no spread props.
fn has_gated_candidate(summary: &Croquis) -> bool {
    let has_component_listener = summary.scopes.iter().any(
        |scope| matches!(scope.data(), ScopeData::EventHandler(data) if data.target_component.is_some()),
    );
    has_component_listener
        && vize_croquis::facts::component_usage_slice(summary)
            .iter()
            .any(|usage| {
                usage.spread_props.is_empty()
                    && usage.events.iter().any(|event| !event.name_is_dynamic)
                    && usage.props.iter().any(|prop| {
                        !prop.name_is_dynamic
                            && prop.name.as_str() != "key"
                            && prop.name.as_str() != "ref"
                            && prop.value.is_some()
                    })
            })
}

/// TypeScript's line terminators: a value with one is not covered by the
/// `@ts-ignore` above its entry.
pub(super) fn is_single_line(value: &str) -> bool {
    !value.contains(['\n', '\r', '\u{2028}', '\u{2029}'])
}

/// The gated call: the first call infers the component's own factory, and the
/// second reads the props only through a generic resolver.
pub(super) fn append_gated_call(
    ts: &mut crate::virtual_ts::TextBuf,
    indent: &str,
    library: &str,
    component_ref: &str,
    emit_props: &str,
    usage_idx: usize,
) {
    append!(
        *ts,
        "{indent}const __vize_emit_factory_{usage_idx} = (undefined as unknown as __vize_emit_resolver<{library}, typeof {component_ref}>)({component_ref}); const {emit_props} = (undefined as unknown as __vize_emit_input<{library}, typeof {component_ref}, typeof __vize_emit_factory_{usage_idx}>)(",
    );
}

/// A `v-if` use calls the factory before its guard, under its own ignore, as
/// the direct call's factory sits on the ignored guard line.
pub(super) fn append_guarded_factory(
    ts: &mut crate::virtual_ts::TextBuf,
    indent: &str,
    library: &str,
    component_ref: &str,
    usage_idx: usize,
) {
    append!(
        *ts,
        "{indent}// @ts-ignore Inference-only factory; called before the guard below.\n{indent}const __vize_emit_factory_{usage_idx} = (undefined as unknown as __vize_emit_resolver<{library}, typeof {component_ref}>)({component_ref});\n",
    );
}

/// The guarded call reads the props only through a generic resolver.
pub(super) fn append_guarded_input(
    ts: &mut crate::virtual_ts::TextBuf,
    indent: &str,
    guard: &str,
    library: &str,
    component_ref: &str,
    usage_idx: usize,
) {
    append!(
        *ts,
        "{indent}if ({guard}) return (undefined as unknown as __vize_emit_input<{library}, typeof {component_ref}, typeof __vize_emit_factory_{usage_idx}>)(",
    );
}

/// Whether `guard` names the root binding of `component_ref`, so it may narrow
/// the component's type inside the guarded branch.
pub(super) fn guard_reads(guard: &str, component_ref: &str) -> bool {
    let is_ident = |c: char| c == '_' || c == '$' || c.is_alphanumeric();
    let root = component_ref
        .split(|c: char| !is_ident(c))
        .next()
        .unwrap_or_default();
    !root.is_empty()
        && guard.match_indices(root).any(|(start, _)| {
            let end = start + root.len();
            let before = guard.get(..start).unwrap_or_default();
            let after = guard.get(end..).unwrap_or_default();
            !before.ends_with(is_ident) && !after.starts_with(is_ident)
        })
}

#[cfg(test)]
mod tests;
