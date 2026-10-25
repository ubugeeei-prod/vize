//! Vue generic SFCs expose a function, not a constructible public instance.

use super::super::generics::generic_suffix;
use super::emission::{GenericContext, GenericSignature, SignatureRole};
use super::{ComponentInstanceAliases, alias_ref};
use crate::virtual_ts::props::strip_const_modifiers;
use vize_carton::{String, append};

pub(super) fn emit(
    ts: &mut String,
    aliases: &ComponentInstanceAliases<'_>,
    authored_parameters: &str,
    alias_parameters: &str,
    names: &str,
    props_type: &str,
) {
    let alias_parameters = generic_suffix(&strip_const_modifiers(alias_parameters));
    let authored_parameters = generic_suffix(authored_parameters);
    let arguments = generic_suffix(names);
    let emits = alias_ref("Emits", aliases.emits_is_generic, names);
    let slots = alias_ref("__VizeSlots", aliases.slots_is_generic, names);
    let expose = if aliases.has_exposed_type {
        alias_ref("Exposed", aliases.exposed_is_generic, names)
    } else {
        alias_ref("{}", false, names)
    };
    append!(
        *ts,
        "{}",
        GenericContext {
            parameters: &alias_parameters,
            arguments: &arguments,
            props_type,
            emits,
            slots,
            exposed: expose,
            emit_props: aliases.has_emits_for_props,
            jsx_slots: aliases.jsx_slots,
            root_element: aliases.has_root_el,
            template_refs: aliases.has_refs,
        }
    );
    // Use the authored parameter declaration: erasing a generated `= any`
    // default by spelling cannot distinguish it from an authored default.
    append!(
        *ts,
        "{}",
        GenericSignature {
            role: SignatureRole::Component,
            parameters: &authored_parameters,
            arguments: &arguments
        }
    );
    // Template-ref probing repeats the authored props only to instantiate this
    // return context. Its mapped component check already owns prop diagnostics,
    // so accept extra attributes here just as the ordinary open checker does.
    append!(
        *ts,
        "{}",
        GenericSignature {
            role: SignatureRole::TemplateRefResolver,
            parameters: &authored_parameters,
            arguments: &arguments
        }
    );
}
