//! Emission of the component instance type and its constructors.
//!
//! The generic constructor declares the SFC's type parameters, so module-scope
//! aliases that re-declared those parameters must be instantiated inside it: a
//! bare reference is legal TypeScript that silently resolves against the
//! alias's declared defaults instead of the caller's arguments (#3354).

mod emission;
mod generic_function;
use emission::AliasReference;

use vize_carton::config::VueVersion;
use vize_carton::{String, append, cstr};
use vize_l0::ts;

use super::component_export::GenericComponentContract;
use super::legacy_vue2::{
    exposed_unwrap_helper, generic_instance_suffix, instance_helper, instance_suffix,
    needs_legacy_vue2_helpers,
};
use super::setup_props::SetupPropsPlan;

/// Which module-scope aliases took the SFC's type parameters, plus the flags
/// shaping the emitted instance type.
pub(super) struct ComponentInstanceAliases<'a> {
    pub(super) generic_params: Option<&'a (String, String)>,
    pub(super) generic_parameters:
        Option<&'a crate::virtual_ts::generic_parameters::GenericParameters>,
    pub(super) slots_is_generic: bool,
    pub(super) emits_is_generic: bool,
    pub(super) exposed_is_generic: bool,
    pub(super) has_emits_for_props: bool,
    pub(super) has_exposed_type: bool,
    pub(super) has_root_el: bool,
    /// `inferComponentDollarRefs`: the instance holds the template's refs.
    pub(super) has_refs: bool,
    pub(super) jsx_slots: bool,
    pub(super) has_authored_default: bool,
    pub(super) hoist_shared_preamble: bool,
}

/// Reference to a module-scope alias inside the generic component constructor:
/// instantiated with the SFC's parameters when the alias re-declared them, bare
/// otherwise. Instantiating an alias that took no parameters is a hard
/// `TS2315`, so the decision is per alias.
fn alias_ref<'a>(name: &'a str, is_generic: bool, generic_names: &'a str) -> AliasReference<'a> {
    AliasReference::new(name, is_generic, generic_names)
}

const LEGACY_VUE2_COMPONENT_CONSTRUCTOR_STATICS: &str = r#"type $Vue2ComponentConstructorStatics = {
  extend: (...args: any[]) => any;
  nextTick: (...args: any[]) => any;
  set: (...args: any[]) => any;
  delete: (...args: any[]) => any;
  directive: (...args: any[]) => any;
  filter: (...args: any[]) => any;
  component: (...args: any[]) => any;
  use: (...args: any[]) => any;
  mixin: (...args: any[]) => any;
  compile: (...args: any[]) => any;
  observable: <T>(obj: T) => T;
  util: { warn: (...args: any[]) => void };
  config: any;
  version: string;
};
"#;

