use super::super::generics::split_generic_params;
use vize_carton::{String, cstr};

/// The child-side slot resolver a parent's `v-slot` scope calls to instantiate
/// this component's generic parameters from the authored props (#4147).
///
/// Only a generic component whose `Slots` alias actually takes those parameters
/// needs it: everywhere else the parent's structural `$slots` probe already
/// yields the exact declared slot map, and a resolver would be an unreferenced
/// widening of the public shape. This inference-only probe accepts partial props;
/// the separate component call checks whether required props are present.
/// Synthetic `= any` defaults are removed so an uninferable call argument falls
/// back to its constraint, exactly as it does through `vue-tsc`'s own generic
/// component signature. Authored defaults such as `U = T` stay intact because
/// they are part of that signature and can determine the slot payload even when
/// no prop directly mentions the parameter.
pub(super) fn slot_resolver_field(
    generic_decl: &str,
    generic_names: &str,
    slots_is_generic: bool,
    props_type: &str,
) -> String {
    if !slots_is_generic {
        return String::default();
    }
    let resolver_decl = strip_synthetic_any_defaults(generic_decl);
    cstr!(
        "__vizeResolveSlots?: <{resolver_decl}>(props: Partial<{props_type}<{generic_names}>> & Record<string, unknown>) => __VizeSlots<{generic_names}>; "
    )
}

/// The prop parameter a parent's template calls on a generic child.
///
/// Keep the declared props intact: optionalizing them loses required-prop
/// diagnostics and mapped key witnesses can prevent generic inference. They accept Vue's global
/// HTML attr surface (`id`, camelized `aria-*`, `data*`, etc.) even when the
/// generated child is closed, matching the strict non-generic checker. What
/// they must not need is an unconditional string index: generated parents
/// already know whether this generated child can fall attributes through, so
/// arbitrary unknown props should only be accepted by a real fallthrough target.
/// Keep the generic fallthrough tail value-open: intersecting native element
/// props here makes declared component props such as `color` collide with DOM
/// attributes, while non-generic usages still get value-sensitive fallthrough
/// checks from their per-prop aliases.
pub(super) fn generic_check_props_param(
    generic_names: &str,
    fallthrough_props_ref: Option<&str>,
    props_type: &str,
) -> String {
    let mut param = cstr!(
        "{props_type}<{generic_names}> & import('vue').VNodeProps & import('vue').AllowedComponentProps & import('vue').ComponentCustomProps & __VizeComponentGlobalHtmlAttrs"
    );
    if fallthrough_props_ref.is_some() {
        param.push_str(" & Record<string, unknown>");
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
pub(super) fn strip_synthetic_any_defaults(generic_decl: &str) -> String {
    let mut stripped = String::default();
    for param in split_generic_params(generic_decl) {
        if !stripped.is_empty() {
            stripped.push_str(", ");
        }
        stripped.push_str(param_without_synthetic_any_default(param).as_str());
    }
    stripped
}

/// One parameter declaration with only its generated `= any` suffix removed; `=>` never terminates it.
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
