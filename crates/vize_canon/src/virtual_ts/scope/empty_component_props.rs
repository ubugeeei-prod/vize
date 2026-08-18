//! Required-prop checks for component usages without authored named values.

use vize_carton::CompactString;
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::append;
use vize_carton::cstr;
use vize_carton::profile;
use vize_croquis::Croquis;
use vize_croquis::croquis::ComponentUsage;

use crate::virtual_ts::component_reference::component_binding_reference;
use crate::virtual_ts::expressions::generate_component_prop_checks;
use crate::virtual_ts::expressions::{ComponentPropCheckContext, ComponentPropSource};
use crate::virtual_ts::types::{VirtualTsOptions, VizeMapping};

use super::component_event_navigation;
use super::component_prop_checker::has_inference_props;
use super::context::{ComponentPropsContext, VForPropsContext};

pub(super) fn is_empty_props_usage(usage: &ComponentUsage) -> bool {
    !has_inference_props(usage) && usage.spread_props.is_empty()
}

/// Keep empty and dynamic-name-only calls out of the template's control-flow
/// graph. Each uninvoked arrow is still type-checked and preserves the call's
/// TS2345 diagnostic and source mapping, while TypeScript analyzes every usage
/// in its own tiny control-flow graph instead of hitting TS2563 (#3527).
pub(super) fn generate_empty_root_checks(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &ComponentPropsContext<'_>,
    usages: &[(usize, &ComponentUsage)],
    closure_scope_ids: &FxHashSet<u32>,
) {
    let root_usages: Vec<_> = usages
        .iter()
        .copied()
        .filter(|(_, usage)| {
            !closure_scope_ids.contains(&usage.scope_id.as_u32()) && is_empty_props_usage(usage)
        })
        .collect();
    if root_usages.is_empty() {
        return;
    }

    let mut empty_context = EmptyChecksContext {
        ts,
        mappings,
        summary: ctx.summary,
        options: ctx.options,
        syntactic_type_only_imported_names: ctx.syntactic_type_only_imported_names,
        template_prop_names: ctx.template_prop_names,
        source_context: ctx.source_context(),
        indent: "  ",
    };
    generate_empty_checks(&mut empty_context, &root_usages);
}

struct EmptyChecksContext<'a, 'b> {
    ts: &'b mut String,
    mappings: &'b mut Vec<VizeMapping>,
    summary: &'a Croquis,
    options: &'a VirtualTsOptions,
    syntactic_type_only_imported_names: &'a FxHashSet<CompactString>,
    template_prop_names: &'a FxHashSet<String>,
    source_context: ComponentPropSource<'a>,
    indent: &'b str,
}

fn generate_empty_checks(
    ctx: &mut EmptyChecksContext<'_, '_>,
    usages: &[(usize, &ComponentUsage)],
) {
    let ts = &mut *ctx.ts;
    let mappings = &mut *ctx.mappings;
    let indent = ctx.indent;
    let arrow_indent = cstr!("{indent}  ");
    let body_indent = cstr!("{indent}    ");
    append!(*ts, "{indent}void [\n");
    for &(idx, usage) in usages {
        let component_ref = component_binding_reference(
            ctx.summary,
            ctx.options,
            ctx.syntactic_type_only_imported_names,
            usage.name.as_str(),
        );
        append!(*ts, "{arrow_indent}() => {{\n");
        let mut check_context = ComponentPropCheckContext::new(
            ts,
            mappings,
            ctx.template_prop_names,
            ctx.source_context,
            body_indent.as_str(),
        );
        profile!(
            "canon.virtual_ts.empty_component_prop_checks",
            generate_component_prop_checks(&mut check_context, usage, idx, component_ref.as_str())
        );
        append!(*ts, "{arrow_indent}}},\n");
    }
    append!(*ts, "{indent}];\n");
}

/// Emit ordinary checks directly in their closure and isolate only the empty
/// checks that would otherwise inflate that closure's control-flow graph.
pub(super) fn generate_scope_checks(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &VForPropsContext<'_>,
    scope_id: u32,
    indent: &str,
) {
    let Some(usages) = ctx.components_by_scope.get(&scope_id) else {
        return;
    };
    component_event_navigation::emit_scoped_event_references(ts, mappings, ctx, usages, indent);
    for &(idx, usage) in usages {
        if is_empty_props_usage(usage) {
            continue;
        }
        profile!("canon.virtual_ts.component_prop_checks", {
            let component_ref = component_binding_reference(
                ctx.summary,
                ctx.options,
                ctx.syntactic_type_only_imported_names,
                usage.name.as_str(),
            );
            let mut check_context = ComponentPropCheckContext::new(
                ts,
                mappings,
                ctx.template_prop_names,
                ctx.source_context,
                indent,
            );
            generate_component_prop_checks(&mut check_context, usage, idx, component_ref.as_str())
        });
    }
    let empty_usages: Vec<_> = usages
        .iter()
        .copied()
        .filter(|(_, usage)| is_empty_props_usage(usage))
        .collect();
    if !empty_usages.is_empty() {
        let mut empty_context = EmptyChecksContext {
            ts,
            mappings,
            summary: ctx.summary,
            options: ctx.options,
            syntactic_type_only_imported_names: ctx.syntactic_type_only_imported_names,
            template_prop_names: ctx.template_prop_names,
            source_context: ctx.source_context,
            indent,
        };
        generate_empty_checks(&mut empty_context, &empty_usages);
    }
}
