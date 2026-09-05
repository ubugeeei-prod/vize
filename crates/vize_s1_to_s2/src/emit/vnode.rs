//! Static native HTML element / children emission.

mod array_child;
mod checks;

pub(super) use array_child::emit_array_child;
use vize_davinci::id::NodeId;
use vize_s2::op::ElementOp;

use super::buf::Buf;
mod slot_child_props;

use slot_child_props::scoped_for_slot_component_slot_child_props;

use super::children::children_need_text_flag;
use super::directive;
use super::flag::emit_patch_flag;
use super::namespace;
use super::props::{
    BindPropsOptions, admit_element_bindings, apply_static_ref_patch, bind_patch, emit_bind_props,
};
use super::props_dynamic::emit_dynamic_props;
use super::props_static::PropHoistPosition;
use super::vnode_children::emit_children;
use super::{EmitCx, EmitError};
use checks::{
    direct_static_children_hoisted, has_cloak, has_dynamic_key_binding,
    has_interpolation_descendant, has_prop_bindings,
    template_if_branch_root_has_direct_interpolation,
};

pub(super) fn emit_unique_element(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    if super::once::has(&element.bindings) {
        return super::once::emit_element(cx, element, None, false);
    }
    super::memo::emit_cached(cx, &element.bindings, |cx| {
        directive::wrap_element(cx, element, |cx| {
            cx.buf.use_open_block();
            cx.buf.use_create_element_block();
            cx.buf.push("(");
            cx.buf.push(Buf::open_block_alias());
            cx.buf.push("(), ");
            emit_call(
                cx,
                element,
                true,
                None,
                (true, id, PropHoistPosition::Root),
                false,
                false,
            )?;
            cx.buf.push(")");
            Ok(())
        })
    })
}

fn emit_block(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    if_key: Option<&str>,
    for_item: bool,
    hoist: (bool, Option<NodeId>, PropHoistPosition),
) -> Result<(), EmitError> {
    directive::wrap_element(cx, element, |cx| {
        cx.buf.use_open_block();
        cx.buf.use_create_element_block();
        cx.buf.push("(");
        cx.buf.push(Buf::open_block_alias());
        cx.buf.push("(), ");
        emit_call(cx, element, true, if_key, hoist, for_item, false)?;
        cx.buf.push(")");
        Ok(())
    })
}

pub(super) fn emit_fragment_element(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    if super::once::has(&element.bindings) {
        return super::once::emit_element(cx, element, None, false);
    }
    if namespace::crosses_boundary(cx, element, direct_static_children_hoisted(cx, element, id)) {
        return super::memo::emit_cached(cx, &element.bindings, |cx| {
            emit_block(
                cx,
                element,
                None,
                false,
                (true, id, PropHoistPosition::Root),
            )
        });
    }
    if has_dynamic_key_binding(element) {
        return emit_nested_block(cx, element, id);
    }
    super::memo::emit_cached(cx, &element.bindings, |cx| {
        directive::wrap_element(cx, element, |cx| {
            cx.buf.use_create_element_vnode();
            emit_call(
                cx,
                element,
                false,
                None,
                (true, id, PropHoistPosition::Root),
                false,
                false,
            )
        })
    })
}

fn emit_nested(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    if super::once::has(&element.bindings) {
        return super::once::emit_element(cx, element, None, false);
    }
    if namespace::crosses_boundary(cx, element, direct_static_children_hoisted(cx, element, id)) {
        return emit_nested_block(cx, element, id);
    }
    if has_dynamic_key_binding(element) {
        return emit_nested_block(cx, element, id);
    }
    super::memo::emit_cached(cx, &element.bindings, |cx| {
        directive::wrap_element(cx, element, |cx| {
            cx.buf.use_create_element_vnode();
            emit_call(
                cx,
                element,
                false,
                None,
                (true, id, PropHoistPosition::Nested),
                false,
                false,
            )
        })
    })
}

fn emit_nested_block(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    super::memo::emit_cached(cx, &element.bindings, |cx| {
        emit_block(
            cx,
            element,
            None,
            false,
            (true, id, PropHoistPosition::Nested),
        )
    })
}

pub(super) fn emit_if_branch_element(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    key: &str,
) -> Result<(), EmitError> {
    emit_block(
        cx,
        element,
        Some(key),
        false,
        (true, None, PropHoistPosition::Nested),
    )
}

