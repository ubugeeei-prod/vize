//! Hoisted listener contracts: the per-component and per-event aliases read
//! the shared `$Listener*` helpers instead of inlining their text, so one
//! instantiation serves every module that renders the same component.

use vize_carton::{String, append, cstr};

use crate::virtual_ts::helpers::shared_types::{
    LISTENER_EMIT_PROPS_BOX, LISTENER_FALLTHROUGH_ARGS, LISTENER_FALLTHROUGH_ARGS_RAW,
    LISTENER_FALLTHROUGH_BOX, LISTENER_LIBRARY, LISTENER_NATIVE_FALLTHROUGH_BOX, LISTENER_PROP_ARGS,
    LISTENER_PROP_ARGS_RAW, LISTENER_PROP_ARRAYS, LISTENER_PROPS_BOX, LISTENER_STATIC_EMIT_ARGS,
};

pub(super) struct ComponentBoxes<'a> {
    pub(super) indent: &'a str,
    pub(super) component_ref: &'a str,
    pub(super) library: &'a str,
    pub(super) props_box: &'a str,
    pub(super) emit_props_box: &'a str,
    pub(super) fallthrough_box: &'a str,
    pub(super) native_fallthrough_only: bool,
    pub(super) fallthrough_listeners: bool,
}

pub(super) fn emit_component(ts: &mut String, boxes: ComponentBoxes<'_>) {
    let ComponentBoxes {
        indent,
        component_ref,
        library,
        props_box,
        emit_props_box,
        fallthrough_box,
        native_fallthrough_only,
        fallthrough_listeners,
    } = boxes;
    append!(
        *ts,
        "{indent}type {library} = {LISTENER_LIBRARY}<typeof {component_ref}>;\n{indent}type {props_box} = {LISTENER_PROPS_BOX}<typeof {component_ref}>;\n{indent}type {emit_props_box} = {LISTENER_EMIT_PROPS_BOX}<typeof {component_ref}>;\n",
    );
    let fallthrough = if native_fallthrough_only {
        LISTENER_NATIVE_FALLTHROUGH_BOX
    } else if fallthrough_listeners {
        LISTENER_FALLTHROUGH_BOX
    } else {
        return;
    };
    append!(*ts, "{indent}type {fallthrough_box} = {fallthrough}<typeof {component_ref}>;\n");
}

/// The listener key and, for a kebab event, the raw `onFoo-bar` key.
pub(super) struct ListenerKey<'a> {
    pub(super) key: &'a str,
    pub(super) raw: Option<&'a str>,
}

impl ListenerKey<'_> {
    fn keyed(&self, plain: &str, raw: &str, contract: &str) -> String {
        let key = self.key;
        match self.raw {
            Some(raw_key) => cstr!("{raw}<{contract}, {key}, {raw_key}>"),
            None => cstr!("{plain}<{contract}, {key}>"),
        }
    }

    pub(super) fn prop_args(&self, library: &str, props_box: &str) -> String {
        let contract = cstr!("{library}, {props_box}");
        self.keyed(LISTENER_PROP_ARGS, LISTENER_PROP_ARGS_RAW, &contract)
    }

    pub(super) fn fallthrough_args(&self, fallthrough_box: &str) -> String {
        self.keyed(LISTENER_FALLTHROUGH_ARGS, LISTENER_FALLTHROUGH_ARGS_RAW, fallthrough_box)
    }

    pub(super) fn static_emit_args(&self, emit_props_box: &str) -> String {
        let key = self.key;
        cstr!("{LISTENER_STATIC_EMIT_ARGS}<{emit_props_box}, {key}>")
    }

    pub(super) fn prop_arrays(&self, props_box: &str) -> String {
        let key = self.key;
        cstr!("{LISTENER_PROP_ARRAYS}<{props_box}, {key}>")
    }
}
