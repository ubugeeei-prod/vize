//! One generic alias per module for every listener's inferred emit arguments.
//!
//! Each listener spells its own conditional over `typeof __vize_emit_props_N`:
//! a fresh conditional root and fresh `{ onX?: (...args: infer __A) => any }`
//! literals per use. With two or more listeners, the alias is declared once in
//! the template function instead, so TypeScript creates those types once per
//! module and reuses an instantiation for equal arguments. A single listener
//! keeps the inline conditional: the declaration would only add bytes.
//! `NoInfer` keeps each check non-distributive, exactly like the inline
//! `typeof … extends …` it replaces, including its `any`, `never` and union
//! cases. The alias stays per module on purpose: one SDK declaration shared by
//! every module would let a result computed under one module's circular
//! resolution answer the others.

use std::cell::Cell;

use vize_carton::{String, cstr};
use vize_croquis::{Croquis, Scope, ScopeData};

use crate::virtual_ts::types::VirtualTsCheckOptions;

/// The alias every inferred-arguments use names.
const ALIAS: &str = "__vizeInferredEmitArgs";

thread_local! {
    static SHARED: Cell<bool> = const { Cell::new(false) };
    static NEEDED: Cell<bool> = const { Cell::new(false) };
}

/// Start a template function whose listeners name `uses` inferred arguments.
pub(crate) fn reset(uses: usize) {
    SHARED.with(|shared| shared.set(uses >= 2));
    NEEDED.with(|needed| needed.set(false));
}

/// The listeners of this template function whose emit arguments are
/// inferred: component handlers the event scope types (emit checks on, a
/// static event name, not a vnode hook, not a slot outlet's), on a usage
/// that infers them.
pub(crate) fn uses(
    summary: &Croquis,
    template_ast: Option<&vize_relief::RootNode<'_>>,
    check_options: VirtualTsCheckOptions,
    slot_outlet_handler: impl Fn(&Scope) -> bool,
) -> usize {
    if !check_options.check_event_handlers() || !check_options.check_emits {
        return 0;
    }
    summary
        .scopes
        .iter()
        .filter(|scope| {
            let ScopeData::EventHandler(data) = scope.data() else {
                return false;
            };
            let Some(component) = data.target_component.as_ref() else {
                return false;
            };
            !data.event_name_is_dynamic
                && super::super::event_scope::event_targets::vnode_hook_signature(
                    data.event_name.as_str(),
                )
                .is_none()
                && !slot_outlet_handler(scope)
                && super::generic_inference::find_component_usage_for_event(
                    summary,
                    component.as_str(),
                    data,
                    scope,
                )
                .is_some_and(|(_, usage)| {
                    super::generic_inference::infers_emit_args(template_ast, &usage)
                })
        })
        .count()
}

/// Declare the alias once in the template function when any use named it.
pub(crate) fn flush(ts: &mut crate::virtual_ts::TextBuf) {
    if NEEDED.with(|needed| needed.replace(false)) {
        ts.push_str(declaration().as_str());
    }
}

/// The inferred arguments of one listener. `key` is the listener prop's
/// object-type key: an identifier or a quoted string literal.
pub(super) fn arguments(library: &str, emit_props: &str, key: &str, loose: bool) -> String {
    if !SHARED.with(Cell::get) {
        // A loose static emit has no declared runtime arity. Preserve the
        // normal listener-prop fallback; dynamic targets keep their marker.
        let loose_args = if loose { "__L" } else { "unknown[]" };
        let non_nullable = crate::virtual_ts::generated_names::lib("NonNullable");
        return cstr!(
            "{library} extends true ? (typeof {emit_props} extends {{ __ctx?: infer __X }} ? {non_nullable}<__X> extends {{ props: infer __P }} ? __P : {{}} : {{}}) extends {{ {key}?: (...args: infer __A) => any }} ? __A : unknown[] : typeof {emit_props} extends {{ readonly __vizeLooseListener: infer __L extends unknown[] }} ? {loose_args} : typeof {emit_props} extends {{ {key}?: (...args: infer __A) => any }} ? __A : unknown[]"
        );
    }
    NEEDED.with(|needed| needed.set(true));
    let loose = if loose { ", true" } else { "" };
    if key.starts_with('"') {
        cstr!("{ALIAS}<{library}, typeof {emit_props}, {key}{loose}>")
    } else {
        cstr!("{ALIAS}<{library}, typeof {emit_props}, \"{key}\"{loose}>")
    }
}

