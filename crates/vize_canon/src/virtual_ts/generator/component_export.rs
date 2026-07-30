use vize_carton::{String, append};

use super::emits::EmitsInfo;

/// Structural shape of the Vue component options object the SFC's default
/// export is intersected with, so template/`InstanceType` consumers see the
/// runtime option keys alongside the constructor.
pub(super) const VUE_COMPONENT_OPTIONS_TYPE: &str = "type __VizeVueComponentOptions = {
  name?: string;
  __name?: string;
  __file?: string;
  __vccOpts?: any;
  props?: any;
  emits?: any;
  slots?: any;
  setup?: any;
  render?: Function;
  components?: any;
  directives?: any;
  inheritAttrs?: boolean;
  compatConfig?: any;
  call?: (this: unknown, ...args: unknown[]) => never;
  __isFragment?: never;
  __isTeleport?: never;
  __isSuspense?: never;
  __defaults?: any;
  __vapor?: boolean;
  __multiRoot?: boolean;
  __isKeepAlive?: boolean;
  __isBuiltIn?: boolean;
};
";

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
            "declare const __vize_component__: __VizeGenericComponentConstructor & __VizeComponentConstructor & __VizeVueComponentOptions & {{ __vizeCheck: <{generic_decl}>(props: Partial<Props<{generic_names}>> & Record<string, unknown>) => void; __vizeResolveProps?: <{generic_decl}>(props: Partial<Props<{generic_names}>> & Record<string, unknown>) => Props<{generic_names}>; {emit_props_static}{emit_props_separator}{emit_props_resolver} }};\n",
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
