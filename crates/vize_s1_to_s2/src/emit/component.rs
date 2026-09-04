//! Component emission, including slots, builtins, and dynamic components.

pub(super) mod binding;
mod call_props;
mod checks;
mod children_arg;
mod entry;
mod preamble;

use call_props::{
    emit_dynamic_props, has_rendered_attrs, has_rendered_binds, rendered_hoist_attrs,
};
use checks::admit;

use vize_davinci::id::NodeId;
use vize_s2::op::ComponentOp;

use super::EmitCx;
use super::EmitError;
use super::UnsupportedReason as Reason;
use super::buf::Buf;
use super::builtin;
use super::children::children_need_text_flag;
use super::create_slots;
use super::directive;
use super::flag::emit_patch_flag;
use super::js::asset_ident;
use super::props::{
    BindPropsOptions, apply_static_ref_patch, bind_patch, emit_bind_props,
    prune_legacy_patchless_dynamic_props,
};
use super::props_static;
use super::props_static::PropHoistPosition as Position;
use super::slots;

pub(super) use entry::{emit_for_item, emit_fragment_root, emit_if_branch, emit_nested, emit_root};
pub(super) use preamble::{collect_names, emit_resolves};

pub(super) fn emit_call(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
    block: bool,
    if_key: Option<&str>,
    for_item: bool,
    id: Option<NodeId>,
    position: Position,
) -> Result<(), EmitError> {
    admit(cx, component)?;
    let facts = id.and_then(|id| cx.facts.slot_facts.get(id));
    let create = create_slots::needs_create_slots(cx, &component.children);
    let spread = slots::slots_spread(&component.bindings)?;
    let array = builtin::array_children(component.name);
    if array && (create || spread.is_some()) {
        return Err(EmitError::unsupported_at(
            Reason::ArrayBuiltinCannotUseSlotObject,
            component.span,
        ));
    }
    let has_array = array && slots::has_implicit_default(&component.children);
    let has_slots = !array && (facts.is_some() || create || spread.is_some());
    let forwards_slot = super::outlet::has_forwarded_outlet(&component.children);
    let filler_default_props_placeholder =
        !array && !has_slots && slots::filler_default_needs_props_placeholder(&component.children);
    let dynamic_names = create || facts.is_some_and(slots::has_dynamic_names) || spread.is_some();
    let transition_slot_root = builtin::transition_slot_root(component.name);
    let transition_props_slot_hoist = call_props::transition_props_slot_hoist(component, has_slots);
    let alias = if block {
        Buf::create_block_alias()
    } else {
        Buf::create_vnode_alias()
    };
    cx.buf.push(alias);
    cx.buf.push("(");
    if builtin::emit_dynamic_tag(cx, component)? {
    } else if let Some(helper) = builtin::helper(component.name) {
        cx.buf.use_helper(helper);
        cx.buf.push(helper.alias());
    } else if binding::push_tag(cx, component.name) {
    } else {
        cx.buf
            .push(asset_ident("component", component.name).as_str());
    }
    let skip_is = builtin::is_dynamic_component(component);
    let has_binds = has_rendered_binds(component, skip_is);
    let has_attrs = has_rendered_attrs(component, skip_is);
    let has_custom = directive::has_custom(&component.bindings);
    let has_runtime = directive::has_runtime(&component.bindings);
    let has_component_root_slot = call_props::has_component_root_slot(&component.bindings);
    let hoist_attrs = rendered_hoist_attrs(component, skip_is);
    let static_nested = builtin::has_static_nested(&component.children);
    let builtin_helper = builtin::helper(component.name).is_some();
    let hoistable_static_props = if cx.hoisted_scope_id.is_some() {
        None
    } else {
        call_props::hoistable_static_props(component, skip_is, &hoist_attrs, cx.is_ts, cx.scope_id)?
    };
    if for_item
        && !has_component_root_slot
        && !has_custom
        && !has_runtime
        && cx.slot_param_depth == 0
        && let Some(props) = hoistable_static_props.as_ref()
        && props.non_key
        && (props_static::should_hoist(cx, id, props_static::PropHoistPosition::ForItem)
            || (props.dynamic_values && !has_slots && !has_array))
    {
        cx.buf.push_hoist(props.source.clone());
    }
    let static_props_hoist_blocked =
        has_custom || for_item || if_key.is_some() || has_component_root_slot;
    let can_hoist_static_props = !static_props_hoist_blocked
        && call_props::can_hoist_static_props(
            cx,
            component,
            id,
            position,
            has_slots,
            create,
            hoistable_static_props.as_ref(),
        )?;
    let foreign_static_props = id
        .and_then(|id| cx.facts.static_facts.get(id))
        .is_some_and(|fact| fact.foreign && fact.props_hoistable);
    let inline_root_hoist = props_static::inline_root_hoist(cx, id, position);
    let hoisted_static_props = if can_hoist_static_props
        && (inline_root_hoist
            || (!array
                && (facts.is_some() || create || foreign_static_props)
                && (!builtin_helper
                    || static_nested
                    || call_props::children_are_direct_static_vnode_hoists(
                        &component.children,
                        cx.is_ts,
                    )
                    || transition_props_slot_hoist
                    || foreign_static_props))
            || (array && static_nested))
    {
        Some(
            cx.buf.push_hoist(
                hoistable_static_props
                    .as_ref()
                    .expect("checked hoisted props")
                    .source
                    .clone(),
            ),
        )
    } else {
        None
    };
    let branch_unused_hoist = !has_component_root_slot
        && !has_custom
        && !for_item
        && if_key.is_some()
        && cx.template_if_branch_root
        && hoistable_static_props.is_some()
        && (static_nested
            || call_props::children_are_direct_static_vnode_hoists(&component.children, cx.is_ts));
    let unused_hoist = hoisted_static_props.is_none()
        && ((can_hoist_static_props && static_nested) || branch_unused_hoist);
    if unused_hoist {
        cx.buf.push_hoist(
            hoistable_static_props
                .as_ref()
                .expect("checked hoisted props")
                .source
                .clone(),
        );
    }
    let mut patch = bind_patch(
        &component.bindings,
        true,
        if_key,
        for_item,
        cx.is_ts,
        &|name| cx.reads_constant_binding_name(name),
        cx.caches_handlers(),
    );
    if skip_is {
        patch.dynamic_props.retain(|name| name.as_str() != "is");
        if patch.dynamic_props.is_empty() {
            patch.flag &= !8;
        }
        if has_runtime && patch.flag & (2 | 4 | 8 | 16) == 0 {
            patch.flag |= 512;
        }
    }
    if has_slots {
        prune_legacy_patchless_dynamic_props(&component.bindings, &mut patch.dynamic_props);
        if patch.dynamic_props.is_empty() {
            patch.flag &= !8;
        }
    }
    let mut flag = patch.flag;
    if array && children_need_text_flag(cx, &component.children) {
        flag |= 1;
    }
    if (cx.in_v_for && has_slots)
        || dynamic_names
        || builtin::always_dynamic_slots(component.name)
        || (cx.slot_param_depth > 0 && forwards_slot)
    {
        flag |= 1024;
    }
    apply_static_ref_patch(&component.attributes, &mut flag);
    if for_item {
        flag &= !512;
    }
    let emit_flag = flag != 0;
    if let Some(alias) = hoisted_static_props.as_ref() {
        cx.buf.push(", ");
        cx.buf.push(alias.as_str());
    } else if if_key.is_some() || has_binds || has_attrs {
        cx.buf.push(", ");
        emit_bind_props(
            cx,
            &component.attributes,
            &component.bindings,
            BindPropsOptions {
                if_key,
                skip_is,
                for_item,
                is_plain_element: false,
                once_layout: false,
                once_cache_initializer: false,
                force_multiline: for_item && if_key.is_some() && has_component_root_slot,
            },
        )?;
    } else if for_item && directive::has_custom(&component.bindings) {
        cx.buf.push(", { }");
    } else if emit_flag || has_slots || has_array || filler_default_props_placeholder {
        cx.buf.push(", null");
    }
    let previous_template_if_branch_root = cx.template_if_branch_root;
    if previous_template_if_branch_root {
        cx.template_if_branch_root = false;
    }
    let children_result = children_arg::emit(
        cx,
        component,
        children_arg::Args {
            array,
            has_array,
            create,
            facts,
            spread: spread.as_ref(),
            emit_flag,
            keyed_branch: if_key.is_some(),
            transition_slot_root,
        },
    );
    cx.template_if_branch_root = previous_template_if_branch_root;
    children_result?;
    if emit_flag {
        emit_patch_flag(cx, flag);
    }
    emit_dynamic_props(cx, &patch.dynamic_props);
    cx.buf.push(")");
    Ok(())
}
