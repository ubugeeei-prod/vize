use vize_carton::{String, append, cstr};

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

/// Alias the authored default export and its instance type so the generated
/// component keeps the declarations the SFC itself wrote.
pub(super) fn emit_authored_component_aliases(ts: &mut String, preserve_authored_component: bool) {
    if !preserve_authored_component {
        return;
    }
    ts.push_str(
        "type __VizeAuthoredComponent = Awaited<ReturnType<typeof __setup>>[\"__default__\"];\n",
    );
    ts.push_str(
        "type __VizeAuthoredInstance = __VizeAuthoredComponent extends abstract new (...args: any[]) => infer __I ? __I : {};\n\n",
    );
}

pub(super) fn emit_default_export_declaration(
    ts: &mut String,
    emits_info: &EmitsInfo,
    generic_component_params: Option<(&str, &str)>,
    has_authored_default: bool,
    static_raw_props_ref: Option<&str>,
) {
    let emit_props_static = emits_info.static_emit_props_field();
    let event_map_static = emits_info.static_event_map_field();
    let authored_component = if has_authored_default {
        "__VizeAuthoredComponent & "
    } else {
        ""
    };
    // Keep canonical props on the component value itself. The normalized
    // generic constructor selects a return type from authored input, so a raw
    // identity buried only in that return does not invalidate parent template
    // checks after an editor changes a child's props. This optional static
    // marker creates the dependency edge without changing InstanceType or the
    // authored call-site contract (#4034).
    //
    // Keep the metadata intersection first. Non-generic components defer their
    // input contract behind `__VizeComponentInput`; putting that constructor
    // before the raw-props member lets TypeScript's incremental relation cache
    // settle only the first of multiple consumers of the same changed SFC.
    // Leading with the direct `Props` identity makes every dependent program
    // observe the edit while the last construct signature remains unchanged.
    let static_raw_props_field = static_raw_props_ref
        .map(|props_ref| cstr!("readonly __vizeRawProps?: {props_ref};"))
        .unwrap_or_default();
    if let Some((generic_decl, generic_names)) = generic_component_params {
        let emit_resolvers = emits_info.generic_emit_resolver_fields(generic_decl, generic_names);
        let event_map_separator = if emit_props_static.is_empty() || event_map_static.is_empty() {
            ""
        } else {
            " "
        };
        let emit_props_separator = if emit_resolvers.is_empty() { "" } else { " " };
        append!(
            *ts,
            "declare const __vize_component__: {{ __vizeCheck: <{generic_decl}>(props: Partial<Props<{generic_names}>> & Record<string, unknown>) => void; __vizeResolveProps?: <{generic_decl}>(props: Partial<Props<{generic_names}>> & Record<string, unknown>) => Props<{generic_names}>; {emit_props_static}{event_map_separator}{event_map_static}{emit_props_separator}{emit_resolvers} {static_raw_props_field} }} & {authored_component}__VizeGenericComponentConstructor & __VizeComponentConstructor & __VizeVueComponentOptions;\n",
        );
    } else if emits_info.has_emits_for_props {
        let event_map_separator = if event_map_static.is_empty() { "" } else { " " };
        append!(
            *ts,
            "declare const __vize_component__: {{ {emit_props_static}{event_map_separator}{event_map_static} {static_raw_props_field} }} & {authored_component}__VizeComponentConstructor & __VizeVueComponentOptions;\n",
        );
    } else if !static_raw_props_field.is_empty() {
        append!(
            *ts,
            "declare const __vize_component__: {{ {static_raw_props_field} }} & {authored_component}__VizeComponentConstructor & __VizeVueComponentOptions;\n",
        );
    } else {
        append!(
            *ts,
            "declare const __vize_component__: {authored_component}__VizeComponentConstructor & __VizeVueComponentOptions;\n",
        );
    }
}

pub(super) fn emit_component_default_export(ts: &mut String, component_name: Option<&str>) {
    let Some(component_name) = component_name else {
        ts.push_str("export default __vize_component__;\n");
        return;
    };
    let mut export_name = String::from(component_name);
    if module_scope_contains_identifier(ts, export_name.as_str()) {
        export_name.push_str("VueComponent");
        while module_scope_contains_identifier(ts, export_name.as_str()) {
            export_name.push('_');
        }
    }
    append!(
        *ts,
        "declare const {export_name}: typeof __vize_component__;\nexport default {export_name};\n",
    );
}

fn module_scope_contains_identifier(ts: &str, name: &str) -> bool {
    const SETUP: &str = "// ========== Setup Scope ==========";
    const AFTER_SETUP: &str = "// Invoke setup to verify types";
    let before_setup = ts.split_once(SETUP).map_or(ts, |(before, _)| before);
    let after_setup = ts.rsplit_once(AFTER_SETUP).map_or("", |(_, after)| after);
    [before_setup, after_setup]
        .into_iter()
        .any(|source| contains_identifier(source, name))
}

fn contains_identifier(source: &str, name: &str) -> bool {
    source.match_indices(name).any(|(start, _)| {
        let end = start + name.len();
        let boundary = |byte: u8| !byte.is_ascii_alphanumeric() && !matches!(byte, b'_' | b'$');
        source
            .as_bytes()
            .get(start.wrapping_sub(1))
            .is_none_or(|byte| boundary(*byte))
            && source
                .as_bytes()
                .get(end)
                .is_none_or(|byte| boundary(*byte))
    })
}
