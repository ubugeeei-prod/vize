use vize_carton::{String, append, cstr};

use super::emits::EmitsInfo;
use super::generics::split_generic_params;

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

/// The child-side slot resolver a parent's `v-slot` scope calls to instantiate
/// this component's generic parameters from the authored props (#4147).
///
/// Only a generic component whose `Slots` alias actually takes those parameters
/// needs it: everywhere else the parent's structural `$slots` probe already
/// yields the exact declared slot map, and a resolver would be an unreferenced
/// widening of the public shape. The parameter mirrors `__vizeResolveProps`
/// exactly so both calls infer the same type arguments from the same literal.
/// Synthetic `= any` defaults are removed so an uninferable call argument falls
/// back to its constraint, exactly as it does through `vue-tsc`'s own generic
/// component signature. Authored defaults such as `U = T` stay intact because
/// they are part of that signature and can determine the slot payload even when
/// no prop directly mentions the parameter.
fn slot_resolver_field(generic_decl: &str, generic_names: &str, slots_is_generic: bool) -> String {
    if !slots_is_generic {
        return String::default();
    }
    let resolver_decl = strip_synthetic_any_defaults(generic_decl);
    cstr!(
        "__vizeResolveSlots?: <{resolver_decl}>(props: Partial<Props<{generic_names}>> & Record<string, unknown>) => Slots<{generic_names}>; "
    )
}

/// The prop parameter a parent's template calls on a generic child.
///
/// Generic SFCs need `Partial<Props<T>>` so authored props can infer `T` without
/// requiring every prop in the generic contract. What they must not need is an
/// unconditional string index: generated parents already know whether this
/// generated child can fall attributes through, so strict-template unknown
/// props should only be accepted by a real fallthrough target. `class` and
/// `style` stay universally public component attrs.
fn generic_check_props_param(generic_names: &str, fallthrough_props_ref: Option<&str>) -> String {
    let mut param =
        cstr!("Partial<Props<{generic_names}>> & {{ class?: unknown; style?: unknown }}");
    if let Some(fallthrough_ref) = fallthrough_props_ref {
        append!(
            param,
            " & {{ [K in keyof {fallthrough_ref}]?: unknown }} & {{ [K in `aria${{string}}`]?: unknown }} & {{ [K in `data${{string}}`]?: unknown }} & Partial<{{ [K in keyof {fallthrough_ref} & string as K extends `aria-${{infer Tail}}` ? `aria${{Capitalize<Tail>}}` : K extends `data-${{infer Tail}}` ? `data${{Capitalize<Tail>}}` : never]: unknown }}>"
        );
    }
    param
}

/// The same parameter list with generated `= any` defaults removed.
///
/// A *type alias* parameter needs the `= any` default so bare references stay
/// legal (#3065), but a call signature must not inherit those synthetic
/// defaults: when a call cannot infer an argument for a parameter, TypeScript
/// falls back to the default when there is one and to the constraint when there
/// is not.
fn strip_synthetic_any_defaults(generic_decl: &str) -> String {
    let mut stripped = String::default();
    for param in split_generic_params(generic_decl) {
        if !stripped.is_empty() {
            stripped.push_str(", ");
        }
        stripped.push_str(param_without_synthetic_any_default(param).as_str());
    }
    stripped
}

/// One parameter declaration with only its generated `= any` suffix removed.
/// `=>` inside a constraint never terminates the declaration.
fn param_without_synthetic_any_default(param: &str) -> String {
    let Some(default_start) = default_start(param) else {
        return param.trim().into();
    };
    let default = param[default_start + 1..].trim();
    if default == "any" {
        param[..default_start].trim().into()
    } else {
        param.trim().into()
    }
}

fn default_start(param: &str) -> Option<usize> {
    let bytes = param.as_bytes();
    let mut depth = 0i32;
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'<' => depth += 1,
            b'>' => depth -= 1,
            b'=' if bytes.get(i + 1) == Some(&b'>') => i += 1,
            b'=' if depth == 0 => return Some(i),
            _ => {}
        }
        i += 1;
    }
    None
}

pub(super) fn emit_default_export_declaration(
    ts: &mut String,
    emits_info: &EmitsInfo,
    generic_component_params: Option<(&str, &str, bool)>,
    has_authored_default: bool,
    static_raw_props_ref: Option<&str>,
    static_slots_ref: Option<&str>,
    fallthrough_props_ref: Option<&str>,
) {
    emit_vue_component_options_type(ts, generic_component_params.is_some());
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
    if let Some((generic_decl, generic_names, slots_is_generic)) = generic_component_params {
        let emit_resolvers = emits_info.generic_emit_resolver_fields(generic_decl, generic_names);
        let event_map_separator = if emit_props_static.is_empty() || event_map_static.is_empty() {
            ""
        } else {
            " "
        };
        let emit_props_separator = if emit_resolvers.is_empty() { "" } else { " " };
        let slot_resolver = slot_resolver_field(generic_decl, generic_names, slots_is_generic);
        let check_props_param = generic_check_props_param(generic_names, fallthrough_props_ref);
        append!(
            *ts,
            "declare const __vize_component__: {{ __vizeCheck: <{generic_decl}>(props: {check_props_param}) => void; __vizeResolveProps?: <{generic_decl}>(props: {check_props_param}) => Props<{generic_names}>; {slot_resolver}{emit_props_static}{event_map_separator}{event_map_static}{emit_props_separator}{emit_resolvers} {component_contract_fields} }} & {authored_component}__VizeGenericComponentConstructor & __VizeComponentConstructor & __VizeVueComponentOptions;\n",
        );
    } else if emits_info.has_emits_for_props {
        let event_map_separator = if event_map_static.is_empty() { "" } else { " " };
        append!(
            *ts,
            "declare const __vize_component__: {{ {emit_props_static}{event_map_separator}{event_map_static} {component_contract_fields} }} & {authored_component}__VizeComponentConstructor & __VizeVueComponentOptions;\n",
        );
    } else if !component_contract_fields.trim().is_empty() {
        append!(
            *ts,
            "declare const __vize_component__: {{ {component_contract_fields} }} & {authored_component}__VizeComponentConstructor & __VizeVueComponentOptions;\n",
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

#[cfg(test)]
mod tests {
    use super::strip_synthetic_any_defaults;

    #[test]
    fn synthetic_any_defaults_are_stripped_without_touching_constraints() {
        assert_eq!(
            strip_synthetic_any_defaults("T extends { id: string; } = any").as_str(),
            "T extends { id: string; }"
        );
        assert_eq!(strip_synthetic_any_defaults("T = any").as_str(), "T");
        assert_eq!(
            strip_synthetic_any_defaults("T extends (value: string) => void = any").as_str(),
            "T extends (value: string) => void"
        );
        assert_eq!(
            strip_synthetic_any_defaults("A extends Record<string, any> = any, B = A").as_str(),
            "A extends Record<string, any>, B = A"
        );
        assert_eq!(
            strip_synthetic_any_defaults("A = any, B extends A = any").as_str(),
            "A, B extends A"
        );
        assert_eq!(
            strip_synthetic_any_defaults("const T extends Tab").as_str(),
            "const T extends Tab"
        );
        assert_eq!(
            strip_synthetic_any_defaults("T = string").as_str(),
            "T = string"
        );
    }
}
