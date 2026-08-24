//! v-for item generation.
//!
//! Generates code for individual items within a v-for loop,
//! including props merging, key handling, and block wrapping.

use crate::{
    ElementNode, ElementType, ExpressionNode, PropNode, RuntimeHelper, TemplateChildNode,
    steps::v_memo::{get_memo_exp, has_v_memo},
};

use super::super::{
    children::{generate_children, generate_children_force_array},
    context::CodegenContext,
    element::helpers::{child_namespace, is_dynamic_component, is_is_prop},
    element::{
        generate_custom_directives_closing, generate_vmodel_closing, generate_vshow_closing,
        has_custom_directives, has_vmodel_directive, has_vshow_directive,
    },
    expression::generate_expression,
    helpers::{escape_js_string, is_builtin_component, to_valid_asset_identifier},
    node::generate_node,
    patch_flag::{
        calculate_element_patch_info, calculate_element_patch_info_skip_is, patch_flag_name,
    },
    slots::{generate_slots, has_dynamic_slots_flag, has_slot_children},
};
use super::helpers::{get_element_key, has_other_props, should_skip_prop};
use super::item_props::{
    EventPropAction, event_prop_action, event_prop_sets, generate_event_prop_action,
    is_for_item_segment_skip_prop, strip_need_patch_for_v_for_item,
};
use super::slot_outlet::generate_for_slot_outlet;
use vize_carton::ToCompactString;

