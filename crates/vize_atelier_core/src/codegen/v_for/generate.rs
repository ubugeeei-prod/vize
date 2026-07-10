//! v-for item generation.
//!
//! Generates code for individual items within a v-for loop,
//! including props merging, key handling, and block wrapping.

use crate::{
    ElementNode, ElementType, ExpressionNode, PropNode, RuntimeHelper, TemplateChildNode,
    rendu::RenduChildren,
    steps::v_memo::{get_memo_exp, has_v_memo},
};

use super::super::{
    children::{emit_children_array_body, generate_children, generate_children_force_array},
    context::CodegenContext,
    element::helpers::{child_namespace, is_dynamic_component},
    element::{
        generate_custom_directives_closing, generate_vmodel_closing, generate_vshow_closing,
        has_custom_directives, has_vmodel_directive, has_vshow_directive,
    },
    expression::generate_expression,
    helpers::{is_builtin_component, to_valid_asset_identifier},
    node::generate_node,
    patch_flag::{
        calculate_element_patch_info, calculate_element_patch_info_skip_is, patch_flag_name,
    },
    slots::{
        generate_slot_outlet_name, generate_slot_outlet_props, generate_slots, has_slot_children,
        has_slot_outlet_props,
    },
};
use super::helpers::get_element_key;
use super::item_props::generate_for_item_props;
use vize_carton::ToCompactString;

fn strip_need_patch_for_v_for_item(patch_flag: Option<i32>) -> Option<i32> {
    patch_flag.and_then(|flag| {
        let next = flag & !512;
        (next > 0).then_some(next)
    })
}

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
                    ctx.push(ctx.helper(RuntimeHelper::OpenBlock));
                    ctx.push("(), ");
                    ctx.push(ctx.helper(RuntimeHelper::CreateElementBlock));
                    ctx.push("(");
                    ctx.push(ctx.helper(RuntimeHelper::Fragment));
                } else {
                    // Stable fragment: use createElementVNode without block wrapper
                    ctx.use_helper(RuntimeHelper::CreateElementVNode);
                    ctx.push(ctx.helper(RuntimeHelper::CreateElementVNode));
                    ctx.push("(\"");
                    let node_el = unwrapped_child.unwrap_or(el);
                    ctx.push(&node_el.tag);
                    ctx.push("\"");
                }

                // Props with key and all other props
                let props_el = unwrapped_child.unwrap_or(el);
                generate_for_item_props(ctx, props_el, key_exp, is_dynamic);

                // Children
                let children_el = unwrapped_child.unwrap_or(el);
                if !children_el.children.is_empty() {
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
                ctx.push(ctx.helper(RuntimeHelper::OpenBlock));
                ctx.push("(), ");

                if is_component {
                    // Component: use createBlock
                    ctx.use_helper(RuntimeHelper::CreateBlock);
                    ctx.push(ctx.helper(RuntimeHelper::CreateBlock));
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
                                return attr.value.as_ref().map(|v| v.content.as_str());
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
                    } else if let Some(builtin) = is_builtin_component(&el.tag) {
                        ctx.use_helper(builtin);
                        ctx.push(ctx.helper(builtin));
                    } else if ctx.push_component_binding_tag(&el.tag) {
                    } else {
                        ctx.push(&to_valid_asset_identifier("component", &el.tag));
                    }
                } else if gen_is_template {
                    // Template with multiple children: use Fragment
                    ctx.use_helper(RuntimeHelper::CreateElementBlock);
                    ctx.use_helper(RuntimeHelper::Fragment);
                    ctx.push(ctx.helper(RuntimeHelper::CreateElementBlock));
                    ctx.push("(");
                    ctx.push(ctx.helper(RuntimeHelper::Fragment));
                } else if let Some(child_el) = unwrapped_child {
                    // Template with single child: unwrap to child element
                    ctx.use_helper(RuntimeHelper::CreateElementBlock);
                    ctx.push(ctx.helper(RuntimeHelper::CreateElementBlock));
                    ctx.push("(\"");
                    ctx.push(&child_el.tag);
                    ctx.push("\"");
                } else {
                    // Regular element
                    ctx.use_helper(RuntimeHelper::CreateElementBlock);
                    ctx.push(ctx.helper(RuntimeHelper::CreateElementBlock));
                    ctx.push("(\"");
                    ctx.push(&el.tag);
                    ctx.push("\"");
                }

                // Props with key and all other props
                // For unwrapped template child, use child's props with template's key
                let props_el = unwrapped_child.unwrap_or(el);
                generate_for_item_props(ctx, props_el, key_exp, is_dynamic);

                // Children
                let children_el = unwrapped_child.unwrap_or(el);
                if !children_el.children.is_empty() {
                    ctx.push(", ");
                    if is_component && has_slot_children(children_el) {
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
                    if matches!(el.tag.as_str(), "KeepAlive" | "keep-alive")
                        || (ctx.in_v_for && has_slot_children(el))
                    {
                        let dynamic_slots_flag = 1024;
                        patch_flag = Some(patch_flag.unwrap_or(0) | dynamic_slots_flag);
                    }
                    patch_flag = strip_need_patch_for_v_for_item(patch_flag);
                    if el.children.is_empty() && (patch_flag.is_some() || dynamic_props.is_some()) {
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

fn generate_for_slot_outlet(ctx: &mut CodegenContext, el: &ElementNode<'_>) {
    ctx.use_helper(RuntimeHelper::RenderSlot);
    ctx.push(ctx.helper(RuntimeHelper::RenderSlot));
    ctx.push("(_ctx.$slots, ");
    generate_slot_outlet_name(ctx, el);

    let has_slot_props = has_slot_outlet_props(el);
    let has_rendered_children = RenduChildren::new(&el.children).rendered().next().is_some();

    if has_rendered_children {
        if has_slot_props {
            ctx.push(", ");
            generate_slot_outlet_props(ctx, el);
        } else {
            ctx.push(", {}");
        }
        ctx.push(", () => [");
        ctx.indent();
        emit_children_array_body(ctx, &el.children);
        ctx.deindent();
        ctx.newline();
        ctx.push("])");
    } else if has_slot_props {
        ctx.push(", ");
        generate_slot_outlet_props(ctx, el);
        ctx.push(")");
    } else {
        ctx.push(")");
    }
}
