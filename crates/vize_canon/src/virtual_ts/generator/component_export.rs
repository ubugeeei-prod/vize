use crate::virtual_ts::types::AuthoredDefaultKind;
use vize_carton::{String, append};

use super::emits::EmitsInfo;
mod generic;
use generic::{generic_check_props_param, slot_resolver_field};

pub(super) struct GenericComponentContract<'a> {
    pub(super) declaration: &'a str,
    pub(super) names: &'a str,
    pub(super) slots_is_generic: bool,
    pub(super) public_type: &'static str,
    pub(super) props_type: &'static str,
}

/// The string index signature Vue's own component options carry, emitted ahead
/// of [`VUE_COMPONENT_OPTIONS_MEMBERS`] for non-generic components.
///
/// `ComponentOptionsBase` — and therefore every `DefineComponent` — extends
/// `LegacyOptions`, which declares:
///
/// ```ts
/// interface LegacyOptions<...> {
///   compatConfig?: CompatConfig;
///   [key: string]: any;
///   ...
/// }
/// ```
///
/// That is what makes `typeof SomeComponent` accept arbitrary members under
/// `vue-tsc`. Without it, the classic `ref<typeof Child | null>(null)` template
/// ref reports `TS2339` on every member of the child's expose surface (#4150).
/// Declared members still win over the index signature, so the `never` markers
/// keep excluding Fragment/Teleport/Suspense, and the *instance* type is
/// untouched: `InstanceType<typeof Child>` keeps reporting a genuinely absent
/// member exactly as `vue-tsc` does.
const COMPONENT_OPTIONS_INDEX_SIGNATURE: &str = "  [key: string]: any;\n";

/// Structural shape of the Vue component options object the SFC's default
/// export is intersected with, so template/`InstanceType` consumers see the
/// runtime option keys alongside the constructor.
const VUE_COMPONENT_OPTIONS_MEMBERS: &str = "  name?: string;
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

/// Emit the `__VizeVueComponentOptions` alias the default export references.
///
/// A generic SFC is *not* a `DefineComponent` for `vue-tsc`: it compiles to a
/// bare generic function component, so member access on its value type reports
/// `TS2339` for everything but `Function`'s own members. Only the non-generic
/// spelling therefore carries [`COMPONENT_OPTIONS_INDEX_SIGNATURE`], which
/// keeps generic components reporting exactly what `vue-tsc` reports.
fn emit_vue_component_options_type(ts: &mut String, has_generic_params: bool) {
    ts.push_str("type __VizeVueComponentOptions = {\n");
    if !has_generic_params {
        ts.push_str(COMPONENT_OPTIONS_INDEX_SIGNATURE);
    }
    ts.push_str(VUE_COMPONENT_OPTIONS_MEMBERS);
}

/// Alias the authored default export and its instance type so the generated
/// component keeps the declarations the SFC itself wrote.
pub(super) fn emit_authored_component_aliases(ts: &mut String, authored: AuthoredDefaultKind) {
    if authored == AuthoredDefaultKind::None {
        return;
    }
    ts.push_str(
        "type __VizeAuthoredComponent = Awaited<ReturnType<typeof __setup>>[\"__default__\"];\n",
    );
    if authored == AuthoredDefaultKind::Component {
        ts.push_str(
            "type __VizeAuthoredInstance = __VizeAuthoredComponent extends abstract new (...args: any[]) => infer __I ? __I : {};\n\n",
        );
    }
}

