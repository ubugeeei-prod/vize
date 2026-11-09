//! Content-mapper per-prop checks for values the props literal also carries.
//!
//! The upstream content-mapper host reports every virtual diagnostic, so the
//! mapper ignores the rest of a per-prop assignment whose value the props
//! literal carries. The literal types an undeclared attr as `unknown`, so such
//! an assignment keeps only the contract the literal leaves unchecked on its
//! name (#12458). `satisfies` keeps the value contextually typed by the full
//! contract, and the value's own type reaches the annotation, so the message
//! is the per-prop one. Native checks dedupe after mapping and keep the plain
//! assignment.

use super::component_props::{ComponentPropSource, has_per_prop_value_check};
use super::props_literal::{literal_carries_value, named_entry_marks};
use crate::virtual_ts::helpers::{
    component_prop_key, push_ts_string_literal, shared_types, to_safe_identifier_fragment,
};
use vize_carton::{String, append, cstr};
use vize_croquis::croquis::{ComponentUsage, PassedProp};

/// Opening of every owned per-prop annotation. The content mapper appends
/// [`unchecked_prop_value_declaration`] once when it finds one.
pub(crate) const UNCHECKED_PROP_VALUE_REFERENCE: &str = "$UncheckedPropValue<";

/// The per-prop contract `V` unless the literal's typed inputs `D` declare
/// `K`, which leaves the value to the literal. With the shared module this is
/// the declared lookup the per-prop contract already resolved; standalone
/// output keeps that helper local to the template, so it is spelled out.
pub(crate) fn unchecked_prop_value_declaration(shared_virtual_module: bool) -> String {
    let declared = if shared_virtual_module {
        cstr!("{}<D, K, never>", shared_types::PROP_VALUE.select(true))
    } else {
        String::from("D extends unknown ? (K extends keyof D ? D[K] : never) : never")
    };
    cstr!(
        "type $UncheckedPropValue<D, K extends PropertyKey, V> = [{declared}] extends [never] ? V : unknown;\n"
    )
}

/// The per-prop assignments of one usage whose value the literal carries.
pub(crate) struct OwnedPropValues {
    owned: Vec<bool>,
    component: String,
    idx: usize,
}

impl OwnedPropValues {
    /// Decide the rows from the literal's own entry marks, and declare the
    /// usage's typed inputs once when any row is owned.
    pub(crate) fn emit(
        ts: &mut crate::virtual_ts::TextBuf,
        (usage, idx, component_ref): (&ComponentUsage, usize, &str),
        source_context: ComponentPropSource<'_>,
        (indent, hoist_shared_preamble): (&str, bool),
    ) -> Self {
        let component = to_safe_identifier_fragment(usage.name.as_str());
        if !source_context.unchecked_prop_values {
            return Self {
                owned: Vec::new(),
                component,
                idx,
            };
        }
        // Merge flags only change `class`/`style` rows, which never own here.
        let marks = named_entry_marks(usage, (false, false));
        let owned = usage
            .props
            .iter()
            .enumerate()
            .map(|(index, prop)| {
                has_per_prop_value_check(prop)
                    && literal_carries_value(usage, &marks, index)
                    && !writes_model(usage, prop)
            })
            .collect::<Vec<_>>();
        if owned.contains(&true) {
            append_declared_inputs(
                ts,
                (component.as_str(), idx, component_ref),
                hoist_shared_preamble,
                indent,
            );
        }
        Self {
            owned,
            component,
            idx,
        }
    }

    pub(crate) fn owns(&self, index: usize) -> bool {
        self.owned.get(index).copied().unwrap_or(false)
    }

    /// `: $UncheckedPropValue<inputs, key, contract> = (`
    pub(crate) fn append_type(&self, ts: &mut crate::virtual_ts::TextBuf, prop: &PassedProp) {
        let (component, idx) = (self.component.as_str(), self.idx);
        let name = to_safe_identifier_fragment(prop.name.as_str());
        append!(
            *ts,
            ": {UNCHECKED_PROP_VALUE_REFERENCE}__{component}_Declared_{idx}, "
        );
        push_ts_string_literal(ts, component_prop_key(prop.name.as_str()).as_str());
        append!(*ts, ", __{component}_{idx}_prop_{name}> = (");
    }

    /// `) satisfies contract` after the value. The parentheses keep a
    /// multi-line value from ending the statement before `satisfies`.
    pub(crate) fn append_tail(&self, ts: &mut crate::virtual_ts::TextBuf, prop: &PassedProp) {
        let (component, idx) = (self.component.as_str(), self.idx);
        let name = to_safe_identifier_fragment(prop.name.as_str());
        append!(*ts, ") satisfies __{component}_{idx}_prop_{name}");
    }
}

/// A `v-model` value and its modifiers keep their write-back owners.
fn writes_model(usage: &ComponentUsage, prop: &PassedProp) -> bool {
    usage.events.iter().any(|event| {
        event.start == prop.start && event.end == prop.end && event.name.starts_with("update:")
    })
}

/// The literal's typed inputs: own props, emitted listeners, required
/// fallthrough props and Vue's public attrs. Its untyped attr tail never
/// checks a value. A fallthrough root's literal reads its own props as
/// `Partial` and reports the same message for a wrong value, so the declared
/// props alone decide ownership there too.
fn append_declared_inputs(
    ts: &mut crate::virtual_ts::TextBuf,
    (component, idx, component_ref): (&str, usize, &str),
    hoist_shared_preamble: bool,
    indent: &str,
) {
    let emit_listeners = shared_types::EMIT_LISTENERS.select(hoist_shared_preamble);
    let required_fallthrough =
        shared_types::REQUIRED_FALLTHROUGH_PROPS.select(hoist_shared_preamble);
    append!(
        *ts,
        "{indent}type __{component}_Declared_{idx} = __{component}_Props_{idx} & {emit_listeners}<typeof {component_ref}> & {required_fallthrough}<typeof {component_ref}> & $PublicComponentAttrs;\n"
    );
}
