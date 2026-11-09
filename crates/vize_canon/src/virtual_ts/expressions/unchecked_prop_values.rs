//! Content-mapper value checks the whole-props literal cannot own.
//!
//! The upstream content-mapper host reports every virtual diagnostic, so the
//! mapper ignores a per-prop assignment whenever the props literal repeats the
//! same binding. The literal types undeclared attrs as `unknown`, so a wrong
//! fallthrough value would otherwise vanish (#12458). Each companion check
//! keeps the per-prop contract only where the literal leaves the key
//! unchecked; native checks dedupe after mapping and never emit it.

use super::component_props::{ComponentPropSource, is_checkable_prop};
use super::prop_sources::{
    append_prop_value, generated_diagnostic_prop_span, prop_name_source_range,
    prop_value_source_range,
};
use crate::virtual_ts::helpers::{
    component_aria_prop_keys, component_prop_key, push_ts_string_literal,
    to_safe_identifier_fragment,
};
use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
use crate::virtual_ts::types::{VizeMapping, VizeSubSpan};
use vize_carton::{FxHashMap, String, append, cstr};
use vize_croquis::croquis::ComponentUsage;

/// Statement prefix the content mapper recognises.
pub(crate) const UNCHECKED_PROP_VALUE_PREFIX: &str = "__vize_unchecked_prop_";

/// Opening of every companion annotation. The content mapper appends
/// [`UNCHECKED_PROP_VALUE_DECLARATION`] once when it finds one.
pub(crate) const UNCHECKED_PROP_VALUE_REFERENCE: &str = "$UncheckedPropValue<";

/// The per-prop contract `V` where the whole-props check `C` leaves key `K`
/// unchecked or checks another type; `unknown` where the literal reports the
/// same diagnostic.
pub(crate) const UNCHECKED_PROP_VALUE_DECLARATION: &str = "type $UncheckedPropValue<C, K extends PropertyKey, V, __L = C extends (props: infer __P, ...args: any) => any ? K extends keyof __P ? __P[K] : never : never> = 0 extends 1 & __L ? V : [__L] extends [V] ? [V] extends [__L] ? unknown : V : V;\n";

pub(super) fn append_unchecked_prop_value_checks(
    ts: &mut crate::virtual_ts::TextBuf,
    mappings: &mut Vec<VizeMapping>,
    (usage, idx): (&ComponentUsage, usize),
    template_binding_access: &TemplateBindingAccess,
    source_context: ComponentPropSource<'_>,
    indent: &str,
) {
    if !source_context.unchecked_prop_values {
        return;
    }
    let component = to_safe_identifier_fragment(usage.name.as_str());
    let mut occurrences: FxHashMap<String, u32> = FxHashMap::default();
    for prop in &usage.props {
        // ARIA entries and inline callbacks are never literal echoes. A
        // `v-model` value and its modifiers keep their own owners.
        if !is_checkable_prop(prop)
            || prop.value.is_none()
            || crate::virtual_ts::scope::is_inline_callback_prop(prop)
            || component_aria_prop_keys(prop.name.as_str()).is_some()
            || usage.events.iter().any(|event| {
                event.start == prop.start
                    && event.end == prop.end
                    && event.name.starts_with("update:")
            })
        {
            continue;
        }
        let (Some(value), Some(src_range)) = (
            generated_diagnostic_prop_span(prop, template_binding_access),
            prop_name_source_range(source_context, prop)
                .or_else(|| prop_value_source_range(source_context, prop)),
        ) else {
            continue;
        };
        let name = to_safe_identifier_fragment(prop.name.as_str());
        let occurrence = occurrences
            .entry(name.clone())
            .and_modify(|count| *count += 1)
            .or_insert(1);
        let check = if *occurrence == 1 {
            cstr!("{UNCHECKED_PROP_VALUE_PREFIX}{idx}_{name}")
        } else {
            cstr!("{UNCHECKED_PROP_VALUE_PREFIX}{idx}_{name}_{occurrence}")
        };
        let statement_start = ts.len();
        append!(*ts, "{indent}const ");
        let check_start = ts.len();
        ts.push_str(check.as_str());
        let check_end = ts.len();
        append!(
            *ts,
            ": {UNCHECKED_PROP_VALUE_REFERENCE}__{component}_Check_{idx}, "
        );
        push_ts_string_literal(ts, component_prop_key(prop.name.as_str()).as_str());
        append!(*ts, ", __{component}_{idx}_prop_{name}> = ");
        let value_range = append_prop_value(ts, value.text.as_str());
        ts.push_str(";\n");
        let statement_end = ts.len();
        append!(*ts, "{indent}void {check};\n");

        // Only the name is mapped. The value repeats the literal's entry and
        // stays an inert copy, exactly like a slot host's.
        source_context.inert().record_prop_projection(
            usage,
            prop,
            value_range.start..value_range.start,
            value_range,
        );
        mappings.push(VizeMapping {
            gen_range: statement_start..statement_end,
            src_range: (source_context.offset + prop.start) as usize
                ..(source_context.offset + prop.end) as usize,
            sub_spans: vec![VizeSubSpan {
                gen_range: check_start..check_end,
                src_range,
            }],
        });
    }
}
