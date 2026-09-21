//! What a component `v-model` writes back (`strictVModel`).
//!
//! The binding is a prop going in and an `update:` listener coming out. The
//! prop check covers the first half; this assigns the listener's payload to
//! the bound expression, so `v-model:foo="count"` on a child that may emit
//! `undefined` is the `TS2322` the Vue toolchain reports for it.

use std::ops::Range;

use vize_carton::{String, append, camelize};
use vize_croquis::croquis::{ComponentUsage, PassedProp};

use crate::virtual_ts::helpers::{push_ts_string_literal, to_safe_identifier_fragment};
use crate::virtual_ts::types::{VizeMapping, VizeSubSpan};

/// Whether `prop` is the prop half of a `v-model` on `usage`.
pub(super) fn is_model_prop(usage: &ComponentUsage, prop: &PassedProp) -> bool {
    !prop.name_is_dynamic
        && usage.events.iter().any(|event| {
            event.start == prop.start
                && event.end == prop.end
                && event
                    .name
                    .strip_prefix("update:")
                    .is_some_and(|name| name == prop.name.as_str())
        })
}

/// Emit `type … = <payload>; void ((value: …) => { <target> = value; });`.
pub(super) fn append_model_update_check(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    (component_type_name, idx): (&str, usize),
    prop: &PassedProp,
    target: &str,
    target_source: Range<usize>,
    indent: &str,
) {
    let safe_prop_name = to_safe_identifier_fragment(prop.name.as_str());
    let payload = vize_carton::cstr!("__{component_type_name}_{idx}_model_{safe_prop_name}");
    append!(
        *ts,
        "{indent}type {payload} = __VizeEmitListeners<__{component_type_name}_Component_{idx}> extends {{ ",
    );
    push_ts_string_literal(
        ts,
        &vize_carton::cstr!("onUpdate:{}", camelize(prop.name.as_str())),
    );
    ts.push_str("?: infer __L } ? NonNullable<__L> extends (...args: infer __A) => any ? __A extends [infer __V, ...any[]] ? __V : any : any : any;\n");

    let statement_start = ts.len();
    append!(*ts, "{indent}void ((__vize_model_value: {payload}) => {{ ");
    let target_start = ts.len();
    ts.push_str(target);
    let target_end = ts.len();
    ts.push_str(" = __vize_model_value; });\n");
    mappings.push(VizeMapping {
        gen_range: statement_start..ts.len(),
        src_range: target_source.clone(),
        sub_spans: vec![VizeSubSpan {
            gen_range: target_start..target_end,
            src_range: target_source,
        }],
    });
}
