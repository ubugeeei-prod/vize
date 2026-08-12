//! The object literal that reproduces a component usage's authored props.
//!
//! Split out of [`super::generic_props_call`] so the same literal can be
//! assembled twice: once as the mapped argument of the child's prop checker,
//! and once as the *unmapped* argument of the child's slot resolver, which only
//! exists so TypeScript instantiates the child's generic parameters from the
//! authored props (see `scope::slot_scope`). Keeping one builder is what makes
//! the two arguments identical, which is the whole premise of the slot type
//! matching the checked props.

use super::super::helpers::to_camel_case;
use super::super::types::{VizeMapping, VizeSubSpan};
use super::component_props::{
    ComponentPropSource, collect_generated_class_bindings, is_checkable_prop,
    merged_class_binding_value,
};
use super::prop_sources::{
    append_prop_value, generated_prop_value, prop_name_source_range, prop_value_source_range,
};
use super::spread_reserved_props::rewrite_reserved_spread_references;
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::append;
use vize_croquis::{
    ScopeId,
    croquis::{ComponentUsage, PassedProp, SpreadProp},
};

/// Append the `{ … }` props literal for `usage`, returning its generated range.
///
/// Entries follow template order, because Vue 3 resolves an overlapping key by
/// source order (last binding wins), exactly as an object literal does.
/// Spreading first unconditionally would check `1` for the `count` of
/// `:count="1" v-bind="bag"`, a key the runtime takes from `bag` instead.
pub(super) fn append_props_literal(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    usage: &ComponentUsage,
    template_prop_names: &FxHashSet<String>,
    source_context: ComponentPropSource<'_>,
    expr_indent: &str,
) -> std::ops::Range<usize> {
    let template_offset = source_context.offset;
    let literal_gen_start = ts.len();
    ts.push_str("{\n");

    let class_bindings = collect_generated_class_bindings(usage, template_prop_names);
    let merge_class_bindings = class_bindings.len() > 1;
    let mut emitted_merged_class = false;
    let mut spreads = usage.spread_props.iter().peekable();
    let mut open_named_group = false;
    for prop in &usage.props {
        if !is_checkable_prop(prop) {
            continue;
        }
        while let Some(spread) = spreads.next_if(|spread| spread.start < prop.start) {
            close_named_group(ts, &mut open_named_group);
            append_spread_entry(
                ts,
                mappings,
                spread,
                template_prop_names,
                usage.scope_id,
                source_context,
                expr_indent,
            );
        }

        let generated_value = if merge_class_bindings && prop.name.as_str() == "class" {
            if emitted_merged_class {
                continue;
            }
            emitted_merged_class = true;
            merged_class_binding_value(&class_bindings)
        } else {
            generated_prop_value(prop, template_prop_names)
        };
        let Some(mut generated_value) = generated_value else {
            continue;
        };
        let inline_callback = crate::virtual_ts::scope::is_inline_callback_prop(prop);
        if inline_callback {
            // The authored callback is emitted by `callback_prop_resolution`
            // for inference and by the mapped per-prop check for diagnostics.
            generated_value = String::from("undefined as any");
        }

        let (prop_src_start, prop_src_end) =
            if merge_class_bindings && prop.name.as_str() == "class" {
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
        // TypeScript reports TS2783 for `{ count: 1, ...bag }` when `bag` also
        // has `count`, while Vue accepts the authored last-wins binding order
        // and the pinned parity oracle excludes that synthetic warning.
        // Express the named prefix as a singleton spread when an object
        // `v-bind` still follows: `{ ...{ count: 1 }, ...bag }` has the same
        // inferred result and runtime order without manufacturing a
        // duplicate-property diagnostic. A trailing named prop stays direct,
        // so real duplicate named attributes are not hidden.
        //
        // Keep each named run in one singleton so authored duplicate props stay
        // in the same object literal and TypeScript still reports TS1117.
        let wrap_for_following_spread = spreads.peek().is_some();

        if wrap_for_following_spread {
            if open_named_group {
                ts.push_str(", ");
            } else {
                append!(*ts, "{expr_indent}  ...{{ ");
                open_named_group = true;
            }
        } else {
            close_named_group(ts, &mut open_named_group);
            append!(*ts, "{expr_indent}  ");
        }
        // Map the whole `"prop": value` entry (key through value) back to the
        // source attribute. TypeScript reports an assignability error for an
        // object-literal property at the property key, not the value, so a
        // value-only mapping would miss it and the diagnostic would be dropped.
        let entry_gen_start = ts.len();
        append!(*ts, "\"{camel_prop_name}\"");
        let key_gen_end = ts.len();
        ts.push_str(": ");
        let value_gen_range = append_prop_value(ts, generated_value.as_str());
        let entry_gen_end = ts.len();
        if !wrap_for_following_spread {
            ts.push_str(",\n");
        }
        // Without sub-spans the whole entry maps proportionally onto the whole
        // attribute, and `"prop": ` is one byte longer than `:prop="` — so an
        // error inside the value (TypeScript anchors a nested object-literal
        // mismatch at the offending key) landed one byte to the right of where
        // vue-tsc puts it (#3446). Mapping the key and the value separately
        // makes the value range verbatim, so offsets inside it are exact. An
        // inline callback has a synthetic `any` value here, so only its key is
        // mapped; the separately emitted authored callback owns its value span.
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
    for spread in spreads {
        close_named_group(ts, &mut open_named_group);
        append_spread_entry(
            ts,
            mappings,
            spread,
            template_prop_names,
            usage.scope_id,
            source_context,
            expr_indent,
        );
    }
    close_named_group(ts, &mut open_named_group);

    append!(*ts, "{expr_indent}}}");
    literal_gen_start..ts.len()
}

fn close_named_group(ts: &mut String, open: &mut bool) {
    if *open {
        ts.push_str(" },\n");
        *open = false;
    }
}

/// Emit one `...expr` entry of the props literal, mapped back to its own
/// directive so an error *inside* the expression lands on the authored bytes.
fn append_spread_entry(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    spread: &SpreadProp,
    template_prop_names: &FxHashSet<String>,
    usage_scope_id: ScopeId,
    source_context: ComponentPropSource<'_>,
    expr_indent: &str,
) {
    append!(*ts, "{expr_indent}  ...");
    let expression = spread.expression.as_str();
    let source_expression = spread_expression_source_range(source_context, spread);
    let (gen_range, sub_spans) = if let Some(rewritten) = rewrite_reserved_spread_references(
        expression,
        template_prop_names,
        source_context.scopes,
        usage_scope_id,
    ) {
        let gen_range = append_prop_value(ts, rewritten.code.as_str());
        let sub_spans = source_expression.map_or_else(Vec::new, |source| {
            rewritten
                .segments
                .into_iter()
                .map(|segment| VizeSubSpan {
                    gen_range: gen_range.start + segment.generated.start
                        ..gen_range.start + segment.generated.end,
                    src_range: source.start + segment.source.start
                        ..source.start + segment.source.end,
                })
                .collect()
        });
        (gen_range, sub_spans)
    } else {
        let gen_range = append_prop_value(ts, expression);
        let sub_spans = source_expression.map_or_else(Vec::new, |source| {
            vec![VizeSubSpan {
                gen_range: gen_range.clone(),
                src_range: source,
            }]
        });
        (gen_range, sub_spans)
    };
    ts.push_str(",\n");
    mappings.push(VizeMapping {
        gen_range,
        src_range: (source_context.offset + spread.start) as usize
            ..(source_context.offset + spread.end) as usize,
        sub_spans,
    });
}

fn spread_expression_source_range(
    source_context: ComponentPropSource<'_>,
    spread: &SpreadProp,
) -> Option<std::ops::Range<usize>> {
    let source = source_context.template?;
    let raw = source.get(spread.start as usize..spread.end as usize)?;
    let relative_start = raw.rfind(spread.expression.as_str())?;
    let start = source_context.offset as usize + spread.start as usize + relative_start;
    Some(start..start + spread.expression.len())
}

/// Sub-spans for one `"prop": value` entry: the key maps to the authored
/// attribute name, the value to the authored expression. Returns an empty list
/// unless both are known, since a partial list would silently drop whichever
/// half is missing.
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
