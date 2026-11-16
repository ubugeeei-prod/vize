//! One generic alias per module for every listener's inferred emit arguments.
//!
//! Each listener used to spell its own conditional over `typeof
//! __vize_emit_props_N`: a fresh conditional root and fresh
//! `{ onX?: (...args: infer __A) => any }` literals per use. The alias is
//! declared once in the template function instead, so TypeScript creates
//! those types once per module and reuses an instantiation for equal
//! arguments. `NoInfer` keeps each check non-distributive, exactly like the
//! inline `typeof … extends …` it replaces, including its `any`, `never` and
//! union cases. The alias stays per module on purpose: one SDK declaration
//! shared by every module would let a result computed under one module's
//! circular resolution answer the others.

use std::cell::Cell;

use vize_carton::{String, cstr};

/// The alias every inferred-arguments use names.
pub(super) const ALIAS: &str = "__vizeInferredEmitArgs";

thread_local! {
    static NEEDED: Cell<bool> = const { Cell::new(false) };
}

pub(crate) fn reset() {
    NEEDED.with(|needed| needed.set(false));
}

/// A use in this template function needs the declaration.
pub(super) fn require() {
    NEEDED.with(|needed| needed.set(true));
}

/// Declare the alias once in the template function when any use named it.
pub(crate) fn flush(ts: &mut crate::virtual_ts::TextBuf) {
    if NEEDED.with(|needed| needed.replace(false)) {
        ts.push_str(declaration().as_str());
    }
}

/// `key` is the listener prop's object-type key: an identifier or a quoted
/// string literal.
pub(super) fn reference(library: &str, emit_props: &str, key: &str, loose: bool) -> String {
    require();
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
    use super::{flush, reference, reset};

    #[test]
    fn uses_name_the_alias_and_one_flush_declares_it() {
        reset();
        assert_eq!(
            reference(
                "__Child_3__library",
                "__vize_emit_props_0",
                "onSelect",
                false
            )
            .as_str(),
            "__vizeInferredEmitArgs<__Child_3__library, typeof __vize_emit_props_0, \"onSelect\">"
        );
        assert_eq!(
            reference(
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
    fn a_function_without_uses_declares_nothing() {
        reset();
        let mut ts = crate::virtual_ts::TextBuf::default();
        flush(&mut ts);
        assert_eq!(ts.as_str(), "");
    }
}
