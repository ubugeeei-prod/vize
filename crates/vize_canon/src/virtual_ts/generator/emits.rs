use vize_carton::{FxHashSet, String, append, cstr};
use vize_croquis::Croquis;

use super::generics::module_alias_generic_suffix;
use super::setup_scope::macro_type_requires_setup_scope;
use crate::virtual_ts::{
    helpers::{EMIT_OVERLOAD_HELPERS, EMIT_PROPS_HELPER},
    macro_type_mappings::MacroTypeMappings,
    props::{add_generic_defaults, extract_generic_names, strip_const_modifiers},
};

#[path = "authored_events.rs"]
mod authored_events;
use authored_events::emit_authored_event_map;

/// Inner type of a macro's `<...>` type-argument text.
fn inner_type_of(type_args: &str) -> &str {
    type_args
        .strip_prefix('<')
        .and_then(|s| s.strip_suffix('>'))
        .unwrap_or(type_args)
}

/// Emit the module-scope `export type Slots` alias. When the slots type from
/// `defineSlots` references an SFC generic parameter, the alias re-declares
/// the parameters (with safe defaults) so declaration emit resolves them
/// (#3065).
/// Returns whether the alias re-declared the SFC's type parameters, so the
/// generic component constructor can instantiate it instead of falling back to
/// the declared defaults (#3354).
pub(super) fn emit_slots_type(
    ts: &mut String,
    summary: &Croquis,
    generic_injection: Option<&(String, Vec<String>)>,
) -> bool {
    let slots_type_args = summary
        .macros
        .define_slots()
        .and_then(|m| m.type_args.as_ref());
    if let Some(type_args) = slots_type_args {
        let inner_type = inner_type_of(type_args);
        let suffix = module_alias_generic_suffix(generic_injection, inner_type);
        append!(*ts, "export type Slots{suffix} = {inner_type};\n");
        !suffix.is_empty()
    } else {
        ts.push_str("export type Slots = {};\n");
        false
    }
}

/// Emit the module-scope `export type Exposed` alias (for `InstanceType` and
/// `useTemplateRef`); returns whether the component exposes anything. A typed
/// `defineExpose` referencing an SFC generic parameter re-declares the
/// parameters just like `Slots` (#3065).
///
/// The second flag reports whether the alias re-declared those parameters, so
/// the generic component constructor can instantiate it (#3354).
pub(super) fn emit_exposed_type(
    ts: &mut String,
    summary: &Croquis,
    generic_injection: Option<&(String, Vec<String>)>,
) -> (bool, bool) {
    let Some(expose) = summary.macros.define_expose() else {
        return (false, false);
    };
    if let Some(ref type_args) = expose.type_args {
        let inner_type = inner_type_of(type_args);
        let suffix = module_alias_generic_suffix(generic_injection, inner_type);
        append!(*ts, "export type Exposed{suffix} = {inner_type};\n");
        (true, !suffix.is_empty())
    } else if expose.runtime_args.is_some() {
        // Runtime args are returned from __setup() to keep them in scope.
        // Use Awaited<ReturnType<...>> to handle both sync and async setup.
        ts.push_str(
            "export type Exposed = Awaited<ReturnType<typeof __setup>>[\"__vize_exposed\"];\n",
        );
        (true, false)
    } else {
        (false, false)
    }
}

pub(super) struct EmitsInfo {
    pub(super) has_emits_for_props: bool,
    has_runtime_emits: bool,
    has_generic_emits: bool,
    generic_event_map_decl: String,
    generic_event_map_names: String,
    preserve_event_navigation: bool,
}

impl EmitsInfo {
    /// Whether the `Emits` alias re-declared the SFC's type parameters, so the
    /// generic component constructor must instantiate it (#3354).
    pub(super) fn has_generic_emits(&self) -> bool {
        self.has_generic_emits
    }