/// Generate item for v-for (as block, not regular vnode)
pub fn generate_for_item(ctx: &mut CodegenContext, node: &TemplateChildNode<'_>, is_stable: bool) {
    match node {
        TemplateChildNode::Element(el) => {
            let key_exp = get_element_key(el);
            let is_template = el.tag_type == ElementType::Template;
            let is_component = el.tag_type == ElementType::Component;
            let is_dynamic = is_component && is_dynamic_component(el);
            let prev_skip_scope_id = ctx.skip_scope_id;
            let unwrapped_child = unwrap_template_single_element(el);
            let gen_is_template = is_template && unwrapped_child.is_none();

            // Check for v-memo directive on for item (skip if already handled by v-for)
            let memo_exp = if !ctx.skip_v_memo && has_v_memo(el) {
                get_memo_exp(el)
            } else {
                None
            };

            if let Some(memo_exp) = memo_exp {
                ctx.use_helper(RuntimeHelper::WithMemo);
                ctx.push(ctx.helper(RuntimeHelper::WithMemo));
                ctx.push("(");
                generate_expression(ctx, memo_exp);
                ctx.push(", () => ");
            }

            let has_custom_dirs = has_custom_directives(el);
            if has_custom_dirs {
                ctx.use_helper(RuntimeHelper::WithDirectives);
                ctx.push(ctx.helper(RuntimeHelper::WithDirectives));
                ctx.push("(");
            }

            let has_vmodel = has_vmodel_directive(el) && !has_custom_dirs;
            if has_vmodel {
                ctx.use_helper(RuntimeHelper::WithDirectives);
                ctx.push(ctx.helper(RuntimeHelper::WithDirectives));
                ctx.push("(");
            }

            let has_vshow = has_vshow_directive(el) && !has_vmodel && !has_custom_dirs;
            if has_vshow {
                ctx.use_helper(RuntimeHelper::WithDirectives);
                ctx.use_helper(RuntimeHelper::VShow);
                ctx.push(ctx.helper(RuntimeHelper::WithDirectives));
                ctx.push("(");
            }

            if el.tag_type == ElementType::Slot {
                generate_for_slot_outlet(ctx, el);
            } else if is_stable && !is_component {
                if gen_is_template {
                    ctx.use_helper(RuntimeHelper::OpenBlock);
                    ctx.use_helper(RuntimeHelper::CreateElementBlock);
                    ctx.use_helper(RuntimeHelper::Fragment);
                    ctx.push("(");
                    ctx.push_vnode_helper(RuntimeHelper::OpenBlock);
                    ctx.push("(), ");
                    ctx.push_vnode_helper(RuntimeHelper::CreateElementBlock);
                    ctx.push("(");
                    ctx.push(ctx.helper(RuntimeHelper::Fragment));
                } else {
                    // Stable fragment: use createElementVNode without block wrapper
                    ctx.use_helper(RuntimeHelper::CreateElementVNode);
                    ctx.push_vnode_helper(RuntimeHelper::CreateElementVNode);
                    ctx.push("(\"");
                    let node_el = unwrapped_child.unwrap_or(el);
                    ctx.push(node_el.tag);
                    ctx.push("\"");
                }

                // Props with key and all other props
                let props_el = unwrapped_child.unwrap_or(el);
                let emitted_props = generate_for_item_props(ctx, props_el, key_exp, is_dynamic);

                // Children
                let children_el = unwrapped_child.unwrap_or(el);
                if !children_el.children.is_empty() {
                    if !emitted_props {
                        ctx.push(", null");
                    }
                    ctx.push(", ");
                    if gen_is_template {
                        ctx.push("[");
                        ctx.indent();
                        for (i, child) in children_el.children.iter().enumerate() {
                            if i > 0 {
                                ctx.push(",");
                            }
                            ctx.newline();
                            generate_node(ctx, child);
                        }
                        ctx.deindent();
                        ctx.newline();
                        ctx.push("]");
                    } else {
                        ctx.with_parent_namespace(child_namespace(children_el), |ctx| {
                            generate_children(ctx, &children_el.children);
                        });
                    }
                }

                if gen_is_template {
                    ctx.push(", 64 /* STABLE_FRAGMENT */");
                } else {
                    let (patch_flag, dynamic_props) = calculate_element_patch_info(
                        children_el,
                        ctx.options.binding_metadata.as_ref(),
                        ctx.cache_handlers_in_current_scope(),
                    );
                    let patch_flag = strip_need_patch_for_v_for_item(patch_flag);
                    if children_el.children.is_empty()
                        && (patch_flag.is_some() || dynamic_props.is_some())
                    {
                        if !emitted_props {
                            ctx.push(", null");
                        }
                        ctx.push(", null");
                    }
                    if let Some(flag) = patch_flag {
                        ctx.push(", ");
                        ctx.push(&flag.to_compact_string());
                        ctx.push(" /* ");
                        ctx.push(&patch_flag_name(flag));
                        ctx.push(" */");
                    }
                    if let Some(props) = dynamic_props {
                        ctx.push(", [");
                        for (i, prop) in props.iter().enumerate() {
                            if i > 0 {
                                ctx.push(", ");
                            }
                            ctx.push("\"");
                            ctx.push(prop);
                            ctx.push("\"");
                        }
                        ctx.push("]");
                    }
                }

                if gen_is_template {
                    ctx.push("))");
                } else {
                    ctx.push(")");
                }
            } else {
                // Dynamic list: wrap in block
                ctx.use_helper(RuntimeHelper::OpenBlock);
                ctx.push("(");
                ctx.push_vnode_helper(RuntimeHelper::OpenBlock);
                ctx.push("(), ");

                if is_component {
                    // Component: use createBlock
                    ctx.use_helper(RuntimeHelper::CreateBlock);
                    ctx.push_vnode_helper(RuntimeHelper::CreateBlock);
                    ctx.push("(");
                    // Handle dynamic component
                    if is_dynamic {
                        let dynamic_is = el.props.iter().find_map(|p| {
                            if let PropNode::Directive(dir) = p
                                && dir.name == "bind"
                                && let Some(ExpressionNode::Simple(arg)) = &dir.arg
                                && arg.content == "is"
                            {
                                return dir.exp.as_ref();
                            }
                            None
                        });
                        let static_is = el.props.iter().find_map(|p| {
                            if let PropNode::Attribute(attr) = p
                                && attr.name == "is"
                            {
                                return attr.value.as_ref().map(|v| v.content);
                            }
                            None
                        });
                        if let Some(is_exp) = dynamic_is {
                            ctx.use_helper(RuntimeHelper::ResolveDynamicComponent);
                            ctx.push(ctx.helper(RuntimeHelper::ResolveDynamicComponent));
                            ctx.push("(");
                            generate_expression(ctx, is_exp);
                            ctx.push(")");
                        } else if let Some(name) = static_is {
                            ctx.use_helper(RuntimeHelper::ResolveDynamicComponent);
                            ctx.push(ctx.helper(RuntimeHelper::ResolveDynamicComponent));
                            ctx.push("(\"");
                            ctx.push(name);
                            ctx.push("\")");
                        } else {
                            ctx.push("_component_component");
                        }
                    } else if let Some(builtin) = is_builtin_component(el.tag) {
                        ctx.use_helper(builtin);
                        ctx.push(ctx.helper(builtin));
                    } else if ctx.push_component_binding_tag(el.tag) {
                    } else {
                        ctx.push(&to_valid_asset_identifier("component", el.tag));
                    }
                } else if gen_is_template {
                    // Template with multiple children: use Fragment
                    ctx.use_helper(RuntimeHelper::CreateElementBlock);
                    ctx.use_helper(RuntimeHelper::Fragment);
                    ctx.push_vnode_helper(RuntimeHelper::CreateElementBlock);
                    ctx.push("(");
                    ctx.push(ctx.helper(RuntimeHelper::Fragment));
                } else if let Some(child_el) = unwrapped_child {
                    // Template with single child: unwrap to child element
                    ctx.use_helper(RuntimeHelper::CreateElementBlock);
                    ctx.push_vnode_helper(RuntimeHelper::CreateElementBlock);
                    ctx.push("(\"");
                    ctx.push(child_el.tag);
                    ctx.push("\"");
                } else {
                    // Regular element
                    ctx.use_helper(RuntimeHelper::CreateElementBlock);
                    ctx.push_vnode_helper(RuntimeHelper::CreateElementBlock);
                    ctx.push("(\"");
                    ctx.push(el.tag);
                    ctx.push("\"");
                }

                // Props with key and all other props
                // For unwrapped template child, use child's props with template's key
                let props_el = unwrapped_child.unwrap_or(el);
                let emitted_props = generate_for_item_props(ctx, props_el, key_exp, is_dynamic);

                // Children
                let children_el = unwrapped_child.unwrap_or(el);
                // A component forwarding `v-slots` has slots without having any
                // children of its own, so the slots argument is emitted on its
                // own account rather than off the child list (#3467).
                let component_slots = is_component && has_slot_children(children_el);
                if !children_el.children.is_empty() || component_slots {
                    if !emitted_props {
                        ctx.push(", null");
                    }
                    ctx.push(", ");
                    if component_slots {
                        // Component children must be compiled as slot functions,
                        // not raw children. Otherwise Vue warns:
                        // "Non-function value encountered for default slot"
                        generate_slots(ctx, children_el);
                    } else if gen_is_template {
                        // Template children are array
                        ctx.push("[");
                        ctx.indent();
                        for (i, child) in children_el.children.iter().enumerate() {
                            if i > 0 {
                                ctx.push(",");
                            }
                            ctx.newline();
                            generate_node(ctx, child);
                        }
                        ctx.deindent();
                        ctx.newline();
                        ctx.push("]");
                    } else if ctx.skip_v_memo {
                        // v-for + v-memo: force array form for children
                        if children_el.tag_type == ElementType::Element {
                            ctx.with_parent_namespace(child_namespace(children_el), |ctx| {
                                generate_children_force_array(ctx, &children_el.children);
                            });
                        } else {
                            generate_children_force_array(ctx, &children_el.children);
                        }
                    } else {
                        if children_el.tag_type == ElementType::Element {
                            ctx.with_parent_namespace(child_namespace(children_el), |ctx| {
                                generate_children(ctx, &children_el.children);
                            });
                        } else {
                            generate_children(ctx, &children_el.children);
                        }
                    }
                }

                // Add patch flag
                if is_component {
                    let (mut patch_flag, dynamic_props) = if is_dynamic {
                        calculate_element_patch_info_skip_is(
                            el,
                            ctx.options.binding_metadata.as_ref(),
                            ctx.cache_handlers_in_current_scope(),
                        )
                    } else {
                        calculate_element_patch_info(
                            el,
                            ctx.options.binding_metadata.as_ref(),
                            ctx.cache_handlers_in_current_scope(),
                        )
                    };
                    // Remove TEXT flag for components with slot children (text is inside slot)
                    if has_slot_children(el)
                        && let Some(flag) = patch_flag
                    {
                        let new_flag = flag & !1;
                        patch_flag = if new_flag > 0 { Some(new_flag) } else { None };
                    }
                    // KeepAlive always gets DYNAMIC_SLOTS, and component
                    // slots inside v-for are dynamic by construction.
                    if matches!(el.tag, "KeepAlive" | "keep-alive")
                        || (ctx.in_v_for && has_slot_children(el))
                        || has_dynamic_slots_flag(el, &ctx.source)
                    {
                        let dynamic_slots_flag = 1024;
                        patch_flag = Some(patch_flag.unwrap_or(0) | dynamic_slots_flag);
                    }
                    patch_flag = strip_need_patch_for_v_for_item(patch_flag);
                    // The slots argument already occupies the children slot, so
                    // the `null` placeholder is only for a component that
                    // emitted no children at all.
                    if el.children.is_empty()
                        && !component_slots
                        && (patch_flag.is_some() || dynamic_props.is_some())
                    {
                        if !emitted_props {
                            ctx.push(", null");
                        }
                        ctx.push(", null");
                    }
                    if let Some(flag) = patch_flag {
                        ctx.push(", ");
                        ctx.push(&flag.to_compact_string());
                        ctx.push(" /* ");
                        ctx.push(&patch_flag_name(flag));
                        ctx.push(" */");
                    }
                    if let Some(props) = dynamic_props {
                        ctx.push(", [");
                        for (i, prop) in props.iter().enumerate() {
                            if i > 0 {
                                ctx.push(", ");
                            }
                            ctx.push("\"");
                            ctx.push(prop);
                            ctx.push("\"");
                        }
                        ctx.push("]");
                    }
                } else if gen_is_template {
                    ctx.push(", 64 /* STABLE_FRAGMENT */");
                } else if !ctx.skip_v_memo {
                    // Skip patch flags for v-memo elements (memo handles reactivity)
                    let flag_el = unwrapped_child.unwrap_or(el);
                    let (patch_flag, dynamic_props) = calculate_element_patch_info(
                        flag_el,
                        ctx.options.binding_metadata.as_ref(),
                        ctx.cache_handlers_in_current_scope(),
                    );
                    let patch_flag = strip_need_patch_for_v_for_item(patch_flag);
                    if flag_el.children.is_empty()
                        && (patch_flag.is_some() || dynamic_props.is_some())
                    {
                        if !emitted_props {
                            ctx.push(", null");
                        }
                        ctx.push(", null");
                    }
                    if let Some(flag) = patch_flag {
                        ctx.push(", ");
                        ctx.push(&flag.to_compact_string());
                        ctx.push(" /* ");
                        ctx.push(&patch_flag_name(flag));
                        ctx.push(" */");
                    }
                    if let Some(props) = dynamic_props {
                        ctx.push(", [");
                        for (i, prop) in props.iter().enumerate() {
                            if i > 0 {
                                ctx.push(", ");
                            }
                            ctx.push("\"");
                            ctx.push(prop);
                            ctx.push("\"");
                        }
                        ctx.push("]");
                    }
                }

                ctx.push("))");
            }

            if has_custom_dirs {
                generate_custom_directives_closing(ctx, el);
            }
            if has_vmodel {
                generate_vmodel_closing(ctx, el);
            }
            if has_vshow {
                generate_vshow_closing(ctx, el);
            }

            // Close withMemo wrapper for v-for + v-memo
            if memo_exp.is_some() {
                ctx.push(", _cache, ");
                if let Some(key) = key_exp {
                    generate_expression(ctx, key);
                } else {
                    ctx.push("0");
                }
                ctx.push(")");
            }

            ctx.skip_scope_id = prev_skip_scope_id;
        }
        _ => generate_node(ctx, node),
    }
}

