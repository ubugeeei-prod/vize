use vize_carton::{FxHashSet, String, append, cstr};
use vize_croquis::Croquis;

use super::setup_scope::macro_type_requires_setup_scope;
use crate::virtual_ts::{
    helpers::{EMIT_OVERLOAD_HELPERS, EMIT_PROPS_HELPER, push_ts_string_literal},
    macro_type_mappings::MacroTypeMappings,
    props::{add_generic_defaults, extract_generic_names, strip_const_modifiers},
};

#[path = "authored_events.rs"]
mod authored_events;
use authored_events::emit_authored_event_map;

/// Inner type of a macro's `<...>` type-argument text.
pub(super) fn inner_type_of(type_args: &str) -> &str {
    type_args
        .strip_prefix('<')
        .and_then(|s| s.strip_suffix('>'))
        .unwrap_or(type_args)
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
        props_type: &str,
    ) -> String {
        let mut field = String::default();
        if self.has_emits_for_props && self.has_generic_emits {
            append!(
                field,
                "__vizeResolveEmitProps?: <{generic_decl}>(props: Partial<{props_type}<{generic_names}>> & Record<string, unknown>) => __EmitProps<Emits<{generic_names}>>;"
            );
        }
        if !self.generic_event_map_decl.is_empty() {
            if !field.is_empty() {
                field.push(' ');
            }
            append!(
                field,
                "__vizeResolveEvents?: <{generic_decl}>(props: Partial<{props_type}<{generic_names}>> & Record<string, unknown>) => __VizeAuthoredEventMap<{generic_names}>;"
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
fn model_update_payload(
    summary: &Croquis,
    model: &vize_croquis::macros::ModelDefinition,
) -> String {
    let base = crate::virtual_ts::model_types::model_value_type(summary, model);
    if model.required || model.default_value.is_some() || base == "unknown" || base == "any" {
        base
    } else {
        cstr!("({base}) | undefined")
    }
}

fn push_model_update_event_literal(ts: &mut String, model_name: &str) {
    let event_name = cstr!("update:{model_name}");
    push_ts_string_literal(ts, event_name.as_str());
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
                    "export type Emits{emits_generic_suffix} = __EmitFn<{inner_type}> & __EmitFn<{{\n"
                );
                for model in models {
                    let name = model.name.as_str();
                    let payload = model_update_payload(summary, model);
                    ts.push_str("  ");
                    push_model_update_event_literal(ts, name);
                    append!(*ts, ": [value: {payload}];\n");
                }
                ts.push_str("}>;\n");
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
                let payload = model_update_payload(summary, model);
                ts.push_str(" & ((event: ");
                push_model_update_event_literal(ts, name);
                append!(*ts, ", value: {payload}) => void)");
            }
            ts.push_str(";\n");
        } else if has_macro_emits || has_model_emits {
            append!(*ts, "export type Emits{emits_generic_suffix} = {{\n");
            let mut emitted_names: FxHashSet<String> = FxHashSet::default();
            for emit in summary.macros.emits() {
                let payload = emit.payload_type.as_deref().unwrap_or("any[]");
                ts.push_str("  ");
                push_ts_string_literal(ts, emit.name.as_str());
                append!(*ts, ": {payload};\n");
                emitted_names.insert(emit.name.as_str().into());
            }
            for model in models {
                let event_name = cstr!("update:{}", model.name);
                if emitted_names.contains(event_name.as_str()) {
                    continue;
                }
                let payload = model_update_payload(summary, model);
                ts.push_str("  ");
                push_ts_string_literal(ts, event_name.as_str());
                append!(*ts, ": [value: {payload}];\n");
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
    event_inference: bool,
) {
    if !info.has_emits_for_props && !event_inference {
        return;
    }
    if !hoist_shared_preamble {
        ts.push_str(EMIT_OVERLOAD_HELPERS);
    }
    ts.push_str(EMIT_PROPS_HELPER);
    ts.push('\n');
    if !info.has_emits_for_props {
        return;
    }
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
