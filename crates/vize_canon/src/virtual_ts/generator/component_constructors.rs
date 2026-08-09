//! Emission of the component instance type and its constructors.
//!
//! The generic constructor declares the SFC's type parameters, so module-scope
//! aliases that re-declared those parameters must be instantiated inside it: a
//! bare reference is legal TypeScript that silently resolves against the
//! alias's declared defaults instead of the caller's arguments (#3354).

use vize_carton::config::VueVersion;
use vize_carton::{String, append, cstr};

use super::legacy_vue2::{
    exposed_unwrap_helper, generic_instance_suffix, instance_helper, instance_suffix,
};
use super::setup_props::SetupPropsPlan;

/// Which module-scope aliases took the SFC's type parameters, plus the flags
/// shaping the emitted instance type.
pub(super) struct ComponentInstanceAliases<'a> {
    pub(super) generic_params: Option<&'a (String, String)>,
    pub(super) slots_is_generic: bool,
    pub(super) emits_is_generic: bool,
    pub(super) exposed_is_generic: bool,
    pub(super) has_emits_for_props: bool,
    pub(super) has_exposed_type: bool,
    pub(super) has_authored_default: bool,
}

/// Reference to a module-scope alias inside the generic component constructor:
/// instantiated with the SFC's parameters when the alias re-declared them, bare
/// otherwise. Instantiating an alias that took no parameters is a hard
/// `TS2315`, so the decision is per alias.
fn alias_ref(name: &str, is_generic: bool, generic_names: &str) -> String {
    if is_generic {
        cstr!("{name}<{generic_names}>")
    } else {
        String::from(name)
    }
}

pub(super) fn emit_component_constructors(
    ts: &mut String,
    setup_props_plan: &SetupPropsPlan,
    aliases: &ComponentInstanceAliases<'_>,
    legacy_vue2: bool,
    dialect: VueVersion,
) {
    ts.push_str("// ========== Default Export ==========\n");
    ts.push_str(instance_helper(legacy_vue2, dialect));
    if aliases.has_exposed_type {
        ts.push_str(exposed_unwrap_helper(legacy_vue2, dialect));
    }
    if aliases.has_authored_default {
        ts.push_str("type __VizeComponentInstance = __VizeAuthoredInstance & {\n");
    } else {
        ts.push_str("type __VizeComponentInstance = {\n");
    }
    setup_props_plan.emit_component_props_field(
        ts,
        aliases.has_emits_for_props,
        aliases.generic_params.map(|(decl, _)| decl.as_str()),
    );
    ts.push_str("  $emit: __EmitFn<Emits>;\n");
    ts.push_str("  $slots: Slots;\n");
    ts.push_str(instance_suffix(
        legacy_vue2,
        dialect,
        aliases.has_exposed_type,
    ));
    ts.push_str(
        "type __VizeComponentConstructor = new (...args: any[]) => __VizeComponentInstance;\n",
    );

    let Some((generic_decl, generic_names)) = aliases.generic_params else {
        return;
    };
    let slots_ref = alias_ref("Slots", aliases.slots_is_generic, generic_names);
    let emits_ref = alias_ref("Emits", aliases.emits_is_generic, generic_names);
    let emit_props_field = if aliases.has_emits_for_props {
        cstr!(" & __EmitProps<{emits_ref}>")
    } else {
        String::default()
    };
    append!(
        *ts,
        "type __VizeGenericComponentConstructor = new <{generic_decl}>(...args: any[]) => "
    );
    if aliases.has_authored_default {
        ts.push_str("__VizeAuthoredInstance & ");
    }
    append!(
        *ts,
        "{{\n  $props: __VizeComponentProps<Props<{generic_names}>>{emit_props_field};\n  readonly __vizeRawProps?: Props<{generic_names}>;\n  $emit: __EmitFn<{emits_ref}>;\n  $slots: {slots_ref};\n"
    );
    ts.push_str(&generic_instance_suffix(
        legacy_vue2,
        dialect,
        aliases.has_exposed_type,
        aliases.exposed_is_generic.then_some(generic_names.as_str()),
    ));
}