fn unwrap_template_single_element<'a>(el: &'a ElementNode<'a>) -> Option<&'a ElementNode<'a>> {
    if el.tag_type != ElementType::Template || el.children.len() != 1 {
        return None;
    }

    let TemplateChildNode::Element(child_el) = &el.children[0] else {
        return None;
    };

    if child_el.tag_type == ElementType::Element {
        Some(child_el)
    } else {
        None
    }
}

/// Generate props for v-for item, including key and all other props
pub(crate) fn generate_for_item_props(
    ctx: &mut CodegenContext,
    el: &ElementNode<'_>,
    key_exp: Option<&ExpressionNode<'_>>,
    skip_is_prop: bool,
) -> bool {
    let has_other = if skip_is_prop {
        el.props
            .iter()
            .any(|prop| !is_is_prop(prop) && !should_skip_prop(prop))
    } else {
        has_other_props(el)
    };
    // skip_scope_id suppresses duplicate scope attrs for synthetic prop objects.
    let scope_id = if ctx.skip_scope_id {
        None
    } else {
        ctx.options.scope_id.clone()
    };

    if key_exp.is_none() && !has_other && scope_id.is_none() {
        return false;
    }

    ctx.push(", ");

    if !has_other {
        // Only key (and optionally scope_id), no other props
        if let Some(key) = key_exp {
            ctx.push("{ key: ");
            generate_expression(ctx, key);
            if let Some(sid) = scope_id {
                ctx.push(", \"");
                ctx.push(sid.as_str());
                ctx.push("\": \"\"");
            }
            ctx.push(" }");
        } else if let Some(sid) = scope_id {
            // No key, no other props, but has scope_id
            ctx.push("{ \"");
            ctx.push(sid.as_str());
            ctx.push("\": \"\" }");
        }
        return true;
    }

    // Check for v-bind/v-on object spreads (v-bind="obj", v-on="handlers")
    let has_vbind_spread = super::super::props::has_vbind_object(&el.props);
    let has_von_spread = super::super::props::has_von_object(&el.props);

    if has_vbind_spread || has_von_spread {
        generate_for_item_props_merged(
            ctx,
            el,
            key_exp,
            &scope_id,
            has_vbind_spread,
            has_von_spread,
            skip_is_prop,
        );
        return true;
    }

    // Detect static class/style that need to be merged with dynamic :class/:style
    let static_class = el.props.iter().find_map(|p| {
        if let PropNode::Attribute(attr) = p
            && attr.name == "class"
        {
            return attr.value.as_ref().map(|v| v.content);
        }
        None
    });

    let static_style = el.props.iter().find_map(|p| {
        if let PropNode::Attribute(attr) = p
            && attr.name == "style"
        {
            return attr.value.as_ref().map(|v| v.content);
        }
        None
    });

    let has_dynamic_class = el.props.iter().any(|p| {
        if let PropNode::Directive(dir) = p
            && dir.name == "bind"
            && let Some(ExpressionNode::Simple(exp)) = &dir.arg
        {
            return exp.content == "class";
        }
        false
    });

    let has_dynamic_style = el.props.iter().any(|p| {
        if let PropNode::Directive(dir) = p
            && dir.name == "bind"
            && let Some(ExpressionNode::Simple(exp)) = &dir.arg
        {
            return exp.content == "style";
        }
        false
    });

    let skip_static_class = static_class.is_some() && has_dynamic_class;
    let skip_static_style = static_style.is_some() && has_dynamic_style;

    // Static class/style are only merged into the dynamic binding's array when a
    // dynamic counterpart exists; source ordering is preserved via StaticMerge.
    let full_merge = super::super::props::StaticMerge::from_props(&el.props);
    let merge_static = super::super::props::StaticMerge {
        class: if skip_static_class {
            full_merge.class
        } else {
            None
        },
        class_before: full_merge.class_before,
        style: if skip_static_style {
            full_merge.style
        } else {
            None
        },
        style_before: full_merge.style_before,
    };
    let (duplicate_events, mut emitted_events) = event_prop_sets(ctx, &el.props);

    if let Some(key) = key_exp {
        // Merge key with other props
        ctx.push("{");
        ctx.indent();
        ctx.newline();
        ctx.push("key: ");
        generate_expression(ctx, key);

        for prop in el.props.iter() {
            if should_skip_prop(prop) || (skip_is_prop && is_is_prop(prop)) {
                continue;
            }
            if skip_static_class
                && let PropNode::Attribute(attr) = prop
                && attr.name == "class"
            {
                continue;
            }
            if skip_static_style
                && let PropNode::Attribute(attr) = prop
                && attr.name == "style"
            {
                continue;
            }
            let action = event_prop_action(ctx, prop, &duplicate_events, &mut emitted_events);
            if matches!(&action, EventPropAction::Skip) {
                continue;
            }
            ctx.push(",");
            ctx.newline();
            generate_event_prop_action(ctx, action, prop, &el.props, merge_static);
        }

        if let Some(sid) = scope_id {
            ctx.push(",");
            ctx.newline();
            ctx.push("\"");
            ctx.push(sid.as_str());
            ctx.push("\": \"\"");
        }

        ctx.deindent();
        ctx.newline();
        ctx.push("}");
    } else {
        // No key, generate props directly (skipping v-for directive)
        ctx.push("{");
        let mut first = true;
        for prop in el.props.iter() {
            if should_skip_prop(prop) || (skip_is_prop && is_is_prop(prop)) {
                continue;
            }
            if skip_static_class
                && let PropNode::Attribute(attr) = prop
                && attr.name == "class"
            {
                continue;
            }
            if skip_static_style
                && let PropNode::Attribute(attr) = prop
                && attr.name == "style"
            {
                continue;
            }
            let action = event_prop_action(ctx, prop, &duplicate_events, &mut emitted_events);
            if matches!(&action, EventPropAction::Skip) {
                continue;
            }
            if !first {
                ctx.push(",");
            }
            ctx.push(" ");
            generate_event_prop_action(ctx, action, prop, &el.props, merge_static);
            first = false;
        }

        if let Some(sid) = scope_id {
            if !first {
                ctx.push(",");
            }
            ctx.push(" \"");
            ctx.push(sid.as_str());
            ctx.push("\": \"\"");
        }

        ctx.push(" }");
    }
    true
}

