use vize_carton::{String, cstr};
use vize_croquis::Croquis;
use vize_l0::ts;
use vize_l0::ts_emit::TsString;

use super::setup_scope::{define_emits_type_requires_setup_scope, macro_type_requires_setup_scope};
use crate::virtual_ts::{
    generated_names as names,
    generic_parameters::GenericParameters,
    helpers::{EMIT_OVERLOAD_HELPERS, EMIT_PROPS_HELPER},
    macro_type_mappings::MacroTypeMappings,
};

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
    has_model_emits: bool,
    has_generic_emits: bool,
    generic_event_map_decl: String,
    generic_event_map_names: String,
    preserve_event_navigation: bool,
    /// The template runs inside the SFC's generic setup signature.
    generic_component: bool,
}

impl EmitsInfo {
    /// Whether the `Emits` alias re-declared the SFC's type parameters, so the
    /// generic component constructor must instantiate it (#3354).
    pub(super) fn has_generic_emits(&self) -> bool {
        self.has_generic_emits
    }

    pub(super) fn static_emit_props_field(&self) -> &'static str {
        if self.has_emits_for_props {
            "__vizeEmitProps?: $StaticEmitProps;"
        } else {
            ""
        }
    }

    pub(super) fn static_event_map_field(&self) -> String {
        if !self.has_emits_for_props || !self.preserve_event_navigation {
            return String::default();
        }
        let (authored, events) = (
            names::declared("$AuthoredEventMap"),
            names::declared("$StaticEventMap"),
        );
        if authored == "$AuthoredEventMap" && events == "$StaticEventMap" {
            return String::const_new(
                "__vizeRawEmits?: $AuthoredEventMap; __vizeEventMap?: $StaticEventMap;",
            );
        }
        cstr!("__vizeRawEmits?: {authored}; __vizeEventMap?: {events};")
    }

    pub(super) fn emit(
        &self,
        ts: &mut crate::virtual_ts::TextBuf,
        summary: &Croquis,
        forwards_roots: bool,
        (legacy_vue2, hoist_shared_preamble): (bool, bool),
    ) {
        let event_inference = crate::virtual_ts::scope::emit_event_inference_helpers(
            ts,
            summary,
            forwards_roots,
            legacy_vue2,
            hoist_shared_preamble,
        );
        crate::virtual_ts::scope::emit_gate_aliases(
            ts,
            summary,
            self.generic_component,
            legacy_vue2,
            hoist_shared_preamble,
        );
        emit_emit_props_helper(ts, summary, self, hoist_shared_preamble, event_inference);
    }

    pub(super) fn generic_emit_resolver_fields(
        &self,
        generic_decl: &str,
        generic_names: &str,
        props_type: &str,
        hoist_shared_preamble: bool,
    ) -> String {
        let mut field = String::default();
        let (partial, record) = (names::lib("Partial"), names::lib("Record"));
        if self.has_emits_for_props && self.has_generic_emits {
            let emit_props = emit_props_ref(hoist_shared_preamble);
            let emits = names::declared("Emits");
            ts! { &mut field =>
                __vizeResolveEmitProps?: <#generic_decl>(props: #partial<#props_type<#generic_names>> & #record<string, unknown>) => #emit_props<#emits<#generic_names>>;
            }
        }
        if !self.generic_event_map_decl.is_empty() {
            if !field.is_empty() {
                field.push(' ');
            }
            let authored = names::declared("$AuthoredEventMap");
            ts! { &mut field =>
                __vizeResolveEvents?: <#generic_decl>(props: #partial<#props_type<#generic_names>> & #record<string, unknown>) => #authored<#generic_names>;
            }
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
    generic_parameters: Option<&GenericParameters>,
    model: &vize_croquis::macros::ModelDefinition,
) -> String {
    let base = crate::virtual_ts::model_types::model_value_type(generic_parameters, model);
    if model.required || model.default_value.is_some() || base == "unknown" || base == "any" {
        base
    } else {
        cstr!("({base}) | undefined")
    }
}

pub(super) fn template_model_emit_type(
    summary: &Croquis,
    generic_parameters: Option<&GenericParameters>,
) -> Option<String> {
    if summary.macros.models().is_empty() {
        return None;
    }
    let mut ts = String::default();
    ts! { &mut ts =>
        __EmitFn<{
            #(model in summary.macros.models() =>
                #TsString(cstr!("update:{}", model.name).as_str()): [value: #(model_update_payload(generic_parameters, model))];
            )*
        }>
    }
    Some(ts)
}

