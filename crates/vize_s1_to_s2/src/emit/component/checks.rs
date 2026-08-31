use vize_s2::op::{BindingOp, ComponentOp, DynamicName};

use super::super::{EmitCx, EmitError, create_slots, slots};

pub(super) fn admit(cx: &EmitCx<'_>, component: &ComponentOp<'_>) -> Result<(), EmitError> {
    if create_slots::needs_create_slots(cx, &component.children)
        || slots::has_implicit_default(&component.children)
    {
        slots::admit_default(&component.children)?;
    }
    super::super::props::admit_bindings(&component.attributes, &component.bindings)
}

pub(super) fn has_dynamic_key_binding(component: &ComponentOp<'_>) -> bool {
    component.bindings.iter().any(|binding| {
        matches!(
            binding,
            BindingOp::Bind(bind)
                if bind.value.is_some()
                    && matches!(bind.name, Some(DynamicName::Static("key")))
        )
    })
}