/// Generate props using _mergeProps when v-bind/v-on object spreads are present.
fn generate_for_item_props_merged(
    ctx: &mut CodegenContext,
    el: &ElementNode<'_>,
    key_exp: Option<&ExpressionNode<'_>>,
    scope_id: &Option<vize_carton::String>,
    has_vbind_spread: bool,
    has_von_spread: bool,
    skip_is_prop: bool,
) {
    ctx.use_helper(RuntimeHelper::MergeProps);
    ctx.push(ctx.helper(RuntimeHelper::MergeProps));
    ctx.push("(");

    let mut first_merge_arg = true;

    let mut extra_pending = key_exp.is_some() || scope_id.is_some();
    let mut seg_start = 0usize;

    for (index, prop) in el.props.iter().enumerate() {
        let PropNode::Directive(dir) = prop else {
            continue;
        };
        let is_vbind_spread = has_vbind_spread && dir.name == "bind" && dir.arg.is_none();
        let is_von_spread = has_von_spread && dir.name == "on" && dir.arg.is_none();
        if !is_vbind_spread && !is_von_spread {
            continue;
        }

        flush_for_item_props_segment(
            ctx,
            &el.props[seg_start..index],
            key_exp,
            scope_id,
            extra_pending,
            &mut first_merge_arg,
            skip_is_prop,
        );
        extra_pending = false;
        seg_start = index + 1;

        if !first_merge_arg {
            ctx.push(", ");
        }
        first_merge_arg = false;
        if is_vbind_spread {
            if let Some(exp) = &dir.exp {
                generate_expression(ctx, exp);
            }
        } else {
            super::super::props::generate_von_object_exp(ctx, &el.props[index..=index]);
        }
    }

    flush_for_item_props_segment(
        ctx,
        &el.props[seg_start..],
        key_exp,
        scope_id,
        extra_pending,
        &mut first_merge_arg,
        skip_is_prop,
    );

    ctx.push(")");
}

