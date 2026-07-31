//! Required-prop checks for component usages without authored named values.

use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::profile;
use vize_croquis::croquis::ComponentUsage;

use crate::virtual_ts::expressions::generate_component_prop_checks;
use crate::virtual_ts::types::VizeMapping;

use super::component_prop_checker::has_inference_props;
use super::context::ComponentPropsContext;

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

    ts.push_str("  void [\n");
    for (idx, usage) in root_usages {
        ts.push_str("    () => {\n");
        profile!(
            "canon.virtual_ts.empty_component_prop_checks",
            generate_component_prop_checks(
                ts,
                mappings,
                usage,
                idx,
                ctx.template_prop_names,
                ctx.source_context(),
                "      "
            )
        );
        ts.push_str("    },\n");
    }
    ts.push_str("  ];\n");
}
