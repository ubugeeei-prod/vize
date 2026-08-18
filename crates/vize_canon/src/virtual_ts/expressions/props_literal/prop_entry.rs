use super::close_named_group;
use crate::virtual_ts::expressions::component_props::{
    ComponentPropSource, merged_class_binding_value,
};
use crate::virtual_ts::expressions::prop_sources::{
    append_prop_value, generated_prop_value, prop_name_source_range, prop_value_source_range,
};
use crate::virtual_ts::helpers::to_camel_case;
use crate::virtual_ts::types::{VizeMapping, VizeSubSpan};
use vize_carton::{FxHashSet, String, append};
use vize_croquis::croquis::PassedProp;

#[allow(clippy::too_many_arguments)]
pub(super) fn append_prop_entry(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    prop: &PassedProp,
    template_prop_names: &FxHashSet<String>,
    source_context: ComponentPropSource<'_>,
    expr_indent: &str,
    merge_class_bindings: bool,
    class_bindings: &[(&PassedProp, String)],
    emitted_merged_class: &mut bool,
    wrap_for_following_spread: bool,
    open_named_group: &mut bool,
) {
    let template_offset = source_context.offset;
    let generated_value = if merge_class_bindings && prop.name.as_str() == "class" {
        if *emitted_merged_class {
            return;
        }
        *emitted_merged_class = true;
        merged_class_binding_value(class_bindings)
    } else {
        generated_prop_value(prop, template_prop_names)
    };
    let Some(mut generated_value) = generated_value else {
        return;
    };
    let inline_callback = crate::virtual_ts::scope::is_inline_callback_prop(prop);
    if inline_callback {
        generated_value = String::from("undefined as any");
    }

    let (prop_src_start, prop_src_end) = if merge_class_bindings && prop.name.as_str() == "class" {
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
    let camel_prop_name = to_camel_case(prop.name.as_str());

    if wrap_for_following_spread {
        if *open_named_group {
            ts.push_str(", ");
        } else {
            append!(*ts, "{expr_indent}  ...{{ ");
            *open_named_group = true;
        }
    } else {
        close_named_group(ts, open_named_group);
        append!(*ts, "{expr_indent}  ");
    }

    let entry_gen_start = ts.len();
    append!(*ts, "\"{camel_prop_name}\"");
    let key_gen_end = ts.len();
    ts.push_str(": ");
    let value_gen_range = append_prop_value(ts, generated_value.as_str());
    let entry_gen_end = ts.len();
    if !wrap_for_following_spread {
        ts.push_str(",\n");
    }

    let sub_spans = match merge_class_bindings && prop.name.as_str() == "class" {
        true => Vec::new(),
        false if inline_callback => {
            prop_name_source_range(source_context, prop).map_or_else(Vec::new, |src_range| {
                vec![VizeSubSpan {
                    gen_range: entry_gen_start..key_gen_end,
                    src_range,
                }]
            })
        }
        false => entry_sub_spans(
            source_context,
            prop,
            entry_gen_start..key_gen_end,
            value_gen_range,
        ),
    };
    mappings.push(VizeMapping {
        gen_range: entry_gen_start..entry_gen_end,
        src_range: prop_src_start..prop_src_end,
        sub_spans,
    });
}

fn entry_sub_spans(
    source_context: ComponentPropSource<'_>,
    prop: &PassedProp,
    key_gen_range: std::ops::Range<usize>,
    value_gen_range: std::ops::Range<usize>,
) -> Vec<VizeSubSpan> {
    let Some(name_src_range) = prop_name_source_range(source_context, prop) else {
        return Vec::new();
    };
    let Some(value_src_range) = prop_value_source_range(source_context, prop) else {
        return Vec::new();
    };
    vec![
        VizeSubSpan {
            gen_range: key_gen_range,
            src_range: name_src_range,
        },
        VizeSubSpan {
            gen_range: value_gen_range,
            src_range: value_src_range,
        },
    ]
}