    pub(super) fn static_emit_props_field(&self) -> &'static str {
        if self.has_emits_for_props {
            "__vizeEmitProps?: __VizeStaticEmitProps;"
        } else {
            ""
        }
    }

    pub(super) fn static_event_map_field(&self) -> &'static str {
        if self.has_emits_for_props && self.preserve_event_navigation {
            "__vizeRawEmits?: __VizeAuthoredEventMap; __vizeEventMap?: __VizeStaticEventMap;"
        } else {
            ""
        }
    }

    pub(super) fn generic_emit_resolver_fields(
        &self,
        generic_decl: &str,
        generic_names: &str,
    ) -> String {
        let mut field = String::default();
        if self.has_emits_for_props && self.has_generic_emits {
            append!(
                field,
                "__vizeResolveEmitProps?: <{generic_decl}>(props: Partial<Props<{generic_names}>> & Record<string, unknown>) => __EmitProps<Emits<{generic_names}>>;"
            );
        }
        if !self.generic_event_map_decl.is_empty() {
            if !field.is_empty() {
                field.push(' ');
            }
            append!(
                field,
                "__vizeResolveEvents?: <{generic_decl}>(props: Partial<Props<{generic_names}>> & Record<string, unknown>) => __VizeAuthoredEventMap<{generic_names}>;"
            );
        }
        field
    }
}

/// The `update:` emit payload for a model.
///
/// An optional model without a default holds `T | undefined` (that is its
/// `ModelRef` type), and the update event can carry that `undefined` back —
/// vue-tsc's synthesized listener accepts exactly `T | undefined`, so a
/// handler typed for bare `T` must be rejected (#3904). A required model, a
/// model with a default, and an untyped model keep the bare payload. The base
/// is parenthesized so a function-typed model does not absorb the union into
/// its return type.
fn model_update_payload(model: &vize_croquis::macros::ModelDefinition) -> String {
    let base = model.model_type.as_deref().unwrap_or("unknown");
    if model.required || model.default_value.is_some() || base == "unknown" || base == "any" {
        String::from(base)
    } else {
        cstr!("({base}) | undefined")
    }
}

