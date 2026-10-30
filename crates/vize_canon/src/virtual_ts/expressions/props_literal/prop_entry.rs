use crate::virtual_ts::expressions::component_prop_bindings::merged_class_binding_value;
use crate::virtual_ts::expressions::component_props::ComponentPropSource;
use crate::virtual_ts::expressions::map_rewritten_template_binding;
use crate::virtual_ts::expressions::prop_sources::{
    PropSpan, append_prop_value, generated_prop_span, prop_name_source_range,
    prop_value_source_range,
};
use crate::virtual_ts::helpers::{component_aria_prop_keys, component_prop_key};
use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
use crate::virtual_ts::types::{VizeMapping, VizeSubSpan};
use vize_carton::{FxHashSet, String, append};
use vize_croquis::croquis::PassedProp;

use super::NAMED_PROP_DIAGNOSTIC_MARKER;

pub(super) struct PropDiagnosticRows<'p> {
    pub(super) name: std::ops::Range<usize>,
    pub(super) value: std::ops::Range<usize>,
    pub(super) ignored_primary: bool,
    pub(super) merged: bool,
    /// Dynamic values repeated inside a merged `class`/`style` entry.
    pub(super) merged_values: Vec<(&'p PassedProp, std::ops::Range<usize>)>,
}

#[expect(clippy::too_many_arguments, reason = "independent emitter inputs")]
pub(super) fn append_prop_entry<'p>(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    prop: &PassedProp,
    template_binding_access: &TemplateBindingAccess,
    source_context: ComponentPropSource<'_>,
    expr_indent: &str,
    merge_class_bindings: bool,
    class_bindings: &[(&'p PassedProp, String)],
    emitted_merged_class: &mut bool,
    before_following_spread: bool,
    authored_named_keys: &mut FxHashSet<String>,
) -> Option<PropDiagnosticRows<'p>> {
    let template_offset = source_context.offset;
    let mut merged_value_ranges = Vec::new();
    let merge_named_bindings =
        merge_class_bindings && matches!(prop.name.as_str(), "class" | "style");
    let generated_value = if merge_named_bindings {
        if *emitted_merged_class {
            return None;
        }
        *emitted_merged_class = true;
        merged_class_binding_value(class_bindings, Some(&mut merged_value_ranges))
            .map(PropSpan::plain)
    } else {
        generated_prop_span(prop, template_binding_access)
    };
    let mut generated_value = generated_value?;
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
    let (aria_raw_prop_name, camel_prop_name) =
        if let Some((raw, camel)) = component_aria_prop_keys(prop.name.as_str()) {
            (Some(raw), camel)
        } else {
            (None, component_prop_key(prop.name.as_str()))
        };
    let aria_present_value = aria_raw_prop_name.is_some();

    let repeated_named_key = !authored_named_keys.insert(camel_prop_name.clone());
    // Keep named props direct so TypeScript contextually elaborates their
    // value errors at the property instead of reporting a second TS2345 on
    // the whole object when a later v-bind spread exists (#10069). The
    // dedicated per-prop assignment above owns that diagnostic. Ignoring
    // this generated entry also suppresses TS2783 when the later spread
    // deliberately overwrites it, while the spread itself remains fully
    // checked by the surrounding call.
    // The ARIA entry witnesses both the retained declared key and Vue's camel
    // key with one value evaluation. The dedicated assignment owns its value
    // diagnostic. Suppressing this synthetic entry also retains `aria-*` as an
    // undeclared HTML attr without widening arbitrary attrs (#11048, #9658).
    if aria_present_value || (before_following_spread && !repeated_named_key) {
        append!(
            *ts,
            "{expr_indent}  // {NAMED_PROP_DIAGNOSTIC_MARKER} are owned by the per-prop check.\n",
        );
    }
    append!(*ts, "{expr_indent}  ");

    if let Some(raw) = aria_raw_prop_name.as_ref() {
        append!(*ts, "...__vizeComponentAriaPropEntries(\"{raw}\", ");
    }
    let entry_gen_start = ts.len();
    append!(*ts, "\"{camel_prop_name}\"");
    let key_gen_end = ts.len();
    ts.push_str(if aria_present_value { ", " } else { ": " });
    let value_gen_range = append_prop_value(ts, generated_value.text.as_str());
    if aria_present_value {
        ts.push(')');
    }
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

    let merged_values: Vec<_> = class_bindings
        .iter()
        .zip(&merged_value_ranges)
        .filter(|((binding, _), _)| binding.is_dynamic)
        .map(|((binding, _), generated)| {
            (
                *binding,
                value_span.start + generated.start..value_span.start + generated.end,
            )
        })
        .collect();
    let sub_spans = match merge_named_bindings {
        true => merged_values
            .iter()
            .filter_map(|(binding, gen_range)| {
                Some(VizeSubSpan {
                    gen_range: gen_range.clone(),
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
        && !generated_value.map_instance_binding(
            ts,
            mappings,
            value_span.clone(),
            source.start,
            template_binding_access,
        )
    {
        map_rewritten_template_binding(
            ts.get(..value_span.end).unwrap_or_default(),
            mappings,
            value_span.start,
            source.start,
            value.as_str(),
            template_binding_access,
        );
    }
    Some(PropDiagnosticRows {
        name: entry_gen_start..key_gen_end,
        // A callback literal carries a synthetic `never`, not the authored
        // body. Only the separately checked callback can own its value range.
        value: if inline_callback { 0..0 } else { value_span },
        ignored_primary: before_following_spread && !repeated_named_key,
        merged: merge_named_bindings,
        merged_values,
    })
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

/// Refine a rewritten prop value around generated-only insertions.
///
/// The caller supplies ranges relative to `value_gen_range`. If the authored
/// and generated lengths cannot be authenticated after removing those gaps,
/// retain only the key mappings instead of publishing a false value map.
pub(crate) fn entry_sub_spans_with_synthetic_value_gaps(
    source_context: ComponentPropSource<'_>,
    prop: &PassedProp,
    key_gen_range: std::ops::Range<usize>,
    value_gen_range: std::ops::Range<usize>,
    synthetic_ranges: &[std::ops::Range<usize>],
) -> Vec<VizeSubSpan> {
    if synthetic_ranges.is_empty() {
        return entry_sub_spans(source_context, prop, key_gen_range, value_gen_range);
    }
    let Some(name_src_range) = prop_name_source_range(source_context, prop) else {
        return Vec::new();
    };
    let Some(value_src_range) = prop_value_source_range(source_context, prop) else {
        return Vec::new();
    };
    let synthetic_len = synthetic_ranges
        .iter()
        .try_fold(0usize, |total, range| total.checked_add(range.len()));
    if synthetic_len.and_then(|length| value_gen_range.len().checked_sub(length))
        != Some(value_src_range.len())
        || synthetic_ranges
            .windows(2)
            .any(|ranges| matches!(ranges, [left, right] if left.end > right.start))
        || synthetic_ranges
            .last()
            .is_some_and(|range| range.end > value_gen_range.len())
    {
        return key_sub_spans(key_gen_range, name_src_range);
    }

    let mut spans = key_sub_spans(key_gen_range, name_src_range);
    let (mut generated_cursor, mut source_cursor) = (0usize, value_src_range.start);
    for gap in synthetic_ranges {
        if generated_cursor < gap.start {
            let length = gap.start - generated_cursor;
            spans.push(VizeSubSpan {
                gen_range: value_gen_range.start + generated_cursor
                    ..value_gen_range.start + gap.start,
                src_range: source_cursor..source_cursor + length,
            });
            source_cursor += length;
        }
        generated_cursor = gap.end;
    }
    if generated_cursor < value_gen_range.len() {
        spans.push(VizeSubSpan {
            gen_range: value_gen_range.start + generated_cursor..value_gen_range.end,
            src_range: source_cursor..value_src_range.end,
        });
    }
    spans
}

pub(crate) fn key_sub_spans(
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

#[cfg(test)]
mod tests {
    use super::{entry_sub_spans_with_synthetic_value_gaps, key_sub_spans};
    use crate::virtual_ts::expressions::component_props::ComponentPropSource;
    use crate::virtual_ts::types::VizeSubSpan;
    use vize_croquis::{ScopeChain, croquis::PassedProp};

    fn fixture() -> (&'static str, PassedProp, ScopeChain) {
        let template = "雪:entry=\"abcdef\"";
        (
            template,
            PassedProp {
                name: "entry".into(),
                name_is_dynamic: false,
                value: Some("abcdef".into()),
                start: "雪".len() as u32,
                end: template.len() as u32,
                is_dynamic: true,
            },
            ScopeChain::default(),
        )
    }

    fn spans(
        synthetic: &[std::ops::Range<usize>],
        generated: std::ops::Range<usize>,
    ) -> (Vec<VizeSubSpan>, Vec<VizeSubSpan>, std::ops::Range<usize>) {
        let (template, prop, scopes) = fixture();
        let offset = 10usize;
        let name = template.find("entry").unwrap();
        let value = template.find("abcdef").unwrap();
        let keys = key_sub_spans(20..27, offset + name..offset + name + "entry".len());
        let actual = entry_sub_spans_with_synthetic_value_gaps(
            ComponentPropSource::new(Some(template), offset as u32, &scopes),
            &prop,
            20..27,
            generated,
            synthetic,
        );
        (
            actual,
            keys,
            offset + value..offset + value + "abcdef".len(),
        )
    }

    #[test]
    fn synthetic_value_gaps_preserve_only_contiguous_authored_bytes() {
        let (actual, mut expected, source) = spans(&[3..5], 100..108);
        expected.extend([
            VizeSubSpan {
                gen_range: 100..103,
                src_range: source.start..source.start + 3,
            },
            VizeSubSpan {
                gen_range: 105..108,
                src_range: source.start + 3..source.end,
            },
        ]);
        assert_eq!(actual, expected);

        let (leading, mut expected, source) = spans(&[0..2], 100..108);
        expected.push(VizeSubSpan {
            gen_range: 102..108,
            src_range: source,
        });
        assert_eq!(
            leading, expected,
            "a leading gap must not emit an empty span"
        );

        let (trailing, mut expected, source) = spans(&[6..8], 100..108);
        expected.push(VizeSubSpan {
            gen_range: 100..106,
            src_range: source,
        });
        assert_eq!(
            trailing, expected,
            "a trailing gap must not emit an empty span"
        );
    }

    #[test]
    fn unauthenticated_synthetic_gaps_fall_back_to_key_spans() {
        for (gaps, generated) in [
            (vec![2..4, 3..5], 100..110),
            (vec![7..9], 100..108),
            (vec![3..5], 100..109),
        ] {
            let (actual, keys, _) = spans(&gaps, generated);
            assert_eq!(actual, keys);
        }
    }
}
