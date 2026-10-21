//! The object literal that reproduces a component usage's authored props.
//!
//! Split out of [`super::generic_props_call`] so the same literal can be
//! assembled for the mapped prop checker and listener inference, and for the
//! unmapped slot resolver that instantiates the child's generic parameters
//! from the authored props (see `scope::slot_scope`). One builder keeps their
//! authored ordering and spread semantics identical.

use super::super::types::{VizeMapping, VizeSubSpan};
use super::component_props::{
    ComponentPropSource, collect_generated_class_bindings, collect_generated_named_bindings,
    is_checkable_prop, merged_class_binding_value,
};
use super::prop_sources::append_prop_value;
use super::spread_reserved_props::rewrite_reserved_spread_references;
use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
use prop_entry::append_prop_entry;
use vize_carton::{FxHashSet, String, append};
use vize_croquis::{
    ScopeChain, ScopeId,
    croquis::{ComponentUsage, PassedProp, SpreadProp},
};

mod prop_entry;
pub(crate) use prop_entry::entry_sub_spans as prop_entry_sub_spans;

/// An inference-only argument uses the same authored ordering and lexical
/// spread binding authority as the mapped prop checker and slot resolver.
pub(crate) fn append_inference_props_literal(
    ts: &mut String,
    usage: &ComponentUsage,
    template_binding_access: &TemplateBindingAccess,
    source_context: ComponentPropSource<'_>,
    expr_indent: &str,
) {
    append_props_literal(
        ts,
        &mut Vec::new(),
        usage,
        template_binding_access,
        source_context,
        expr_indent,
    );
}

/// Collect the simple named props used by inference-only probes while folding
/// Vue's repeatable `class` and `style` bindings into one object entry.
///
/// These probes intentionally preserve their legacy shape instead of using the
/// mapped prop literal: the authored prop checker remains the diagnostic owner.
pub(crate) fn collect_inference_probe_props<'a>(
    usage: &'a ComponentUsage,
    template_binding_access: &TemplateBindingAccess,
) -> Vec<(&'a PassedProp, String)> {
    let class_bindings = collect_generated_class_bindings(usage, template_binding_access);
    let style_bindings = collect_generated_named_bindings(usage, template_binding_access, "style");
    let mut emitted_class = false;
    let mut emitted_style = false;
    let mut props = Vec::new();

    for prop in &usage.props {
        if prop.name_is_dynamic || matches!(prop.name.as_str(), "key" | "ref") {
            continue;
        }
        let bindings = match prop.name.as_str() {
            "class" if class_bindings.len() > 1 => Some((&class_bindings, &mut emitted_class)),
            "style" if style_bindings.len() > 1 => Some((&style_bindings, &mut emitted_style)),
            _ => None,
        };
        let value = if let Some((bindings, emitted)) = bindings {
            if *emitted {
                continue;
            }
            *emitted = true;
            merged_class_binding_value(bindings.as_slice(), None)
        } else {
            super::prop_sources::generated_prop_value(prop, template_binding_access)
        };
        if let Some(value) = value {
            props.push((prop, value));
        }
    }
    props
}