pub(super) fn emit_default_export_declaration(
    ts: &mut String,
    emits_info: &EmitsInfo,
    generic_component_params: Option<GenericComponentContract<'_>>,
    authored_default: AuthoredDefaultKind,
    static_raw_props_ref: Option<&str>,
    static_slots_ref: Option<&str>,
    fallthrough_props_ref: Option<&str>,
) {
    emit_vue_component_options_type(ts, generic_component_params.is_some());
    let emit_props_static = emits_info.static_emit_props_field();
    let event_map_static = emits_info.static_event_map_field();
    let authored_component = if authored_default == AuthoredDefaultKind::Component {
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
    let mut component_contract_fields = String::default();
    if static_raw_props_ref.is_some()
        || static_slots_ref.is_some()
        || fallthrough_props_ref.is_some()
    {
        component_contract_fields.push_str("readonly __vizeComponentMarker: true;");
    }
    if let Some(props_ref) = static_raw_props_ref {
        append!(
            component_contract_fields,
            " readonly __vizeRawProps?: {props_ref};"
        );
    }
    // `defineSlots` describes the callable payload for each slot; it does not
    // require a parent to provide every declared slot. Keep the private marker
    // partial so parent payload inference remains exact without turning an
    // omitted slot into a component-usage error.
    if let Some(slots_ref) = static_slots_ref {
        append!(
            component_contract_fields,
            " readonly __vizeSlots?: Partial<{slots_ref}>;"
        );
    }
    if let Some(fallthrough_ref) = fallthrough_props_ref {
        append!(
            component_contract_fields,
            " readonly __vizeHasFallthroughProps: true; readonly __vizeFallthroughProps?: {fallthrough_ref};"
        );
    }
    ts.push_str("declare const __vize_component__: ");
    if authored_default != AuthoredDefaultKind::None {
        // A normal script may export a primitive. Adding the generated Vue
        // constructor to that value would change its authored public type.
        ts.push_str("__VizeAuthoredComponent extends object ? ");
    }
    if let Some(GenericComponentContract {
        declaration: "",
        public_type: public_component_type,
        ..
    }) = generic_component_params
    {
        append!(
            *ts,
            "{{ {emit_props_static} {event_map_static} {component_contract_fields} }} & {authored_component}{public_component_type}"
        );
    } else if let Some(GenericComponentContract {
        declaration: generic_decl,
        names: generic_names,
        slots_is_generic,
        public_type: public_component_type,
        props_type,
    }) = generic_component_params
    {
        let emit_resolvers =
            emits_info.generic_emit_resolver_fields(generic_decl, generic_names, props_type);
        let event_map_separator = if emit_props_static.is_empty() || event_map_static.is_empty() {
            ""
        } else {
            " "
        };
        let emit_props_separator = if emit_resolvers.is_empty() { "" } else { " " };
        let slot_resolver =
            slot_resolver_field(generic_decl, generic_names, slots_is_generic, props_type);
        let check_props_param =
            generic_check_props_param(generic_names, fallthrough_props_ref, props_type);
        append!(
            *ts,
            "{{ __vizeCheck: <{generic_decl}>(props: {check_props_param}) => void; __vizeResolveProps?: <{generic_decl}>(props: {check_props_param}) => {props_type}<{generic_names}>; {slot_resolver}{emit_props_static}{event_map_separator}{event_map_static}{emit_props_separator}{emit_resolvers} {component_contract_fields} }} & {authored_component}{public_component_type}",
        );
    } else if emits_info.has_emits_for_props {
        let event_map_separator = if event_map_static.is_empty() { "" } else { " " };
        append!(
            *ts,
            "{{ {emit_props_static}{event_map_separator}{event_map_static} {component_contract_fields} }} & {authored_component}__VizeComponentConstructor & __VizeVueComponentOptions",
        );
    } else if !component_contract_fields.trim().is_empty() {
        append!(
            *ts,
            "{{ {component_contract_fields} }} & {authored_component}__VizeComponentConstructor & __VizeVueComponentOptions",
        );
    } else {
        append!(
            *ts,
            "{authored_component}__VizeComponentConstructor & __VizeVueComponentOptions",
        );
    }
    if authored_default != AuthoredDefaultKind::None {
        ts.push_str(" : __VizeAuthoredComponent");
    }
    ts.push_str(";\n");
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

#[cfg(test)]
#[path = "component_export_tests.rs"]
mod tests;
