use vize_croquis::{Croquis, EventHandlerScopeData, Scope};

use super::find_component_usage_for_event;

pub(super) fn requires_unresolved_handler_implicit_any(
    summary: &Croquis,
    component_name: &str,
    data: &EventHandlerScopeData,
    scope: &Scope,
) -> bool {
    let Some((_, usage)) = find_component_usage_for_event(summary, component_name, data, scope)
    else {
        return false;
    };

    usage.slots.iter().any(|slot| {
        slot.has_scope
            && slot.scope_vars.iter().any(|slot_var| {
                data.param_names
                    .iter()
                    .any(|param| param.as_str() == slot_var.as_str())
            })
    })
}
