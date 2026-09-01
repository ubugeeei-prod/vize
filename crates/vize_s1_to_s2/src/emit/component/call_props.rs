use alloc::vec::Vec as StdVec;

use vize_davinci::id::NodeId;
use vize_s0::String;
use vize_s2::op::{Attribute, BindingOp, ComponentOp};

use super::super::EmitCx;
use super::super::hoist::compact_props_object;
use super::super::props_static::ComponentHoistProps;
use super::super::{EmitError, builtin, directive, props_static, slots};

pub(super) fn has_rendered_binds(component: &ComponentOp<'_>, skip_is: bool) -> bool {
    component.bindings.iter().any(|binding| {
        !(matches!(binding, BindingOp::SlotContent(_))
            || slots::is_slots_spread(binding)
            || directive::is_runtime(binding)
            || super::super::memo::is_memo(binding)
            || super::super::once::is_once(binding)
            || matches!(binding, BindingOp::VueCloak(_))
            || (skip_is && builtin::is_is_bind(binding)))
    })
}

pub(super) fn has_rendered_attrs(component: &ComponentOp<'_>, skip_is: bool) -> bool {
    component
        .attributes
        .iter()
        .any(|attr| !skip_is || attr.name != "is")
}

pub(super) fn rendered_hoist_attrs<'a, 'b>(
    component: &'b ComponentOp<'a>,
    skip_is: bool,
) -> StdVec<&'b Attribute<'a>> {
    component
        .attributes
        .iter()
        .filter(|attr| !skip_is || attr.name != "is")
        .collect()
}

pub(super) fn hoistable_static_props(
    component: &ComponentOp<'_>,
    skip_is: bool,
    hoist_attrs: &[&Attribute<'_>],
) -> Result<Option<ComponentHoistProps>, EmitError> {
    if skip_is {
        return Ok((!hoist_attrs.is_empty()).then(|| ComponentHoistProps {
            source: compact_props_object(hoist_attrs.iter().copied()),
            dynamic_values: false,
            non_key: hoist_attrs.iter().any(|attr| attr.name != "key"),
            valued_prop: hoist_attrs.iter().any(|attr| attr.value.is_some()),
            all_static_binds: false,
        }));
    }
    props_static::component_hoist_props(&component.attributes, &component.bindings)
}

pub(super) fn can_hoist_static_props(
    cx: &EmitCx<'_>,
    component: &ComponentOp<'_>,
    id: Option<NodeId>,
    if_key: Option<&str>,
    for_item: bool,
    has_custom: bool,
    has_slots: bool,
    props: Option<&ComponentHoistProps>,
) -> bool {
    let Some(props) = props else {
        return false;
    };
    if has_custom || for_item || if_key.is_some() {
        return false;
    }
    let static_vnode_context =
        cx.hoist_static_vnodes && slots::has_text_only_implicit_default(&component.children);
    let loop_context = cx.in_v_for
        && (slots::has_text_only_implicit_default(&component.children) || props.all_static_binds);
    let hoist_context = static_vnode_context || loop_context || cx.slot_param_depth > 0;
    props_static::should_hoist(cx, id, props_static::PropHoistPosition::Nested)
        || id.is_some_and(|id| {
            cx.template_for_item_root_id == Some(id)
                && props_static::props_hoistable(cx, Some(id))
                && !directive::has_runtime(&component.bindings)
        })
        || (!props.dynamic_values && props.valued_prop && hoist_context && !has_slots)
        || (props.dynamic_values
            && cx.slot_param_depth == 0
            && !cx.in_v_for
            && (!cx.hoist_static_vnodes
                || !has_slots
                || slots::has_text_only_implicit_default(&component.children)))
}

pub(super) fn emit_dynamic_props(cx: &mut EmitCx<'_>, names: &[String]) {
    if names.is_empty() {
        return;
    }
    cx.buf.push(", [");
    for (i, name) in names.iter().enumerate() {
        if i > 0 {
            cx.buf.push(", ");
        }
        cx.buf.push("\"");
        cx.buf.push(name.as_str());
        cx.buf.push("\"");
    }
    cx.buf.push("]");
}
