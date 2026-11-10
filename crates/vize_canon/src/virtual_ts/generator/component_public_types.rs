//! Public slot and expose aliases, captured from the setup and template scopes.

use super::{emits::inner_type_of, generics::module_alias_generic_suffix};
use crate::virtual_ts::generated_names as names;
use crate::virtual_ts::macro_type_mappings::MacroTypeMappings;
use vize_carton::{String, append, cstr};
use vize_croquis::Croquis;
use vize_l0::ts;

/// Emit the private slot contract and preserve the public `Slots` export. When the slots type from
/// `defineSlots` references an SFC generic parameter, the alias re-declares
/// the parameters (with safe defaults) so declaration emit resolves them
/// (#3065).
/// Returns whether the alias re-declared the SFC's type parameters, so the
/// generic component constructor can instantiate it instead of falling back to
/// the declared defaults (#3354).
pub(super) fn emit_slots_type(
    ts: &mut crate::virtual_ts::TextBuf,
    summary: &Croquis,
    mut mappings: MacroTypeMappings<'_>,
    generic_injection: Option<&(String, Vec<String>)>,
    export_slots: bool,
    authored_slots_binding: bool,
    options_api_slots: Option<&str>,
    inferred_slots: bool,
) -> bool {
    let slots_type_args = summary
        .macros
        .define_slots()
        .and_then(|m| m.type_args.as_ref());
    let slots = names::declared("$Slots");
    let is_generic = if let Some(type_args) = slots_type_args {
        let inner_type = inner_type_of(type_args);
        if super::setup_scope::define_slots_type_requires_setup_scope(summary) {
            let (declaration, arguments) = generic_injection
                .map(|(declaration, arguments)| {
                    (
                        cstr!("<{declaration}>"),
                        cstr!("<{}>", arguments.join(", ")),
                    )
                })
                .unwrap_or_default();
            // Preserve the original quote bytes: retained projections and their
            // source coordinates are shared with independent native observers.
            let setup = names::setup_return(&arguments);
            ts! { &mut *ts =>
                #("type ") #slots #declaration = #setup #("['__vize_typed_slots']")
            }
            mappings.emit_slot_key_witnesses(ts, &summary.macros);
            ts! { &mut *ts => ; }
            generic_injection.is_some()
        } else {
            let suffix = module_alias_generic_suffix(generic_injection, inner_type);
            ts! { &mut *ts => #("type ") #slots #suffix = }
            let start = ts.len();
            ts! { &mut *ts => #inner_type }
            let end = ts.len();
            ts! { &mut *ts => ; }
            mappings.map_slot_keys(start..end, &summary.macros);
            !suffix.is_empty()
        }
    } else if let Some(options_api_slots) = options_api_slots {
        ts! { &mut *ts => #("type ") #slots #(" = ") #options_api_slots #(";") }
        false
    } else if inferred_slots {
        let (declaration, arguments) = generic_injection
            .map(|(declaration, arguments)| {
                (
                    cstr!("<{declaration}>"),
                    cstr!("<{}>", arguments.join(", ")),
                )
            })
            .unwrap_or_default();
        let setup = names::setup_return(&arguments);
        ts! { &mut *ts =>
            #("type ") #slots #declaration = #setup #("['__vize_template_slots']");
        }
        generic_injection.is_some()
    } else {
        ts! { &mut *ts => #("type ") #slots = {}; }
        false
    };
    if export_slots {
        if authored_slots_binding {
            // The authored binding is already the public contract named by
            // `defineSlots<Slots>()`. Export that binding instead of declaring
            // a second `Slots` alias in the same type namespace.
            ts! { &mut *ts => export type { Slots }; }
        } else {
            // Re-export without introducing a module-local `Slots` binding.
            // The component marker consumes `$Slots` directly, so an unrelated
            // authored alias or value-only import named `Slots` cannot collide
            // with or replace the generated contract.
            append!(*ts, "export type {{ {slots} as Slots }};\n");
        }
    }
    is_generic
}

/// Whether a module-level authored `Slots` binding is the actual
/// `defineSlots` contract. A same-named declaration is not sufficient: inline
/// slot contracts and setup-local aliases that depend on setup values must use
/// the generated `$Slots` alias.
pub(super) fn uses_module_scoped_authored_slots_binding(summary: &Croquis) -> bool {
    summary
        .macros
        .define_slots()
        .and_then(|call| call.type_args.as_deref())
        .is_some_and(|args| {
            inner_type_of(args).trim() == "Slots"
                && !super::setup_scope::define_slots_type_requires_setup_scope(summary)
        })
}

/// Emit the module-scope `export type Exposed` alias (for `InstanceType` and
/// `useTemplateRef`); returns whether the component exposes anything. A typed
/// `defineExpose` referencing an SFC generic parameter re-declares the
/// parameters just like `Slots` (#3065).
///
/// The second flag reports whether the alias re-declared those parameters, so
/// the generic component constructor can instantiate it (#3354).
pub(super) fn emit_exposed_type(
    ts: &mut crate::virtual_ts::TextBuf,
    summary: &Croquis,
    generic_injection: Option<&(String, Vec<String>)>,
) -> (bool, bool) {
    let exposed = emit_exposed_alias(ts, summary, generic_injection);
    if exposed.0 {
        names::reexport(ts, "Exposed");
    }
    exposed
}

fn emit_exposed_alias(
    ts: &mut crate::virtual_ts::TextBuf,
    summary: &Croquis,
    generic_injection: Option<&(String, Vec<String>)>,
) -> (bool, bool) {
    let Some(expose) = summary.macros.define_expose() else {
        return (false, false);
    };
    let head = names::export_type("Exposed");
    if expose.type_args.is_some()
        && super::setup_scope::define_expose_type_requires_setup_scope(summary)
    {
        // Setup-local names resolve only through the setup return.
        if let Some((declaration, arguments)) = generic_injection {
            let arguments = cstr!("<{}>", arguments.join(", "));
            let setup = names::setup_return(&arguments);
            ts! { &mut *ts =>
                #head<#declaration> = #setup["__vize_typed_exposed"];
            }
            return (true, true);
        }
        let setup = names::setup_return("");
        ts! { &mut *ts =>
            #head = #setup["__vize_typed_exposed"];
        }
        (true, false)
    } else if let Some(ref type_args) = expose.type_args {
        let inner_type = inner_type_of(type_args);
        let suffix = module_alias_generic_suffix(generic_injection, inner_type);
        ts! { &mut *ts => #head #suffix = #inner_type; }
        (true, !suffix.is_empty())
    } else if expose.runtime_args.is_some() {
        // Runtime args are returned from __setup() to keep them in scope.
        // Use Awaited<ReturnType<...>> to handle both sync and async setup.
        if let Some((declaration, arguments)) = generic_injection {
            let arguments = cstr!("<{}>", arguments.join(", "));
            let setup = names::setup_return(&arguments);
            ts! { &mut *ts =>
                #head<#declaration> = #setup["__vize_exposed"];
            }
            return (true, true);
        }
        let setup = names::setup_return("");
        ts! { &mut *ts =>
            #head = #setup["__vize_exposed"];
        }
        (true, false)
    } else {
        (false, false)
    }
}

pub(super) fn infers_slots(
    summary: &Croquis,
    root: Option<&vize_relief::RootNode<'_>>,
    options: crate::virtual_ts::types::VirtualTsCheckOptions,
) -> bool {
    options.check_props
        && options.check_template_bindings
        && crate::virtual_ts::scope::has_inferred_slots(summary, root)
}

/// The component's own slot map in the setup/template lexical scope. Referring
/// to the captured template value preserves SFC generics without passing back
/// through the module-level public component alias.
pub(super) struct TemplateSlotsType {
    pub(super) expression: String,
    pub(super) authored_range: Option<std::ops::Range<usize>>,
}

impl TemplateSlotsType {
    /// Emit the authored type at each context use. An extra generated alias
    /// would hide the useful slot signatures behind its private name in hover.
    pub(super) fn emit(
        &self,
        ts: &mut crate::virtual_ts::TextBuf,
        mappings: &mut Vec<crate::virtual_ts::VizeMapping>,
        source_offset: &dyn Fn(usize) -> usize,
    ) {
        let start = ts.len();
        let expression = self.expression.as_str();
        ts! { &mut *ts => #expression }
        if let Some(range) = &self.authored_range {
            mappings.push(crate::virtual_ts::VizeMapping {
                gen_range: start..ts.len(),
                src_range: source_offset(range.start)..source_offset(range.end),
                sub_spans: Vec::new(),
            });
        }
    }
}

pub(super) fn template_slots_type(
    summary: &Croquis,
    inferred_slots: bool,
    script: Option<&str>,
) -> TemplateSlotsType {
    if let Some(call) = summary.macros.define_slots()
        && let Some(type_args) = call.type_args.as_ref()
    {
        let inner = inner_type_of(type_args);
        let authored_range = script
            .and_then(|script| script.get(call.start as usize..call.end as usize))
            .and_then(|source| source.find(type_args.as_str()))
            .and_then(|start| type_args.find(inner).map(|offset| start + offset))
            .map(|offset| {
                let start = call.start as usize + offset;
                start..start + inner.len()
            });
        TemplateSlotsType {
            expression: inner.into(),
            authored_range,
        }
    } else if inferred_slots {
        TemplateSlotsType {
            expression: "typeof __vize_template".into(),
            authored_range: None,
        }
    } else {
        TemplateSlotsType {
            expression: "{}".into(),
            authored_range: None,
        }
    }
}

impl TemplateSlotsType {
    /// Read the inferred slots from `value` instead of the whole template value.
    pub(super) fn read_template_value(&mut self, value: &str) {
        if self.authored_range.is_none() && self.expression == "typeof __vize_template" {
            self.expression = cstr!("typeof {value}");
        }
    }
}