pub(super) fn emit_component_constructors<'a>(
    ts: &mut String,
    setup_props_plan: &SetupPropsPlan,
    aliases: &ComponentInstanceAliases<'a>,
    authored_generic: Option<&'a str>,
    legacy_vue2: bool,
    dialect: VueVersion,
) -> Option<GenericComponentContract<'a>> {
    let _profile = vize_l0::profiler::global_profiler()
        .global_span("canon.virtual_ts.emit_component_constructors");
    let legacy_component = needs_legacy_vue2_helpers(legacy_vue2, dialect);
    if aliases.jsx_slots {
        ts.push_str("declare global { namespace JSX { interface ElementChildrenAttribute {} } }\ntype $JsxSlotProps<S> = { [K in keyof JSX.ElementChildrenAttribute]?: S };\n");
    }
    if !legacy_component
        && let Some((declaration, names)) = aliases.generic_params
        && let Some(authored_generic) = authored_generic
    {
        generic_function::emit(
            ts,
            aliases,
            authored_generic,
            declaration,
            names,
            setup_props_plan.component_props_type_ref(),
        );
        return Some(GenericComponentContract {
            // Alias defaults keep bare type references legal. Calls infer from
            // the authored signature, including its genuine defaults only.
            declaration: authored_generic,
            names,
            slots_is_generic: aliases.slots_is_generic,
            public_type: "$GenericComponent",
            props_type: setup_props_plan.component_props_type_ref(),
            ref_resolver_type: Some("$GenericRefResolver"),
        });
    }
    ts.push_str("// ========== Default Export ==========\n");
    ts.push_str(instance_helper(legacy_vue2, dialect));
    if aliases.has_exposed_type {
        ts.push_str(exposed_unwrap_helper(legacy_vue2, dialect));
    }
    if aliases.has_authored_default && legacy_component {
        ts.push_str("type $ComponentInstance = $AuthoredInstance & {\n");
    } else if aliases.has_authored_default {
        ts.push_str(
            "type $ComponentInstance = Omit<$AuthoredInstance, '$props' | '$emit' | '$slots'> & {\n",
        );
    } else {
        ts.push_str("type $ComponentInstance = {\n");
    }
    setup_props_plan.emit_component_props_field(
        ts,
        aliases.has_emits_for_props,
        aliases.generic_params.map(|(decl, _)| decl.as_str()),
        legacy_component,
        aliases.jsx_slots,
        aliases.hoist_shared_preamble,
    );
    if legacy_component {
        ts.push_str("  $emit: __EmitFn<Emits>;\n");
        ts.push_str("  $slots: $Slots;\n");
    } else {
        ts.push_str("  $emit: $PublicEmit<Emits>;\n");
        ts.push_str("  $slots: $PublicSlots<$Slots>;\n");
    }
    ts.push_str(instance_suffix(
        legacy_vue2,
        dialect,
        aliases.has_exposed_type,
        (aliases.has_root_el, aliases.has_refs),
    ));
    if legacy_component {
        ts.push_str(LEGACY_VUE2_COMPONENT_CONSTRUCTOR_STATICS);
        ts.push_str(
            "type $ComponentConstructor = {\n  new (...args: any[]): any;\n  new (...args: any[]): $ComponentInstance;\n} & $Vue2ComponentConstructorStatics;\n",
        );
    } else {
        let mut props_ref = aliases
            .generic_params
            .map(|_| setup_props_plan.generic_fallback_component_props_type_ref())
            .unwrap_or_else(|| setup_props_plan.component_props_type_ref().into());
        if aliases.jsx_slots {
            props_ref = cstr!("{props_ref} & $JsxSlotProps<$Slots>");
        }
        let emit_props_ref = if aliases.has_emits_for_props {
            let emit_props = crate::virtual_ts::helpers::shared_types::EMIT_PROPS
                .select(aliases.hoist_shared_preamble);
            cstr!("{emit_props}<Emits>")
        } else {
            String::from("{}")
        };
        // `$ComponentInput` is a conditional on the authored type
        // parameter, so the checker defers it instead of resolving the input
        // shape while it checks the alias declaration. That matters because
        // `$ComponentInputProps` reaches `$FallthroughAttrs`, whose
        // `Omit<$DomListenerProps, ...>` distributes over the whole DOM
        // event map: eagerly resolving it once per component made `vize check`
        // ~6% slower on a 1000-file corpus, and every file that never
        // instantiates the component paid it for nothing. Call sites still
        // resolve the exact same intersection once `$AuthoredProps` is
        // inferred, so the authored contract is unchanged.
        append!(
            *ts,
            "type $ComponentConstructor = new <$AuthoredProps = unknown>(props?: $AuthoredProps & $ComponentInput<{props_ref}, {emit_props_ref}, $AuthoredProps>, ...args: any[]) => $UsePublicInstance<$AuthoredProps> extends true ? $ComponentInstance : $ReplaceComponentProps<$ComponentInstance, $AuthoredKeyWitness<$AuthoredProps> & $ComponentInputProps<{props_ref}, {emit_props_ref}> & $ComponentInputGuard<{props_ref}, {emit_props_ref}, $AuthoredProps>>;\n"
        );
    }

    let (generic_decl, generic_names) = aliases.generic_params?;
    let slots_ref = alias_ref("$Slots", aliases.slots_is_generic, generic_names);
    let emits_ref = alias_ref("Emits", aliases.emits_is_generic, generic_names);
    let emit_props_field = if aliases.has_emits_for_props {
        let emit_props = crate::virtual_ts::helpers::shared_types::EMIT_PROPS
            .select(aliases.hoist_shared_preamble);
        cstr!(" & {emit_props}<{emits_ref}>")
    } else {
        String::default()
    };
    ts! { &mut *ts =>
        #("type $GenericComponentConstructor = ")
        {
            #("\n  new (...args: any[]): any;\n  new <")
            #generic_decl
            #(">(...args: any[]): ")
            #{
                if aliases.has_authored_default {
                    __ts.push_str("$AuthoredInstance & ");
                }
            }
            #("{\n  $props: $ComponentProps<Props<")
            #generic_names
            #(">>")
            #emit_props_field
            #(";\n  readonly __vizeRawProps?: Props<")
            #generic_names
            #(">;\n  $emit: __EmitFn<")
            #emits_ref
            #(">;\n  $slots: ")
            #slots_ref
            #(";\n")
            #{
                __ts.push_str(&generic_instance_suffix(
                    legacy_vue2,
                    dialect,
                    aliases.has_exposed_type,
                    aliases.exposed_is_generic.then_some(generic_names.as_str()),
                ));
            }
        }
        #(" & $Vue2ComponentConstructorStatics;\n")
    }
    Some(GenericComponentContract {
        declaration: generic_decl,
        names: generic_names,
        slots_is_generic: aliases.slots_is_generic,
        public_type: "$GenericComponentConstructor & $ComponentConstructor & $VueComponentOptions",
        props_type: setup_props_plan.component_props_type_ref(),
        ref_resolver_type: None,
    })
}
