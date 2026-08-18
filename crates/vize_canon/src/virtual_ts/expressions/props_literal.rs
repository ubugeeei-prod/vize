//! The object literal that reproduces a component usage's authored props.
//!
//! Split out of [`super::generic_props_call`] so the same literal can be
//! assembled twice: once as the mapped argument of the child's prop checker,
//! and once as the *unmapped* argument of the child's slot resolver, which only
//! exists so TypeScript instantiates the child's generic parameters from the
//! authored props (see `scope::slot_scope`). Keeping one builder is what makes
//! the two arguments identical, which is the whole premise of the slot type
//! matching the checked props.

use super::super::types::{VizeMapping, VizeSubSpan};
use super::component_props::{
    ComponentPropSource, collect_generated_class_bindings, is_checkable_prop,
};
use super::prop_sources::append_prop_value;
use super::spread_reserved_props::rewrite_reserved_spread_references;
use prop_entry::append_prop_entry;
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::append;
use vize_croquis::{
    ScopeId,
    croquis::{ComponentUsage, SpreadProp},
};

mod prop_entry;

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
        while spreads
            .peek()
            .is_some_and(|pending| pending.start < prop.start)
        {
            close_named_group(ts, &mut open_named_group);
            append_spread_entry(
                ts,
                mappings,
                spreads.next().expect("pending spread exists"),
                template_prop_names,
                usage.scope_id,
                source_context,
                expr_indent,
            );
        }

        append_prop_entry(
            ts,
            mappings,
            prop,
            template_prop_names,
            source_context,
            expr_indent,
            merge_class_bindings,
            &class_bindings,
            &mut emitted_merged_class,
            spreads.peek().is_some(),
            &mut open_named_group,
        );
    }
    while spreads.peek().is_some() {
        close_named_group(ts, &mut open_named_group);
        append_spread_entry(
            ts,
            mappings,
            spreads.next().expect("pending spread exists"),
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

pub(super) fn close_named_group(ts: &mut String, open: &mut bool) {
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