fn declaration() -> String {
    let no_infer = crate::virtual_ts::generated_names::lib("NoInfer");
    let non_nullable = crate::virtual_ts::generated_names::lib("NonNullable");
    let property_key = crate::virtual_ts::generated_names::lib("PropertyKey");
    cstr!(
        "  type {ALIAS}<L, EP, K extends {property_key}, Loose = false> = {no_infer}<L> extends true ? ({no_infer}<EP> extends {{ __ctx?: infer __X }} ? {non_nullable}<__X> extends {{ props: infer __P }} ? __P : {{}} : {{}}) extends {{ [Q in K]?: (...args: infer __A) => any }} ? __A : unknown[] : {no_infer}<EP> extends {{ readonly __vizeLooseListener: infer __L extends unknown[] }} ? ([Loose] extends [true] ? __L : unknown[]) : {no_infer}<EP> extends {{ [Q in K]?: (...args: infer __A) => any }} ? __A : unknown[];\n"
    )
}

#[cfg(test)]
mod tests {
    use super::{arguments, flush, reset};

    #[test]
    fn two_uses_name_the_alias_and_one_flush_declares_it() {
        reset(2);
        assert_eq!(
            arguments(
                "__Child_3__library",
                "__vize_emit_props_0",
                "onSelect",
                false
            )
            .as_str(),
            "__vizeInferredEmitArgs<__Child_3__library, typeof __vize_emit_props_0, \"onSelect\">"
        );
        assert_eq!(
            arguments(
                "__Child_3__library",
                "__vize_emit_props_0",
                "\"onUpdate:modelValue\"",
                true
            )
            .as_str(),
            "__vizeInferredEmitArgs<__Child_3__library, typeof __vize_emit_props_0, \"onUpdate:modelValue\", true>"
        );
        let mut ts = crate::virtual_ts::TextBuf::default();
        flush(&mut ts);
        assert_eq!(
            ts.as_str(),
            "  type __vizeInferredEmitArgs<L, EP, K extends PropertyKey, Loose = false> = NoInfer<L> extends true ? (NoInfer<EP> extends { __ctx?: infer __X } ? NonNullable<__X> extends { props: infer __P } ? __P : {} : {}) extends { [Q in K]?: (...args: infer __A) => any } ? __A : unknown[] : NoInfer<EP> extends { readonly __vizeLooseListener: infer __L extends unknown[] } ? ([Loose] extends [true] ? __L : unknown[]) : NoInfer<EP> extends { [Q in K]?: (...args: infer __A) => any } ? __A : unknown[];\n"
        );
        // Declared once per template function.
        let mut again = crate::virtual_ts::TextBuf::default();
        flush(&mut again);
        assert_eq!(again.as_str(), "");
    }

    #[test]
    fn a_single_use_keeps_the_inline_conditional() {
        reset(1);
        assert_eq!(
            arguments(
                "__Child_3__library",
                "__vize_emit_props_0",
                "onSelect",
                true
            )
            .as_str(),
            "__Child_3__library extends true ? (typeof __vize_emit_props_0 extends { __ctx?: infer __X } ? NonNullable<__X> extends { props: infer __P } ? __P : {} : {}) extends { onSelect?: (...args: infer __A) => any } ? __A : unknown[] : typeof __vize_emit_props_0 extends { readonly __vizeLooseListener: infer __L extends unknown[] } ? __L : typeof __vize_emit_props_0 extends { onSelect?: (...args: infer __A) => any } ? __A : unknown[]"
        );
        let mut ts = crate::virtual_ts::TextBuf::default();
        flush(&mut ts);
        assert_eq!(ts.as_str(), "");
    }
}
