use vize_carton::{String, append};
use vize_croquis::croquis::ComponentUsage;

use crate::virtual_ts::helpers::{to_camel_case, to_safe_identifier_fragment};

pub(super) fn is_inline_function_prop_value(value: &str) -> bool {
    let value = value.trim();
    value.contains("=>") || value.starts_with("function") || value.starts_with("async function")
}

pub(super) fn has_dynamic_props(usage: &ComponentUsage) -> bool {
    usage.props.iter().any(|prop| {
        prop.name.as_str() != "key"
            && prop.name.as_str() != "ref"
            && prop.value.is_some()
            && prop.is_dynamic
    })
}

pub(super) fn append_prop_checker_alias(
    ts: &mut String,
    usage: &ComponentUsage,
    component_type_name: &str,
    component_ref: &str,
    idx: usize,
) {
    append!(
        *ts,
        "  type __{component_type_name}_CheckProps_{idx} = {{\n",
    );
    for prop in &usage.props {
        if prop.name.as_str() == "key" || prop.name.as_str() == "ref" {
            continue;
        }
        if let Some(value) = prop.value.as_ref()
            && prop.is_dynamic
        {
            if !is_inline_function_prop_value(value.as_str()) {
                continue;
            }
            let camel_prop_name = to_camel_case(prop.name.as_str());
            let safe_prop_name = to_safe_identifier_fragment(prop.name.as_str());
            append!(
                *ts,
                "    \"{camel_prop_name}\": __{component_type_name}_{idx}_prop_{safe_prop_name};\n",
            );
        }
    }
    ts.push_str("  };\n");
    append!(
        *ts,
        "  type __{component_type_name}_Check_{idx} = __VizePropChecker<typeof {component_ref}, __{component_type_name}_CheckProps_{idx}>;\n",
    );
}