pub(super) fn emit_call(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    block: bool,
    if_key: Option<&str>,
    hoist: (bool, Option<NodeId>, PropHoistPosition),
    for_item: bool,
    once: bool,
) -> Result<(), EmitError> {
    admit_element_bindings(&element.attributes, &element.bindings)?;
    let (allow_hoist, id, prop_hoist) = hoist;
    let once_layout = once || super::once::has(&element.bindings);
    let alias = if block {
        Buf::create_element_block_alias()
    } else {
        Buf::create_element_vnode_alias()
    };
    cx.buf.push(alias);
    cx.buf.push("(\"");
    cx.buf.push(element.tag);
    cx.buf.push("\"");
    let has_children = !element.children.ops.is_empty();
    let hoist_static_children = super::vnode_static::should_hoist_static_children(
        cx,
        element,
        id,
        allow_hoist,
        if_key.is_some() && !for_item,
        for_item,
    );
    let has_memo = super::memo::has(&element.bindings);
    let memo_block = block && has_memo && !(if_key.is_some() && !for_item);
    let force_array_children = once
        || memo_block
        || (directive::has_custom(&element.bindings) && allow_hoist && block && if_key.is_none());
    let cache_static_children = allow_hoist
        && cx.static_cache
        && !hoist_static_children
        && !force_array_children
        && !for_item
        && !cx.in_v_for
        && cx.slot_param_depth == 0
        && !template_if_branch_root_has_direct_interpolation(cx, element, if_key);
    let has_binds = has_prop_bindings(&element.bindings);
    let conditional_v_for_dynamic_text = cx.conditional_v_for_item
        && matches!(prop_hoist, PropHoistPosition::Nested)
        && has_interpolation_descendant(element);
    let should_hoist_props = super::props_static::should_hoist(cx, id, prop_hoist)
        || scoped_for_slot_component_slot_child_props(cx, element, prop_hoist);
    let hoisted_props =
        if allow_hoist && if_key.is_none() && !conditional_v_for_dynamic_text && should_hoist_props
        {
            super::props_static::root_hoist_props(
                &element.attributes,
                &element.bindings,
                cx.is_ts,
                cx.scope_id,
            )?
        } else {
            None
        };
    let hoist = hoisted_props.is_some();
    let patch = bind_patch(
        &element.bindings,
        false,
        if_key,
        for_item,
        cx.is_ts,
        &|name| cx.reads_constant_binding_name(name),
        &|on| super::on::caches_handler(cx, on),
        cx.caches_handlers(),
    );
    let text_flag = !once && !memo_block && children_need_text_flag(cx, &element.children);
    let mut flag = patch.flag;
    if text_flag {
        flag |= 1;
    }
    apply_static_ref_patch(&element.attributes, &mut flag);
    if for_item {
        flag &= !512;
    }
    if once {
        flag &= 2 | 4;
    }
    if memo_block {
        flag = 0;
    }
    let omit_text_only = hoist && block && flag == 1;
    let emit_flag = flag != 0 && !omit_text_only;
    let omit_empty_once_patch_child = once && !has_children && flag & !(2 | 4) == 0;
    let empty_runtime_for = for_item
        && (directive::has_runtime(&element.bindings) || has_cloak(&element.bindings))
        && !has_binds
        && element.attributes.is_empty()
        && if_key.is_none();
    if hoist {
        let props_alias = cx
            .buf
            .hoist_root_props(hoisted_props.expect("checked hoisted props"));
        cx.buf.push(", ");
        cx.buf.push(props_alias.as_str());
    } else if if_key.is_some() || has_binds {
        cx.buf.push(", ");
        emit_bind_props(
            cx,
            &element.attributes,
            &element.bindings,
            BindPropsOptions {
                if_key,
                skip_is: false,
                for_item,
                is_plain_element: true,
                once_layout,
                once_cache_initializer: once,
                force_multiline: false,
            },
        )?;
    } else if !element.attributes.is_empty() {
        cx.buf.push(", ");
        super::props_static::emit_inline(cx, element.attributes.iter(), once_layout);
    } else if let Some(scope_id) = cx
        .scope_id_here()
        .filter(|_| cx.in_v_for && (has_children || emit_flag))
    {
        cx.buf.push(", ");
        let props = super::hoist::compact_props_object(element.attributes.iter(), Some(scope_id));
        cx.buf.push(props.as_str());
    } else if empty_runtime_for {
        cx.buf.push(", { }");
    } else if has_children || emit_flag {
        cx.buf.push(", null");
    }
    if has_children {
        cx.buf.push(", ");
        let template_if_branch_root = cx.template_if_branch_root;
        cx.with_once_element(|cx| {
            namespace::with_child(cx, element, |cx| {
                let previous_template_if_branch_root = cx.template_if_branch_root;
                let previous_suppress_template_for_child_key = cx.suppress_template_for_child_key;
                if template_if_branch_root {
                    cx.template_if_branch_root = false;
                }
                if previous_suppress_template_for_child_key {
                    cx.suppress_template_for_child_key = false;
                }
                let result = emit_children(
                    cx,
                    &element.children,
                    force_array_children,
                    hoist_static_children,
                    cache_static_children,
                );
                cx.template_if_branch_root = previous_template_if_branch_root;
                cx.suppress_template_for_child_key = previous_suppress_template_for_child_key;
                result
            })
        })?;
    } else if emit_flag && !omit_empty_once_patch_child {
        cx.buf.push(", null");
    }
    if emit_flag {
        emit_patch_flag(cx, flag);
    }
    let suppress_memo_for_item_dynamic_props = memo_block && cx.skip_memo && for_item;
    if !once && !suppress_memo_for_item_dynamic_props {
        emit_dynamic_props(cx, &patch.dynamic_props);
    }
    cx.buf.push(")");
    Ok(())
}
