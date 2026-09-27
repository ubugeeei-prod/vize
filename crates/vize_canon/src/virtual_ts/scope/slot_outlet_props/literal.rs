//! Source-mapped slot payload literals shared by checking and inference.

use super::SlotOutlet;
use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
use crate::virtual_ts::{
    expressions::{
        ComponentPropSource, append_prop_value, generated_prop_value, prop_name_source_range,
        prop_value_source_range,
    },
    helpers::to_camel_case,
    types::{VizeMapping, VizeSubSpan},
};
use std::ops::Range;
use vize_carton::{String, append};
use vize_croquis::croquis::{PassedProp, SpreadProp};

enum SlotOutletLiteralEntry<'a> {
    Prop(&'a PassedProp),
    Spread(&'a SpreadProp),
}

impl SlotOutletLiteralEntry<'_> {
    const fn start(&self) -> u32 {
        match self {
            Self::Prop(prop) => prop.start,
            Self::Spread(spread) => spread.start,
        }
    }
}

/// What the literal is emitted for, which decides how its entries are typed.
#[derive(Clone, Copy)]
pub(super) enum SlotOutletLiteralMode<'a> {
    /// The payload a slots type is inferred from: nothing constrains it, so a
    /// static string attribute keeps its literal type.
    Infer,
    /// The argument checked against the declared slot payload type.
    Check { payload_type: &'a str },
}

pub(super) fn append_slot_outlet_literal(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    outlet: &SlotOutlet,
    mode: SlotOutletLiteralMode<'_>,
    template_binding_access: &TemplateBindingAccess,
    source_context: ComponentPropSource<'_>,
    expr_indent: &str,
) -> Range<usize> {
    let payload_type = match mode {
        SlotOutletLiteralMode::Infer => "unknown",
        SlotOutletLiteralMode::Check { payload_type } => payload_type,
    };
    let literal_static_values = matches!(mode, SlotOutletLiteralMode::Infer);
    let literal_gen_start = ts.len();
    ts.push_str("{\n");

    let mut entries = Vec::with_capacity(outlet.props.len() + outlet.spread_props.len());
    entries.extend(outlet.props.iter().map(SlotOutletLiteralEntry::Prop));
    entries.extend(
        outlet
            .spread_props
            .iter()
            .map(SlotOutletLiteralEntry::Spread),
    );
    entries.sort_by_key(SlotOutletLiteralEntry::start);

    for entry in entries {
        match entry {
            SlotOutletLiteralEntry::Prop(prop) => {
                let Some(generated_value) = generated_prop_value(prop, template_binding_access)
                else {
                    continue;
                };
                let prop_src_start = (source_context.offset + prop.start) as usize;
                let prop_src_end = (source_context.offset + prop.end) as usize;
                append!(*ts, "{expr_indent}  ");
                let entry_gen_start = ts.len();
                let camel_prop_name = to_camel_case(prop.name.as_str());
                append!(*ts, "\"{camel_prop_name}\"");
                let key_gen_end = ts.len();
                ts.push_str(": ");
                let value_gen_range = append_prop_value(ts, generated_value.as_str());
                // An inferred payload is a plain object literal, which widens
                // `viewMode="sp"` to `string`; the parent then cannot pass it
                // on to a `'pc' | 'sp'` prop. Keep the authored literal type,
                // as the runtime value is exactly that string.
                if literal_static_values && is_static_string_value(prop) {
                    ts.push_str(" as const");
                }
                let entry_gen_end = ts.len();
                ts.push_str(",\n");
                mappings.push(VizeMapping {
                    gen_range: entry_gen_start..entry_gen_end,
                    src_range: prop_src_start..prop_src_end,
                    sub_spans: entry_sub_spans(
                        source_context,
                        prop,
                        entry_gen_start..key_gen_end,
                        value_gen_range,
                    ),
                });
            }
            SlotOutletLiteralEntry::Spread(spread) => {
                append!(
                    *ts,
                    "{expr_indent}  ...__vizeSlotOutletSpread<{payload_type}>()(",
                );
                let gen_range = append_prop_value(ts, spread.expression.as_str());
                ts.push_str("),\n");
                let source_expression = spread_expression_source_range(source_context, spread);
                mappings.push(VizeMapping {
                    gen_range: gen_range.clone(),
                    src_range: (source_context.offset + spread.start) as usize
                        ..(source_context.offset + spread.end) as usize,
                    sub_spans: source_expression.map_or_else(Vec::new, |src_range| {
                        vec![VizeSubSpan {
                            gen_range,
                            src_range,
                        }]
                    }),
                });
            }
        }
    }

    append!(*ts, "{expr_indent}}}");
    literal_gen_start..ts.len()
}

/// A static attribute with an authored string value: `name="value"`, not a
/// valueless attribute and not `style`, whose object form has its own shape.
fn is_static_string_value(prop: &PassedProp) -> bool {
    !prop.is_dynamic && prop.value.is_some() && prop.name != "style"
}

fn entry_sub_spans(
    source_context: ComponentPropSource<'_>,
    prop: &PassedProp,
    key_gen_range: Range<usize>,
    value_gen_range: Range<usize>,
) -> Vec<VizeSubSpan> {
    let mut sub_spans = Vec::new();
    // Keep key and value spans independent when authored value text is synthetic.
    if let Some(name_src_range) = prop_name_source_range(source_context, prop) {
        sub_spans.push(VizeSubSpan {
            gen_range: key_gen_range,
            src_range: name_src_range,
        });
    }
    if let Some(value_src_range) = prop_value_source_range(source_context, prop) {
        sub_spans.push(VizeSubSpan {
            gen_range: value_gen_range,
            src_range: value_src_range,
        });
    }
    sub_spans
}

fn spread_expression_source_range(
    source_context: ComponentPropSource<'_>,
    spread: &SpreadProp,
) -> Option<Range<usize>> {
    let source = source_context.template?;
    let raw = source.get(spread.start as usize..spread.end as usize)?;
    let relative_start = raw.rfind(spread.expression.as_str())?;
    let source_start = source_context.offset as usize + spread.start as usize + relative_start;
    Some(source_start..source_start + spread.expression.len())
}