pub(super) fn emit_emits_type(
    ts: &mut String,
    summary: &Croquis,
    mut mappings: MacroTypeMappings<'_>,
    preserve_event_navigation: bool,
    generic_param: Option<&str>,
    has_runtime_emits: bool,
) -> EmitsInfo {
    let generated_start = ts.len();
    let emits_already_defined = summary
        .type_exports
        .iter()
        .any(|te| te.name.as_str() == "Emits");
    let define_emits_type_args = summary
        .macros
        .define_emits()
        .and_then(|call| call.type_args.as_ref());
    let models = summary.macros.models();
    let has_model_emits = !models.is_empty();
    let has_macro_emits = !summary.macros.emits().is_empty();
    let has_emits_for_props = emits_already_defined
        || define_emits_type_args.is_some()
        || has_runtime_emits
        || has_macro_emits
        || has_model_emits;
    let emits_generic_decl = generic_param
        .filter(|_| !emits_already_defined)
        .filter(|_| define_emits_type_args.is_some() || has_macro_emits || has_model_emits)
        .map(|generic| strip_const_modifiers(&add_generic_defaults(generic)));
    let emits_generic_suffix = emits_generic_decl
        .as_ref()
        .map(|generic| cstr!("<{generic}>"))
        .unwrap_or_default();
    let generic_event_map_decl = if preserve_event_navigation && !has_runtime_emits {
        emits_generic_decl.clone().unwrap_or_default()
    } else {
        String::default()
    };
    let generic_event_map_names = generic_param
        .filter(|_| !generic_event_map_decl.is_empty())
        .map(extract_generic_names)
        .unwrap_or_default();

    if !emits_already_defined {
        if let Some(type_args) = define_emits_type_args {
            let inner_type = inner_type_of(type_args);
            if has_model_emits {
                append!(
                    *ts,
                    "export type Emits{emits_generic_suffix} = {inner_type} & {{\n"
                );
                for model in models {
                    let name = model.name.as_str();
                    let payload = model_update_payload(model);
                    append!(*ts, "  \"update:{name}\": [value: {payload}];\n");
                }
                ts.push_str("};\n");
            } else {
                append!(
                    *ts,
                    "export type Emits{emits_generic_suffix} = {inner_type};\n"
                );
            }
        } else if has_runtime_emits {
            append!(
                *ts,
                "export type Emits{emits_generic_suffix} = Awaited<ReturnType<typeof __setup>>[\"__vize_emits\"]",
            );
            for model in models {
                let name = model.name.as_str();
                let payload = model_update_payload(model);
                append!(
                    *ts,
                    " & ((event: \"update:{name}\", value: {payload}) => void)"
                );
            }
            ts.push_str(";\n");
        } else if has_macro_emits || has_model_emits {
            append!(*ts, "export type Emits{emits_generic_suffix} = {{\n");
            let mut emitted_names: FxHashSet<String> = FxHashSet::default();
            for emit in summary.macros.emits() {
                let payload = emit.payload_type.as_deref().unwrap_or("any[]");
                append!(*ts, "  \"{}\": {payload};\n", emit.name);
                emitted_names.insert(emit.name.as_str().into());
            }
            for model in models {
                let event_name = cstr!("update:{}", model.name);
                if emitted_names.contains(event_name.as_str()) {
                    continue;
                }
                let payload = model_update_payload(model);
                append!(*ts, "  \"{event_name}\": [value: {payload}];\n");
            }
            ts.push_str("};\n");
        } else {
            ts.push_str("export type Emits = {};\n");
        }
    }

    // The `Emits` alias lives at module scope, so mapping it back onto the
    // authored macro only makes sense when the authored type resolves from
    // there. `defineEmits<Emits<typeof state>>` and friends read setup-scope
    // names the alias cannot see, and mapping that region would report the
    // synthetic "cannot find name" on valid SFC source (#4074).
    let emits_type_is_module_scoped = define_emits_type_args
        .is_some_and(|type_args| !macro_type_requires_setup_scope(summary, type_args));
    if summary.macros.emits().is_empty() && emits_type_is_module_scoped {
        mappings.map_exported_type(ts, generated_start, summary.macros.define_emits(), "Emits");
    }
    if preserve_event_navigation && has_emits_for_props {
        emit_authored_event_map(
            ts,
            summary,
            &mut mappings,
            !emits_already_defined,
            emits_generic_decl.as_deref().filter(|_| !has_runtime_emits),
            generic_event_map_names.as_str(),
        );
    }
    EmitsInfo {
        has_emits_for_props,
        has_runtime_emits,
        has_generic_emits: emits_generic_decl.is_some(),
        generic_event_map_decl,
        generic_event_map_names,
        preserve_event_navigation,
    }
}

pub(super) fn emit_emit_props_helper(
    ts: &mut String,
    info: &EmitsInfo,
    hoist_shared_preamble: bool,
) {
    if !info.has_emits_for_props {
        return;
    }
    if !hoist_shared_preamble {
        ts.push_str(EMIT_OVERLOAD_HELPERS);
    }
    ts.push_str(EMIT_PROPS_HELPER);
    ts.push('\n');
    if info.has_runtime_emits {
        if info.preserve_event_navigation {
            ts.push_str("type __VizeStaticEventMap = __EmitOptions<Awaited<ReturnType<typeof __setup>>[\"__vize_emit_options\"]>;\n");
        }
        ts.push_str("type __VizeStaticEmitProps = __EmitProps<Awaited<ReturnType<typeof __setup>>[\"__vize_emit_options\"]>;\n\n");
    } else {
        if info.preserve_event_navigation {
            if info.generic_event_map_decl.is_empty() {
                ts.push_str("type __VizeStaticEventMap = __EmitOptions<Emits>;\n");
            } else {
                append!(
                    *ts,
                    "type __VizeStaticEventMap<{}> = __EmitOptions<Emits<{}>>;\n",
                    info.generic_event_map_decl,
                    info.generic_event_map_names
                );
            }
        }
        ts.push_str("type __VizeStaticEmitProps = __EmitProps<Emits>;\n\n");
    }
}