pub(super) fn emit_emits_type(
    ts: &mut crate::virtual_ts::TextBuf,
    summary: &Croquis,
    mut mappings: MacroTypeMappings<'_>,
    preserve_event_navigation: bool,
    generic_param: Option<&GenericParameters>,
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
        .map(|generic| generic.alias_declaration.clone());
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
        .map(|generic| generic.names.clone())
        .unwrap_or_default();

    if !emits_already_defined {
        let head = names::export_type("Emits");
        let emit_fn = names::helper("__EmitFn", &[""]);
        if let Some(type_args) = define_emits_type_args {
            let inner_type = if define_emits_type_requires_setup_scope(summary) {
                let arguments = generic_param
                    .map(|generic| cstr!("<{}>", generic.names))
                    .unwrap_or_default();
                cstr!(
                    "{}[\"__vize_typed_emits\"]",
                    names::setup_return(&arguments)
                )
            } else {
                String::from(inner_type_of(type_args))
            };
            if has_model_emits {
                ts! { &mut *ts =>
                    #head #emits_generic_suffix = #emit_fn<#inner_type> & #emit_fn<{
                        #(model in models =>
                            #("  ") #TsString(cstr!("update:{}", model.name).as_str()): [value: #(model_update_payload(generic_param, model))];
                        )*
                    }>;
                }
            } else {
                ts! { &mut *ts =>
                    #head #emits_generic_suffix = #inner_type;
                }
            }
        } else if has_runtime_emits {
            let setup = names::setup_return("");
            ts! { &mut *ts =>
                #head #emits_generic_suffix = #setup["__vize_emits"]
                #(model in models =>
                    & ((event: #TsString(cstr!("update:{}", model.name).as_str()), value: #(model_update_payload(generic_param, model))) => void)
                )*
                ;
            }
        } else if has_macro_emits || has_model_emits {
            let mut fields: Vec<(String, String)> = Vec::new();
            for emit in summary.macros.emits() {
                fields.push((
                    emit.name.clone(),
                    emit.payload_type.as_deref().unwrap_or("any[]").into(),
                ));
            }
            for model in models {
                let event_name = cstr!("update:{}", model.name);
                let payload = cstr!("[value: {}]", model_update_payload(generic_param, model));
                if let Some((_, existing)) = fields
                    .iter_mut()
                    .find(|(name, _)| name.as_str() == event_name.as_str())
                {
                    // Template `$emit` inference records only the expression's
                    // value type. The model declaration owns the public event
                    // shape, which must remain a tuple for `__EmitShape`.
                    *existing = payload;
                } else {
                    fields.push((event_name, payload));
                }
            }
            ts! { &mut *ts =>
                #head #emits_generic_suffix = {
                    #(field in fields.iter() =>
                        #("  ") #TsString(field.0.as_str()): #(field.1.as_str());
                    )*
                };
            }
        } else {
            ts! { &mut *ts => #head = {}; }
        }
        names::reexport(ts, "Emits");
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
        has_model_emits,
        has_generic_emits: emits_generic_decl.is_some(),
        generic_event_map_decl,
        generic_event_map_names,
        preserve_event_navigation,
        generic_component: generic_param.is_some_and(|generic| !generic.parameters.is_empty()),
    }
}

fn emit_emit_props_helper(
    ts: &mut crate::virtual_ts::TextBuf,
    summary: &Croquis,
    info: &EmitsInfo,
    hoist_shared_preamble: bool,
    event_inference: bool,
) {
    if !info.has_emits_for_props && !event_inference {
        return;
    }
    if !hoist_shared_preamble {
        crate::virtual_ts::helpers::hygiene::emit_helper_text(ts, summary, EMIT_OVERLOAD_HELPERS);
    }
    if !hoist_shared_preamble {
        ts! { &mut *ts => #EMIT_PROPS_HELPER #("\n") }
    }
    if !info.has_emits_for_props {
        return;
    }
    let emit_props =
        crate::virtual_ts::helpers::shared_types::EMIT_PROPS.select(hoist_shared_preamble);
    let static_event_map = names::declared("$StaticEventMap");
    let emit_options = names::helper("__EmitOptions", &[""]);
    let emits = names::declared("Emits");
    if info.has_runtime_emits && !info.has_model_emits {
        if info.preserve_event_navigation {
            ts! { &mut *ts =>
                #("type ") #static_event_map = __EmitOptions<Awaited<ReturnType<typeof __setup>>["__vize_emits"]>;
            }
        }
        ts! { &mut *ts =>
            type $StaticEmitProps = #emit_props<Awaited<ReturnType<typeof __setup>>["__vize_emits"]>;
            #("\n")
        }
    } else {
        if info.preserve_event_navigation {
            if info.generic_event_map_decl.is_empty() {
                ts! { &mut *ts => #("type ") #static_event_map = __EmitOptions<#emits>; }
            } else {
                let declaration = info.generic_event_map_decl.as_str();
                let arguments = info.generic_event_map_names.as_str();
                ts! { &mut *ts =>
                    #("type ") #static_event_map<#declaration> = #emit_options<#emits<#arguments>>;
                }
            }
        }
        ts! { &mut *ts =>
            type $StaticEmitProps = #emit_props<#emits>;
            #("\n")
        }
    }
}

/// `__EmitProps` inside a generic scope: an import type in batch projections,
/// the module helper in standalone ones.
pub(super) fn emit_props_ref(hoist_shared_preamble: bool) -> String {
    if hoist_shared_preamble {
        String::const_new(crate::virtual_ts::helpers::shared_types::EMIT_PROPS.select(true))
    } else {
        names::helper("__EmitProps", &[""])
    }
}
