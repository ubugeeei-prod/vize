use crate::script::ScriptCompileContext;
use crate::types::{BindingMetadata, BindingType};

pub(super) fn merge_props_bindings(
    script_bindings: &mut BindingMetadata,
    ctx: &ScriptCompileContext,
) {
    for (name, bt) in &ctx.bindings.bindings {
        if matches!(bt, BindingType::Props | BindingType::PropsAliased) {
            script_bindings.bindings.entry(name.clone()).or_insert(*bt);
        }
    }
    if let Some(destructure) = &ctx.macros.props_destructure
        && let Some(rest) = &destructure.rest_id
    {
        script_bindings
            .bindings
            .insert(rest.clone(), BindingType::SetupReactiveConst);
    }
    for (local, key) in &ctx.bindings.props_aliases {
        script_bindings
            .props_aliases
            .entry(local.clone())
            .or_insert_with(|| key.clone());
    }
}
