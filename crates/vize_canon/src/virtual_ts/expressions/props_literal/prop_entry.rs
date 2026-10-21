use crate::virtual_ts::expressions::component_props::{
    ComponentPropSource, merged_class_binding_value,
};
use crate::virtual_ts::expressions::map_rewritten_template_binding;
use crate::virtual_ts::expressions::prop_sources::{
    PropSpan, append_prop_value, generated_prop_span, prop_name_source_range,
    prop_value_source_range,
};
use crate::virtual_ts::helpers::component_prop_key;
use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
use crate::virtual_ts::types::{VizeMapping, VizeSubSpan};
use vize_carton::{FxHashSet, String, append};
use vize_croquis::croquis::PassedProp;

use super::NAMED_PROP_DIAGNOSTIC_MARKER;

#[expect(clippy::too_many_arguments, reason = "independent emitter inputs")]
pub(super) fn append_prop_entry(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    prop: &PassedProp,
    template_binding_access: &TemplateBindingAccess,
    source_context: ComponentPropSource<'_>,
    expr_indent: &str,
    merge_class_bindings: bool,
    class_bindings: &[(&PassedProp, String)],
    emitted_merged_class: &mut bool,
    before_following_spread: bool,
    authored_named_keys: &mut FxHashSet<String>,
) {
    let template_offset = source_context.offset;
    let mut merged_value_ranges = Vec::new();
    let merge_named_bindings =
        merge_class_bindings && matches!(prop.name.as_str(), "class" | "style");
    let generated_value = if merge_named_bindings {
        if *emitted_merged_class {
            return;
        }
        *emitted_merged_class = true;
        merged_class_binding_value(class_bindings, Some(&mut merged_value_ranges))
            .map(PropSpan::plain)
    } else {
        generated_prop_span(prop, template_binding_access)
    };
    let Some(mut generated_value) = generated_value else {
        return;
    };
    let inline_callback = crate::virtual_ts::scope::is_inline_callback_prop(prop);
    if inline_callback {
        // The authored callback is checked against its resolved type separately.
        // `never` also satisfies a declared `never` prop; `any` would produce a
        // second diagnostic about this synthetic value at the same attribute.
        generated_value = PropSpan::plain(String::from("undefined as never"));
    }

    let (prop_src_start, prop_src_end) = if merge_named_bindings {
        let start = class_bindings
            .iter()
            .map(|(binding, _)| binding.start)
            .min()
            .unwrap_or(prop.start);
        let end = class_bindings
            .iter()
            .map(|(binding, _)| binding.end)
            .max()
            .unwrap_or(prop.end);
        (
            (template_offset + start) as usize,
            (template_offset + end) as usize,
        )
    } else {
        (
            (template_offset + prop.start) as usize,
            (template_offset + prop.end) as usize,
        )
    };
    let camel_prop_name = component_prop_key(prop.name.as_str());

    let repeated_named_key = !authored_named_keys.insert(camel_prop_name.clone());
    if before_following_spread && !repeated_named_key {
        // Keep named props direct so TypeScript contextually elaborates their
        // value errors at the property instead of reporting a second TS2345 on
        // the whole object when a later v-bind spread exists (#10069). The
        // dedicated per-prop assignment above owns that diagnostic. Ignoring
        // this generated entry also suppresses TS2783 when the later spread
        // deliberately overwrites it, while the spread itself remains fully
        // checked by the surrounding call.
        append!(
            *ts,
            "{expr_indent}  // {NAMED_PROP_DIAGNOSTIC_MARKER} are owned by the per-prop check.\n",
        );
    }
    append!(*ts, "{expr_indent}  ");

    let entry_gen_start = ts.len();
    append!(*ts, "\"{camel_prop_name}\"");
    let key_gen_end = ts.len();
    ts.push_str(": ");
    let value_gen_range = append_prop_value(ts, generated_value.text.as_str());
    let entry_gen_end = if generated_value.omits_tail() {
        generated_value.mapped_limit(value_gen_range.start)
    } else {
        ts.len()
    };
    let value_span = if generated_value.omits_tail() {
        value_gen_range.start..entry_gen_end
    } else {
        value_gen_range.clone()
    };
    ts.push_str(",\n");

    let sub_spans = match merge_named_bindings {
        true => class_bindings
            .iter()
            .zip(&merged_value_ranges)
            .filter(|((binding, _), _)| binding.is_dynamic)
            .filter_map(|((binding, _), generated)| {
                Some(VizeSubSpan {
                    gen_range: value_span.start + generated.start..value_span.start + generated.end,
                    src_range: prop_value_source_range(source_context, binding)?,
                })
            })
            .collect(),
        false if inline_callback => prop_name_source_range(source_context, prop)
            .map_or_else(Vec::new, |src_range| {
                key_sub_spans(entry_gen_start..key_gen_end, src_range)
            }),
        false => entry_sub_spans(
            source_context,
            prop,
            entry_gen_start..key_gen_end,
            value_span.clone(),
        ),
    };
    mappings.push(VizeMapping {
        gen_range: entry_gen_start..entry_gen_end,
        src_range: prop_src_start..prop_src_end,
        sub_spans,
    });
    if merge_named_bindings && !template_binding_access.is_empty() {
        for ((binding, _), generated) in class_bindings.iter().zip(&merged_value_ranges) {
            if binding.is_dynamic
                && let Some(value) = binding.value.as_ref()
                && let Some(source) = prop_value_source_range(source_context, binding)
            {
                map_rewritten_template_binding(
                    ts.get(..value_span.start + generated.end)
                        .unwrap_or_default(),
                    mappings,
                    value_span.start + generated.start,
                    source.start,
                    value.as_str(),
                    template_binding_access,
                );
            }
        }
    }
    if !template_binding_access.is_empty()
        && prop.is_dynamic
        && !inline_callback
        && !merge_named_bindings
        && let Some(value) = prop.value.as_ref()
        && let Some(source) = prop_value_source_range(source_context, prop)
    {
        map_rewritten_template_binding(
            ts,
            mappings,
            value_span.start,
            source.start,
            value.as_str(),
            template_binding_access,
        );
    }
}

pub(crate) fn entry_sub_spans(
    source_context: ComponentPropSource<'_>,
    prop: &PassedProp,
    key_gen_range: std::ops::Range<usize>,
    value_gen_range: std::ops::Range<usize>,
) -> Vec<VizeSubSpan> {
    if let Some(spans) = crate::virtual_ts::expressions::model_modifiers::modifier_sub_spans(
        source_context,
        prop,
        value_gen_range.clone(),
    ) {
        return spans;
    }
    let Some(name_src_range) = prop_name_source_range(source_context, prop) else {
        return Vec::new();
    };
    let Some(value_src_range) = prop_value_source_range(source_context, prop) else {
        return Vec::new();
    };
    let mut spans = key_sub_spans(key_gen_range, name_src_range);
    spans.push(VizeSubSpan {
        gen_range: value_gen_range,
        src_range: value_src_range,
    });
    spans
}

fn key_sub_spans(
    generated: std::ops::Range<usize>,
    source: std::ops::Range<usize>,
) -> Vec<VizeSubSpan> {
    vec![
        // TypeScript rename targets the literal contents, whereas diagnostics
        // can cover its quotes too. Both ranges own the same authored prop.
        VizeSubSpan {
            gen_range: generated.start + 1..generated.end - 1,
            src_range: source.clone(),
        },
        VizeSubSpan {
            gen_range: generated,
            src_range: source,
        },
    ]
}