/// Append the `{ … }` props literal for `usage`, returning its generated range.
///
/// Entries follow template order, because Vue 3 resolves an overlapping key by
/// source order (last binding wins), exactly as an object literal does.
/// Spreading first unconditionally would check `1` for the `count` of
/// `:count="1" v-bind="bag"`, a key the runtime takes from `bag` instead.
pub(crate) fn append_props_literal(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    usage: &ComponentUsage,
    template_binding_access: &TemplateBindingAccess,
    source_context: ComponentPropSource<'_>,
    expr_indent: &str,
) -> std::ops::Range<usize> {
    let literal_gen_start = ts.len();
    ts.push_str("{\n");

    // Listeners are props at runtime. Their separately checked handlers must
    // satisfy required `onFoo` keys without treating unknown listeners as
    // unknown props (that is controlled by checkUnknownEvents). Emit these
    // witnesses first so they cannot erase any authored prop value checks.
    for event in &usage.events {
        if !event.name_is_dynamic && !event.name.is_empty() {
            let handler = vize_carton::capitalize(&vize_carton::camelize(event.name.as_str()));
            append!(*ts, "{expr_indent}  ...{{}} as {{ ");
            super::super::helpers::push_ts_string_literal(ts, &vize_carton::cstr!("on{handler}"));
            ts.push_str(": never },\n");
        }
    }

    let class_bindings = collect_generated_class_bindings(usage, template_binding_access);
    let merge_class_bindings = class_bindings.len() > 1;
    let mut emitted_merged_class = false;
    let style_bindings = collect_generated_named_bindings(usage, template_binding_access, "style");
    let merge_style_bindings = style_bindings.len() > 1;
    let mut emitted_merged_style = false;
    let mut spreads = usage.spread_props.iter().peekable();
    let mut authored_named_keys = FxHashSet::default();
    for prop in &usage.props {
        if !is_checkable_prop(prop) {
            continue;
        }
        while let Some(spread) = spreads.next_if(|pending| pending.start < prop.start) {
            append_spread_entry(
                ts,
                mappings,
                spread,
                template_binding_access,
                usage.scope_id,
                source_context,
                expr_indent,
            );
        }

        let (merge_bindings, bindings, emitted_merged) = if prop.name.as_str() == "style" {
            (
                merge_style_bindings,
                style_bindings.as_slice(),
                &mut emitted_merged_style,
            )
        } else {
            (
                merge_class_bindings,
                class_bindings.as_slice(),
                &mut emitted_merged_class,
            )
        };
        append_prop_entry(
            ts,
            mappings,
            prop,
            template_binding_access,
            source_context,
            expr_indent,
            merge_bindings,
            bindings,
            emitted_merged,
            spreads.peek().is_some(),
            &mut authored_named_keys,
        );
    }
    for spread in spreads {
        append_spread_entry(
            ts,
            mappings,
            spread,
            template_binding_access,
            usage.scope_id,
            source_context,
            expr_indent,
        );
    }

    append!(*ts, "{expr_indent}}}");
    literal_gen_start..ts.len()
}

/// Emit one `...expr` entry of the props literal, mapped back to its own
/// directive so an error *inside* the expression lands on the authored bytes.
fn append_spread_entry(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    spread: &SpreadProp,
    template_binding_access: &TemplateBindingAccess,
    usage_scope_id: ScopeId,
    source_context: ComponentPropSource<'_>,
    expr_indent: &str,
) {
    append!(*ts, "{expr_indent}  ...");
    let (gen_range, sub_spans) = append_spread_value(
        ts,
        spread.expression.as_str(),
        spread_expression_source_range(source_context, spread),
        template_binding_access,
        source_context.scopes,
        usage_scope_id,
    );
    ts.push_str(",\n");
    mappings.push(VizeMapping {
        gen_range,
        src_range: (source_context.offset + spread.start) as usize
            ..(source_context.offset + spread.end) as usize,
        sub_spans,
    });
}

/// Emit a spread expression with reserved prop names (`as`, `in`, ...)
/// read through their template bindings, plus the sub spans that map it back
/// to the authored expression.
pub(crate) fn append_spread_value(
    ts: &mut String,
    expression: &str,
    source_expression: Option<std::ops::Range<usize>>,
    template_binding_access: &TemplateBindingAccess,
    scopes: &ScopeChain,
    usage_scope_id: ScopeId,
) -> (std::ops::Range<usize>, Vec<VizeSubSpan>) {
    if let Some(rewritten) = rewrite_reserved_spread_references(
        expression,
        template_binding_access,
        scopes,
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
        return (gen_range, sub_spans);
    }
    let gen_range = append_prop_value(ts, expression);
    let sub_spans = source_expression.map_or_else(Vec::new, |source| {
        vec![VizeSubSpan {
            gen_range: gen_range.clone(),
            src_range: source,
        }]
    });
    (gen_range, sub_spans)
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
