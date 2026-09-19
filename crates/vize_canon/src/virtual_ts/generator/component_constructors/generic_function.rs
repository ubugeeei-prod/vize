//! Vue generic SFCs expose a function, not a constructible public instance.

use super::super::generics::generic_suffix;
use super::{ComponentInstanceAliases, alias_ref};
use crate::virtual_ts::props::strip_const_modifiers;
use vize_carton::{String, append, cstr};

pub(super) fn emit(
    ts: &mut String,
    aliases: &ComponentInstanceAliases<'_>,
    authored_parameters: &str,
    alias_parameters: &str,
    names: &str,
) {
    let alias_parameters = generic_suffix(&strip_const_modifiers(alias_parameters));
    let authored_parameters = generic_suffix(authored_parameters);
    let arguments = generic_suffix(names);
    let emits = alias_ref("Emits", aliases.emits_is_generic, names);
    let slots = alias_ref("__VizeSlots", aliases.slots_is_generic, names);
    let expose = if aliases.has_exposed_type {
        alias_ref("Exposed", aliases.exposed_is_generic, names)
    } else {
        String::from("{}")
    };
    let mut expose = cstr!("import('vue').ShallowUnwrapRef<{expose}>");
    if aliases.has_root_el {
        append!(
            expose,
            " & {{ $el: Awaited<ReturnType<typeof __setup{arguments}>>['__vize_root_el'] }}"
        );
    }
    let mut listeners = if aliases.has_emits_for_props {
        cstr!(" & __EmitProps<{emits}>")
    } else {
        String::default()
    };
    if aliases.jsx_slots {
        append!(listeners, " & __VizeJsxSlotProps<{slots}>");
    }
    let emit = if aliases.has_emits_for_props {
        cstr!("__VizePublicEmit<{emits}>")
    } else {
        String::from("{}")
    };
    append!(
        *ts,
        "type __VizeGenericContext{alias_parameters} = {{\n  props: Props{arguments}{listeners} & import('vue').VNodeProps & import('vue').AllowedComponentProps & import('vue').ComponentCustomProps;\n  attrs: any;\n  emit: {emit};\n  slots: {slots};\n  expose: (exposed: {expose}) => void;\n}};\n"
    );
    // Use the authored parameter declaration: erasing a generated `= any`
    // default by spelling cannot distinguish it from an authored default.
    append!(
        *ts,
        "type __VizeGenericComponent = {authored_parameters}(props: __VizeGenericContext{arguments}['props'], ctx?: Pick<__VizeGenericContext{arguments}, 'attrs' | 'emit' | 'slots'>, expose?: __VizeGenericContext{arguments}['expose']) => import('vue').VNode & {{ __ctx?: __VizeGenericContext{arguments} }};\n"
    );
}
