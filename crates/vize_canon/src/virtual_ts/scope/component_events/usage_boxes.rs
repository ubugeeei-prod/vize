//! A component's listener boxes: whether it is a vue-tsc generic library
//! export, and its `$props` and emit props as one-tuples (`[]` when absent).

use crate::virtual_ts::helpers::shared_types;

pub(super) struct Boxes<'a> {
    pub(super) library: &'a str,
    pub(super) props: &'a str,
    pub(super) emit_props: &'a str,
}

/// Each box reads only the component type. Hoisted modules name the shared
/// helpers, which check it non-distributively exactly as the inline forms do,
/// so TypeScript evaluates them once per component type instead of per usage.
pub(super) fn emit(
    ts: &mut crate::virtual_ts::TextBuf,
    indent: &str,
    boxes: Boxes<'_>,
    component_ref: &str,
    hoisted: bool,
) {
    let Boxes {
        library,
        props: props_box,
        emit_props: emit_props_box,
    } = boxes;
    if hoisted {
        let usage_library = shared_types::USAGE_LIBRARY.select(true);
        let usage_props_box = shared_types::USAGE_PROPS_BOX.select(true);
        let usage_emit_props_box = shared_types::USAGE_EMIT_PROPS_BOX.select(true);
        vize_l0::emit_ts!(
            ts,
            "{indent}type {library} = {usage_library}<typeof {component_ref}>;\n",
        );
        vize_l0::emit_ts!(
            ts,
            "{indent}type {props_box} = {usage_props_box}<typeof {component_ref}>;\n",
        );
        vize_l0::emit_ts!(
            ts,
            "{indent}type {emit_props_box} = {usage_emit_props_box}<typeof {component_ref}>;\n",
        );
        return;
    }
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
}
