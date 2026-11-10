//! The per-use boxes every listener of one component reads: its library flag,
//! props, emit props and fallthrough root. Under the shared listener contract
//! each box names a shared SDK helper keyed only on the component, so
//! TypeScript resolves a component's boxes once per program instead of once
//! per use and file. The alias names stay per use.

use super::native_fallthrough;
use crate::virtual_ts::helpers::shared_types::{
    COMPONENT_EMIT_PROPS_BOX, COMPONENT_LIBRARY_FLAG, COMPONENT_NATIVE_FALLTHROUGH_BOX,
    COMPONENT_PROPS_BOX,
};

pub(super) struct Boxes<'a> {
    pub(super) library: &'a str,
    pub(super) props: &'a str,
    pub(super) emit_props: &'a str,
    pub(super) fallthrough: &'a str,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Fallthrough {
    None,
    Native,
    Contextual,
}

pub(super) fn emit(
    ts: &mut crate::virtual_ts::TextBuf,
    indent: &str,
    boxes: &Boxes<'_>,
    component_ref: &str,
    fallthrough: Fallthrough,
    is_any: &str,
    shared: bool,
) {
    let Boxes {
        library,
        props: props_box,
        emit_props: emit_props_box,
        fallthrough: fallthrough_box,
    } = *boxes;
    if shared && fallthrough != Fallthrough::Contextual {
        for (alias, helper) in [
            (library, COMPONENT_LIBRARY_FLAG),
            (props_box, COMPONENT_PROPS_BOX),
            (emit_props_box, COMPONENT_EMIT_PROPS_BOX),
        ]
        .into_iter()
        .chain(
            (fallthrough == Fallthrough::Native)
                .then_some((fallthrough_box, COMPONENT_NATIVE_FALLTHROUGH_BOX)),
        ) {
            let helper = helper.select(true);
            vize_l0::emit_ts!(
                ts,
                "{indent}type {alias} = {helper}<typeof {component_ref}>;\n"
            );
        }
        return;
    }
    // A vue-tsc generic library export instantiates its generics with their
    // constraints here, so its payload comes from the bound-props call below.
    vize_l0::emit_ts!(
        ts,
        "{indent}type {library} = typeof {component_ref} extends {{ __vizeCheck: any }} | {{ readonly __vizeComponentMarker: true }} ? false : typeof {component_ref} extends (...args: any[]) => infer __R ? 0 extends (1 & __R) ? false : '__ctx' extends keyof __R ? true : false : false;\n",
    );
    vize_l0::emit_ts!(
        ts,
        "{indent}type {props_box} = typeof {component_ref} extends {{ new (...args: any[]): {{ $props: infer __P }} }} ? [__P] : typeof {component_ref} extends (props: infer __P, ...args: any[]) => any ? [__P] : [];\n",
    );
    vize_l0::emit_ts!(
        ts,
        "{indent}type {emit_props_box} = typeof {component_ref} extends {{ __vizeEmitProps?: infer __EP }} ? [__EP] : [];\n",
    );
    match fallthrough {
        Fallthrough::Native => {
            native_fallthrough::emit_box(ts, indent, fallthrough_box, component_ref, is_any);
        }
        // An `any` root has no typed contract, the same as no root.
        Fallthrough::Contextual => vize_l0::emit_ts!(
            ts,
            "{indent}type {fallthrough_box} = typeof {component_ref} extends {{ readonly __vizeFallthroughProps?: infer __F }} ? {is_any}<__F> extends true ? [] : [__F] : [];\n",
        ),
        Fallthrough::None => {}
    }
}