fn flush_for_item_props_segment(
    ctx: &mut CodegenContext,
    props: &[PropNode<'_>],
    key_exp: Option<&ExpressionNode<'_>>,
    scope_id: &Option<vize_carton::String>,
    include_extra: bool,
    first_merge_arg: &mut bool,
    skip_is_prop: bool,
) {
    let has_props = props
        .iter()
        .any(|prop| !is_for_item_segment_skip_prop(prop, skip_is_prop));
    if !has_props && !include_extra {
        return;
    }

    if !*first_merge_arg {
        ctx.push(", ");
    }
    *first_merge_arg = false;

    let skip_static_class = props.iter().any(|prop| {
        matches!(
            prop,
            PropNode::Attribute(attr) if attr.name == "class"
        )
    }) && props.iter().any(|prop| {
        matches!(
            prop,
            PropNode::Directive(dir)
                if dir.name == "bind"
                    && matches!(&dir.arg, Some(ExpressionNode::Simple(exp)) if exp.content == "class")
        )
    });
    let skip_static_style = props.iter().any(|prop| {
        matches!(
            prop,
            PropNode::Attribute(attr) if attr.name == "style"
        )
    }) && props.iter().any(|prop| {
        matches!(
            prop,
            PropNode::Directive(dir)
                if dir.name == "bind"
                    && matches!(&dir.arg, Some(ExpressionNode::Simple(exp)) if exp.content == "style")
        )
    });

    let full_merge = super::super::props::StaticMerge::from_props(props);
    let merge_static = super::super::props::StaticMerge {
        class: if skip_static_class {
            full_merge.class
        } else {
            None
        },
        class_before: full_merge.class_before,
        style: if skip_static_style {
            full_merge.style
        } else {
            None
        },
        style_before: full_merge.style_before,
    };
    let (duplicate_events, mut emitted_events) = event_prop_sets(ctx, props);

    ctx.push("{");
    ctx.indent();
    let mut first_prop = true;
    let prev_skip_normalize = ctx.skip_normalize;
    ctx.skip_normalize = true;

    if include_extra && let Some(key) = key_exp {
        ctx.newline();
        ctx.push("key: ");
        generate_expression(ctx, key);
        first_prop = false;
    }

    for prop in props {
        if is_for_item_segment_skip_prop(prop, skip_is_prop) {
            continue;
        }
        if skip_static_class
            && let PropNode::Attribute(attr) = prop
            && attr.name == "class"
        {
            continue;
        }
        if skip_static_style
            && let PropNode::Attribute(attr) = prop
            && attr.name == "style"
        {
            continue;
        }
        let action = event_prop_action(ctx, prop, &duplicate_events, &mut emitted_events);
        if matches!(&action, EventPropAction::Skip) {
            continue;
        }
        if !first_prop {
            ctx.push(",");
        }
        ctx.newline();
        generate_event_prop_action(ctx, action, prop, props, merge_static);
        first_prop = false;
    }

    if include_extra && let Some(sid) = scope_id {
        if !first_prop {
            ctx.push(",");
        }
        ctx.newline();
        ctx.push("\"");
        ctx.push(sid.as_str());
        ctx.push("\": \"\"");
    }

    ctx.skip_normalize = prev_skip_normalize;
    ctx.deindent();
    ctx.newline();
    ctx.push("}");
}

/// Generate a single prop (attribute or directive)
pub(super) fn generate_single_prop(
    ctx: &mut CodegenContext,
    prop: &PropNode<'_>,
    static_merge: super::super::props::StaticMerge<'_>,
) {
    match prop {
        PropNode::Attribute(attr) => {
            let ref_value = if attr.name == "ref" && ctx.options.inline {
                attr.value.as_ref()
            } else {
                None
            };
            let ref_binding_type = ref_value.and_then(|v| {
                ctx.options
                    .binding_metadata
                    .as_ref()
                    .and_then(|m| m.bindings.get(v.content).copied())
            });
            let should_ref_runtime_binding = matches!(
                ref_binding_type,
                Some(
                    crate::options::BindingType::SetupLet
                        | crate::options::BindingType::SetupRef
                        | crate::options::BindingType::SetupMaybeRef
                )
            );
            let needs_ref_for = attr.name == "ref" && ctx.in_v_for;

            if let (true, Some(ref_value)) = (should_ref_runtime_binding, ref_value) {
                let ref_name = &ref_value.content;
                if needs_ref_for {
                    ctx.push("ref_for: true, ");
                }
                ctx.push("ref_key: \"");
                ctx.push(ref_name);
                ctx.push("\", ref: ");
                ctx.push(ref_name);
                return;
            }

            if needs_ref_for {
                ctx.push("ref_for: true, ");
            }
            let needs_quotes = !super::super::helpers::is_valid_js_identifier(attr.name);
            if needs_quotes {
                ctx.push("\"");
            }
            ctx.push(attr.name);
            if needs_quotes {
                ctx.push("\"");
            }
            ctx.push(": ");
            if let Some(value) = &attr.value {
                if should_ref_runtime_binding {
                    ctx.push(value.content);
                } else {
                    ctx.push("\"");
                    ctx.push(&escape_js_string(value.content));
                    ctx.push("\"");
                }
            } else {
                ctx.push("\"\"");
            }
        }
        PropNode::Directive(dir) => {
            super::super::props::generate_directive_prop_with_static(ctx, dir, static_merge);
        }
    }
}
