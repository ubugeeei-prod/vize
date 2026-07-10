use vize_carton::{String, append};

use super::emits::EmitsInfo;

pub(super) fn emit_default_export_declaration(
    ts: &mut String,
    emits_info: &EmitsInfo,
    generic_component_params: Option<(&str, &str)>,
) {
    let emit_props_static = emits_info.static_emit_props_field();
    if let Some((generic_decl, generic_names)) = generic_component_params {
        let emit_props_resolver =
            emits_info.generic_emit_props_resolver_field(generic_decl, generic_names);
        let emit_props_separator = if emit_props_resolver.is_empty() {
            ""
        } else {
            " "
        };
        append!(
            *ts,
            "declare const __vize_component__: __VizeGenericComponentConstructor & __VizeComponentConstructor & __VizeVueComponentOptions & {{ __vizeCheck: <{generic_decl}>(props: Partial<Props<{generic_names}>> & Record<string, unknown>) => void; {emit_props_static}{emit_props_separator}{emit_props_resolver} }};\n",
        );
    } else if emits_info.has_emits_for_props {
        append!(
            *ts,
            "declare const __vize_component__: __VizeComponentConstructor & __VizeVueComponentOptions & {{ {emit_props_static} }};\n",
        );
    } else {
        ts.push_str(
            "declare const __vize_component__: __VizeComponentConstructor & __VizeVueComponentOptions;\n",
        );
    }
}
